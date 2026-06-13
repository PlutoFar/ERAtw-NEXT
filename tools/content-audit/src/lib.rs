//! M1 readonly content audit.
//!
//! This crate reads filesystem metadata, ERB/CSV text for aggregate statistics,
//! and resource bytes for hashes. It never executes files, accesses the network,
//! or emits source content bodies.

mod resource_analysis;
mod text_analysis;

use jsonschema::{Draft, JSONSchema};
use resource_analysis::analyze_resource;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;
use text_analysis::{analyze_csv, analyze_erb, CsvFileAnalysis, ErbFileAnalysis};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub const SUMMARY_SCHEMA_VERSION: &str = "content-audit-summary/v1";
pub const DEFAULT_ALLOWED_SOURCE: &str = r"D:\AICODE\eratw-content";
pub const DEFAULT_PROFILE: &str = "m1-readonly";

#[derive(Debug)]
pub enum AuditError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidArgs(String),
    UnsafeSource(String),
    UnsafeOutput(String),
    Validation(String),
}

impl Display for AuditError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Json(err) => write!(f, "JSON error: {err}"),
            Self::InvalidArgs(message) => write!(f, "invalid arguments: {message}"),
            Self::UnsafeSource(message) => write!(f, "unsafe source: {message}"),
            Self::UnsafeOutput(message) => write!(f, "unsafe output: {message}"),
            Self::Validation(message) => write!(f, "report validation failed: {message}"),
        }
    }
}

impl Error for AuditError {}

impl From<std::io::Error> for AuditError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for AuditError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

pub type AuditResult<T> = Result<T, AuditError>;

#[derive(Debug, Clone)]
pub struct AuditOptions {
    pub source: PathBuf,
    pub out_dir: PathBuf,
    pub allowed_source: PathBuf,
    pub profile: String,
}

impl AuditOptions {
    pub fn new(source: PathBuf, out_dir: PathBuf) -> Self {
        Self {
            source,
            out_dir,
            allowed_source: PathBuf::from(DEFAULT_ALLOWED_SOURCE),
            profile: DEFAULT_PROFILE.to_string(),
        }
    }

    pub fn with_allowed_source(mut self, allowed_source: PathBuf) -> Self {
        self.allowed_source = allowed_source;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditPolicy {
    pub readonly: bool,
    pub network: bool,
    pub execute: bool,
    pub follow_reparse_points: bool,
    pub exclude_git: bool,
    pub profile: String,
}

impl AuditPolicy {
    fn m1(profile: &str) -> Self {
        Self {
            readonly: true,
            network: false,
            execute: false,
            follow_reparse_points: false,
            exclude_git: true,
            profile: profile.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSummary {
    pub schema_version: String,
    pub source_root: String,
    pub generated_at: String,
    pub policy: AuditPolicy,
    pub totals: AuditTotals,
    pub extensions: Vec<ExtensionSummary>,
    pub risks: Vec<RiskRecord>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditTotals {
    pub files: u64,
    pub directories: u64,
    pub bytes: u64,
    pub excluded_directories: u64,
    pub reparse_points_skipped: u64,
    pub max_path_length: usize,
    pub long_path_files: u64,
    pub no_extension_files: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRecord {
    pub relative_path: String,
    pub kind: String,
    pub extension: String,
    pub size_bytes: u64,
    pub modified_time: Option<String>,
    pub depth: usize,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryRecord {
    pub relative_path: String,
    pub depth: usize,
    pub file_count: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionSummary {
    pub extension: String,
    pub files: u64,
    pub bytes: u64,
    pub kinds: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskRecord {
    pub code: String,
    pub severity: String,
    pub relative_path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErbStats {
    pub schema_version: String,
    pub erb_files: u64,
    pub erb_header_files: u64,
    pub bytes: u64,
    pub body_read: bool,
    pub token_stats_collected: bool,
    pub encoding_counts: BTreeMap<String, u64>,
    pub decode_error_files: Vec<String>,
    pub lines: u64,
    pub blank_lines: u64,
    pub comment_lines: u64,
    pub token_counts: BTreeMap<String, u64>,
    pub resource_references: Vec<ReferenceSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvStats {
    pub schema_version: String,
    pub csv_files: u64,
    pub csv_directory_files: u64,
    pub bytes: u64,
    pub special_files: Vec<String>,
    pub body_read: bool,
    pub row_column_stats_collected: bool,
    pub encoding_counts: BTreeMap<String, u64>,
    pub decode_error_files: Vec<String>,
    pub totals: CsvAggregateTotals,
    pub files: Vec<CsvFileStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesStats {
    pub schema_version: String,
    pub resource_directory_files: u64,
    pub resource_directory_non_asset_files: u64,
    pub image_files: u64,
    pub audio_files: u64,
    pub font_files: u64,
    pub bytes: u64,
    pub body_read: bool,
    pub hash_collected: bool,
    pub files: Vec<ResourceFileStats>,
    pub duplicate_hash_groups: Vec<Vec<String>>,
    pub duplicate_file_names: Vec<DuplicateNameGroup>,
    pub same_stem_different_extensions: Vec<DuplicateNameGroup>,
    pub unresolved_reference_candidates: Vec<String>,
    pub unknown_author_files: u64,
    pub unknown_license_files: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSummary {
    pub file_name: String,
    pub occurrences: u64,
    pub resolved: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvAggregateTotals {
    pub rows: u64,
    pub blank_rows: u64,
    pub duplicate_first_column_values: u64,
    pub parse_errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvFileStats {
    pub relative_path: String,
    pub encoding: String,
    pub decode_errors: bool,
    pub delimiter: String,
    pub rows: u64,
    pub min_columns: u64,
    pub max_columns: u64,
    pub blank_rows: u64,
    pub duplicate_first_column_values: u64,
    pub parse_errors: u64,
    pub special_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceFileStats {
    pub relative_path: String,
    pub file_name: String,
    pub stem: String,
    pub extension: String,
    pub media_type: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub numeric_prefix: Option<u64>,
    pub variant_tokens: Vec<String>,
    pub author: String,
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateNameGroup {
    pub key: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AuditReport {
    pub summary: AuditSummary,
    pub files: Vec<FileRecord>,
    pub directories: Vec<DirectoryRecord>,
    pub erb_stats: ErbStats,
    pub csv_stats: CsvStats,
    pub resources: ResourcesStats,
}

#[derive(Default)]
struct Scanner {
    totals: AuditTotals,
    files: Vec<FileRecord>,
    risks: Vec<RiskRecord>,
    directories: BTreeMap<String, DirectoryRecord>,
    extensions: BTreeMap<String, ExtensionSummary>,
    erb_stats: ErbStatsBuilder,
    csv_stats: CsvStatsBuilder,
    resources: ResourcesStatsBuilder,
}

#[derive(Default)]
struct ErbStatsBuilder {
    erb_files: u64,
    erb_header_files: u64,
    bytes: u64,
    encoding_counts: BTreeMap<String, u64>,
    decode_error_files: Vec<String>,
    lines: u64,
    blank_lines: u64,
    comment_lines: u64,
    token_counts: BTreeMap<String, u64>,
    resource_reference_counts: BTreeMap<String, u64>,
    resolved_resource_names: BTreeSet<String>,
}

#[derive(Default)]
struct CsvStatsBuilder {
    csv_files: u64,
    csv_directory_files: u64,
    bytes: u64,
    special_files: BTreeSet<String>,
    encoding_counts: BTreeMap<String, u64>,
    decode_error_files: Vec<String>,
    totals: CsvAggregateTotals,
    files: Vec<CsvFileStats>,
}

#[derive(Default)]
struct ResourcesStatsBuilder {
    resource_directory_files: u64,
    resource_directory_non_asset_files: u64,
    image_files: u64,
    audio_files: u64,
    font_files: u64,
    bytes: u64,
    files: Vec<ResourceFileStats>,
    unresolved_reference_candidates: Vec<String>,
}

pub fn run_audit(options: &AuditOptions) -> AuditResult<AuditReport> {
    if options.profile != DEFAULT_PROFILE {
        return Err(AuditError::InvalidArgs(format!(
            "unsupported profile '{}'; expected '{}'",
            options.profile, DEFAULT_PROFILE
        )));
    }

    let source_root = validate_source(&options.source, &options.allowed_source)?;
    let out_dir = validate_output_path(&options.out_dir, &source_root)?;
    let generated_at = now_rfc3339();
    let mut scanner = Scanner::default();
    scanner.scan_dir(&source_root, Path::new(""))?;
    scanner.finalize_cross_checks();

    let mut extensions: Vec<_> = scanner.extensions.into_values().collect();
    extensions.sort_by(|a, b| {
        b.files
            .cmp(&a.files)
            .then_with(|| b.bytes.cmp(&a.bytes))
            .then_with(|| a.extension.cmp(&b.extension))
    });

    let erb_stats = scanner.erb_stats.finish();
    let csv_stats = scanner.csv_stats.finish();
    let resources = scanner.resources.finish();
    let summary = AuditSummary {
        schema_version: SUMMARY_SCHEMA_VERSION.to_string(),
        source_root: normalize_path(&source_root),
        generated_at,
        policy: AuditPolicy::m1(&options.profile),
        totals: scanner.totals,
        extensions,
        risks: scanner.risks,
    };

    let report = AuditReport {
        erb_stats,
        csv_stats,
        resources,
        directories: scanner.directories.into_values().collect(),
        files: scanner.files,
        summary,
    };

    validate_report_contracts(&report)?;
    write_reports(&out_dir, &report)?;
    Ok(report)
}

fn validate_report_contracts(report: &AuditReport) -> AuditResult<()> {
    validate_schema_value(
        "content-audit-summary.schema.json",
        &report.summary,
        "summary",
    )?;
    validate_schema_values(
        "content-audit-file-record.schema.json",
        &report.files,
        "file record",
    )?;
    validate_schema_values(
        "content-audit-directory-record.schema.json",
        &report.directories,
        "directory record",
    )?;
    validate_schema_value(
        "content-audit-erb-stats.schema.json",
        &report.erb_stats,
        "ERB statistics",
    )?;
    validate_schema_value(
        "content-audit-csv-stats.schema.json",
        &report.csv_stats,
        "CSV statistics",
    )?;
    validate_schema_value(
        "content-audit-resources.schema.json",
        &report.resources,
        "resource statistics",
    )?;
    Ok(())
}

fn validate_schema_values<T: Serialize>(
    schema_file: &str,
    values: &[T],
    label: &str,
) -> AuditResult<()> {
    let compiled = compile_schema(schema_file)?;
    for value in values {
        let instance = serde_json::to_value(value)?;
        if let Err(errors) = compiled.validate(&instance) {
            let paths: Vec<_> = errors
                .take(5)
                .map(|error| error.instance_path.to_string())
                .collect();
            return Err(AuditError::Validation(format!(
                "{label} does not match {schema_file} at {}",
                paths.join(", ")
            )));
        };
    }
    Ok(())
}

fn validate_schema_value<T: Serialize>(
    schema_file: &str,
    value: &T,
    label: &str,
) -> AuditResult<()> {
    validate_schema_values(schema_file, std::slice::from_ref(value), label)
}

fn compile_schema(schema_file: &str) -> AuditResult<JSONSchema> {
    let schema_path = repository_root().join("schemas").join(schema_file);
    let schema: serde_json::Value =
        serde_json::from_reader(BufReader::new(File::open(&schema_path)?))?;
    JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema)
        .map_err(|error| {
            AuditError::Validation(format!(
                "could not compile '{}': {error}",
                schema_path.display()
            ))
        })
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("content-audit must live under tools/")
        .to_path_buf()
}

fn validate_source(source: &Path, allowed_source: &Path) -> AuditResult<PathBuf> {
    if !source.is_absolute() {
        return Err(AuditError::UnsafeSource(
            "--source must be an absolute path".to_string(),
        ));
    }
    if has_parent_dir_component(source) {
        return Err(AuditError::UnsafeSource(
            "--source must not contain parent-directory traversal".to_string(),
        ));
    }
    if looks_like_unc_path(source) {
        return Err(AuditError::UnsafeSource(
            "UNC/network paths are not allowed".to_string(),
        ));
    }

    let source_root = source.canonicalize().map_err(|err| {
        AuditError::UnsafeSource(format!(
            "source does not resolve to a real directory: {err}"
        ))
    })?;
    if !source_root.is_dir() {
        return Err(AuditError::UnsafeSource(
            "source must resolve to a directory".to_string(),
        ));
    }

    let allowed_root = allowed_source.canonicalize().map_err(|err| {
        AuditError::UnsafeSource(format!("allowed source does not resolve: {err}"))
    })?;
    if source_root != allowed_root {
        return Err(AuditError::UnsafeSource(format!(
            "source '{}' is outside allowlist '{}'",
            normalize_path(&source_root),
            normalize_path(&allowed_root)
        )));
    }

    Ok(source_root)
}

fn has_parent_dir_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

fn looks_like_unc_path(path: &Path) -> bool {
    let text = path.as_os_str().to_string_lossy();
    text.starts_with(r"\\") || text.starts_with("//")
}

fn validate_output_path(out_dir: &Path, source_root: &Path) -> AuditResult<PathBuf> {
    if has_parent_dir_component(out_dir) {
        return Err(AuditError::UnsafeOutput(
            "--out must not contain parent-directory traversal".to_string(),
        ));
    }
    if looks_like_unc_path(out_dir) {
        return Err(AuditError::UnsafeOutput(
            "UNC/network output paths are not allowed".to_string(),
        ));
    }

    let absolute_out = if out_dir.is_absolute() {
        out_dir.to_path_buf()
    } else {
        std::env::current_dir()?.join(out_dir)
    };
    if absolute_out.exists() {
        return Err(AuditError::UnsafeOutput(
            "--out must not already exist".to_string(),
        ));
    }

    let resolved_out = resolve_new_path(&absolute_out)?;
    if path_is_within(&resolved_out, source_root) {
        return Err(AuditError::UnsafeOutput(
            "audit reports must not be written inside the readonly source".to_string(),
        ));
    }
    Ok(absolute_out)
}

fn resolve_new_path(path: &Path) -> AuditResult<PathBuf> {
    let mut ancestor = path;
    let mut suffix = Vec::new();
    while !ancestor.exists() {
        let name = ancestor.file_name().ok_or_else(|| {
            AuditError::UnsafeOutput("output path has no existing ancestor".to_string())
        })?;
        suffix.push(name.to_os_string());
        ancestor = ancestor.parent().ok_or_else(|| {
            AuditError::UnsafeOutput("output path has no existing ancestor".to_string())
        })?;
    }
    let mut resolved = ancestor.canonicalize()?;
    for component in suffix.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn path_is_within(path: &Path, parent: &Path) -> bool {
    let path = normalize_path(path).to_lowercase();
    let parent = normalize_path(parent).to_lowercase();
    path == parent || path.starts_with(&(parent + "/"))
}

impl Scanner {
    fn scan_dir(&mut self, root: &Path, relative_dir: &Path) -> AuditResult<()> {
        let absolute_dir = root.join(relative_dir);
        let mut entries = Vec::new();
        for entry in fs::read_dir(&absolute_dir)? {
            entries.push(entry?);
        }
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let file_name = entry.file_name();
            let child_relative = relative_dir.join(&file_name);
            let relative_path = normalize_path(&child_relative);

            if file_name == OsStr::new(".git") {
                self.totals.excluded_directories += 1;
                continue;
            }

            let metadata = fs::symlink_metadata(entry.path())?;
            if is_reparse_point_or_symlink(&metadata) {
                self.totals.reparse_points_skipped += 1;
                self.risks.push(RiskRecord {
                    code: "REPARSE_POINT_SKIPPED".to_string(),
                    severity: "blocker".to_string(),
                    relative_path,
                    message: "Reparse point skipped by readonly audit policy.".to_string(),
                });
                continue;
            }

            if metadata.is_dir() {
                self.totals.directories += 1;
                self.track_directory(&child_relative, 0, 0);
                self.scan_dir(root, &child_relative)?;
            } else if metadata.is_file() {
                self.track_file(root, &child_relative, &metadata)?;
            } else {
                self.risks.push(RiskRecord {
                    code: "UNSUPPORTED_FILE_TYPE".to_string(),
                    severity: "warning".to_string(),
                    relative_path,
                    message: "Unsupported filesystem entry type skipped.".to_string(),
                });
            }
        }

        Ok(())
    }

    fn track_file(
        &mut self,
        root: &Path,
        relative: &Path,
        metadata: &fs::Metadata,
    ) -> AuditResult<()> {
        let relative_path = normalize_path(relative);
        let extension = normalized_extension(relative);
        let kind = classify_file(relative, &extension).to_string();
        let size_bytes = metadata.len();
        let depth = path_depth(relative);
        let mut flags = Vec::new();

        self.totals.files += 1;
        self.totals.bytes += size_bytes;
        self.totals.max_path_length = self.totals.max_path_length.max(relative_path.len());

        if extension.is_empty() {
            self.totals.no_extension_files += 1;
            flags.push("no_extension".to_string());
            self.risks.push(RiskRecord {
                code: "NO_EXTENSION_FILE".to_string(),
                severity: "info".to_string(),
                relative_path: relative_path.clone(),
                message: "File has no extension and needs manual classification.".to_string(),
            });
        }
        if !relative_path.is_ascii() {
            flags.push("non_ascii_path".to_string());
        }
        if relative_path.len() > 240 {
            self.totals.long_path_files += 1;
            flags.push("long_path".to_string());
            self.risks.push(RiskRecord {
                code: "LONG_PATH".to_string(),
                severity: "warning".to_string(),
                relative_path: relative_path.clone(),
                message: "Relative path length exceeds 240 characters.".to_string(),
            });
        }
        if matches!(
            kind.as_str(),
            "tool_script" | "runtime_binary" | "runtime_state" | "archive"
        ) {
            flags.push("excluded_candidate".to_string());
            self.risks.push(RiskRecord {
                code: format!("{}_PRESENT", kind.to_ascii_uppercase()),
                severity: if kind == "runtime_binary" {
                    "blocker".to_string()
                } else {
                    "warning".to_string()
                },
                relative_path: relative_path.clone(),
                message: "File is reported as metadata only and must not be executed.".to_string(),
            });
        }

        let modified_time = metadata.modified().ok().map(format_system_time);
        let record = FileRecord {
            relative_path: relative_path.clone(),
            kind: kind.clone(),
            extension: extension.clone(),
            size_bytes,
            modified_time,
            depth,
            flags,
        };

        self.track_parent_directories(relative, size_bytes);
        self.track_extension(&extension, &kind, size_bytes);
        self.track_domain_stats(root, relative, &record);
        self.files.push(record);
        Ok(())
    }

    fn track_parent_directories(&mut self, relative: &Path, size_bytes: u64) {
        let mut parent = relative.parent();
        while let Some(dir) = parent {
            if dir.as_os_str().is_empty() {
                break;
            }
            self.track_directory(dir, 1, size_bytes);
            parent = dir.parent();
        }
    }

    fn track_directory(&mut self, relative: &Path, file_count_delta: u64, bytes_delta: u64) {
        let relative_path = normalize_path(relative);
        let entry = self
            .directories
            .entry(relative_path.clone())
            .or_insert_with(|| DirectoryRecord {
                relative_path,
                depth: path_depth(relative),
                file_count: 0,
                total_bytes: 0,
            });
        entry.file_count += file_count_delta;
        entry.total_bytes += bytes_delta;
    }

    fn track_extension(&mut self, extension: &str, kind: &str, bytes: u64) {
        let key = if extension.is_empty() {
            "(none)"
        } else {
            extension
        }
        .to_string();
        let entry = self
            .extensions
            .entry(key.clone())
            .or_insert_with(|| ExtensionSummary {
                extension: key,
                files: 0,
                bytes: 0,
                kinds: BTreeMap::new(),
            });
        entry.files += 1;
        entry.bytes += bytes;
        *entry.kinds.entry(kind.to_string()).or_insert(0) += 1;
    }

    fn track_domain_stats(&mut self, root: &Path, relative: &Path, record: &FileRecord) {
        let absolute = root.join(relative);
        match record.kind.as_str() {
            "erb" => {
                self.erb_stats.erb_files += 1;
                self.erb_stats.bytes += record.size_bytes;
                self.analyze_erb_file(&absolute, record);
            }
            "erb_header" => {
                self.erb_stats.erb_header_files += 1;
                self.erb_stats.bytes += record.size_bytes;
                self.analyze_erb_file(&absolute, record);
            }
            "csv" => {
                self.csv_stats.csv_files += 1;
                self.csv_stats.bytes += record.size_bytes;
                if top_component(relative).eq_ignore_ascii_case("CSV") {
                    self.csv_stats.csv_directory_files += 1;
                }
                if is_special_csv(relative) {
                    self.csv_stats
                        .special_files
                        .insert(record.relative_path.clone());
                }
                self.analyze_csv_file(&absolute, relative, record);
            }
            "resource_image" => {
                self.resources.image_files += 1;
                self.resources.bytes += record.size_bytes;
                self.analyze_resource_file(&absolute, record);
            }
            "resource_audio" => {
                self.resources.audio_files += 1;
                self.resources.bytes += record.size_bytes;
                self.analyze_resource_file(&absolute, record);
            }
            "font" => {
                self.resources.font_files += 1;
                self.resources.bytes += record.size_bytes;
                self.analyze_resource_file(&absolute, record);
            }
            _ => {}
        }

        if top_component(relative).eq_ignore_ascii_case("resources") {
            self.resources.resource_directory_files += 1;
            if !matches!(
                record.kind.as_str(),
                "resource_image" | "resource_audio" | "font"
            ) {
                self.resources.resource_directory_non_asset_files += 1;
            }
        }
    }

    fn analyze_erb_file(&mut self, absolute: &Path, record: &FileRecord) {
        match analyze_erb(absolute) {
            Ok(analysis) => self.erb_stats.add(record, analysis),
            Err(err) => self.risks.push(RiskRecord {
                code: "ERB_ANALYSIS_FAILED".to_string(),
                severity: "warning".to_string(),
                relative_path: record.relative_path.clone(),
                message: format!("ERB aggregate analysis failed: {err}"),
            }),
        }
    }

    fn analyze_csv_file(&mut self, absolute: &Path, relative: &Path, record: &FileRecord) {
        match analyze_csv(absolute) {
            Ok(analysis) => self.csv_stats.add(relative, record, analysis),
            Err(err) => self.risks.push(RiskRecord {
                code: "CSV_ANALYSIS_FAILED".to_string(),
                severity: "warning".to_string(),
                relative_path: record.relative_path.clone(),
                message: format!("CSV structural analysis failed: {err}"),
            }),
        }
    }

    fn analyze_resource_file(&mut self, absolute: &Path, record: &FileRecord) {
        match analyze_resource(absolute) {
            Ok(analysis) => {
                let file_name = absolute
                    .file_name()
                    .map(|value| value.to_string_lossy().to_string())
                    .unwrap_or_default();
                let stem = absolute
                    .file_stem()
                    .map(|value| value.to_string_lossy().to_string())
                    .unwrap_or_default();
                self.resources.files.push(ResourceFileStats {
                    relative_path: record.relative_path.clone(),
                    file_name,
                    stem,
                    extension: record.extension.clone(),
                    media_type: record.kind.clone(),
                    size_bytes: record.size_bytes,
                    sha256: analysis.sha256,
                    numeric_prefix: analysis.numeric_prefix,
                    variant_tokens: analysis.variant_tokens,
                    author: "unknown".to_string(),
                    license: "unknown".to_string(),
                });
            }
            Err(err) => self.risks.push(RiskRecord {
                code: "RESOURCE_HASH_FAILED".to_string(),
                severity: "warning".to_string(),
                relative_path: record.relative_path.clone(),
                message: format!("Resource hash failed: {err}"),
            }),
        }
    }

    fn finalize_cross_checks(&mut self) {
        let resource_names: BTreeSet<String> = self
            .resources
            .files
            .iter()
            .map(|resource| resource.file_name.to_lowercase())
            .collect();
        for reference in self.erb_stats.resource_reference_counts.keys() {
            if resource_names.contains(&reference.to_lowercase()) {
                self.erb_stats
                    .resolved_resource_names
                    .insert(reference.to_lowercase());
            } else {
                self.resources
                    .unresolved_reference_candidates
                    .push(reference.clone());
            }
        }
        self.resources.unresolved_reference_candidates.sort();

        for reference in &self.resources.unresolved_reference_candidates {
            self.risks.push(RiskRecord {
                code: "MISSING_RESOURCE_REFERENCE_CANDIDATE".to_string(),
                severity: "warning".to_string(),
                relative_path: reference.clone(),
                message:
                    "ERB token resembles a resource reference but no matching file name exists."
                        .to_string(),
            });
        }
    }
}

impl ErbStatsBuilder {
    fn add(&mut self, record: &FileRecord, analysis: ErbFileAnalysis) {
        *self.encoding_counts.entry(analysis.encoding).or_insert(0) += 1;
        if analysis.decode_errors {
            self.decode_error_files.push(record.relative_path.clone());
        }
        self.lines += analysis.lines;
        self.blank_lines += analysis.blank_lines;
        self.comment_lines += analysis.comment_lines;
        add_count(
            &mut self.token_counts,
            "function_definitions",
            analysis.function_definitions,
        );
        add_count(&mut self.token_counts, "calls", analysis.calls);
        add_count(
            &mut self.token_counts,
            "conditionals",
            analysis.conditionals,
        );
        add_count(
            &mut self.token_counts,
            "select_cases",
            analysis.select_cases,
        );
        add_count(
            &mut self.token_counts,
            "print_commands",
            analysis.print_commands,
        );
        add_count(
            &mut self.token_counts,
            "variable_references",
            analysis.variable_references,
        );
        for reference in analysis.resource_references {
            *self.resource_reference_counts.entry(reference).or_insert(0) += 1;
        }
    }

    fn finish(self) -> ErbStats {
        ErbStats {
            schema_version: "content-audit-erb-stats/v1".to_string(),
            erb_files: self.erb_files,
            erb_header_files: self.erb_header_files,
            bytes: self.bytes,
            body_read: true,
            token_stats_collected: true,
            encoding_counts: self.encoding_counts,
            decode_error_files: self.decode_error_files,
            lines: self.lines,
            blank_lines: self.blank_lines,
            comment_lines: self.comment_lines,
            token_counts: self.token_counts,
            resource_references: self
                .resource_reference_counts
                .into_iter()
                .map(|(file_name, occurrences)| ReferenceSummary {
                    resolved: self
                        .resolved_resource_names
                        .contains(&file_name.to_lowercase()),
                    file_name,
                    occurrences,
                })
                .collect(),
        }
    }
}

impl CsvStatsBuilder {
    fn add(&mut self, relative: &Path, record: &FileRecord, analysis: CsvFileAnalysis) {
        *self
            .encoding_counts
            .entry(analysis.encoding.clone())
            .or_insert(0) += 1;
        if analysis.decode_errors {
            self.decode_error_files.push(record.relative_path.clone());
        }
        self.totals.rows += analysis.rows;
        self.totals.blank_rows += analysis.blank_rows;
        self.totals.duplicate_first_column_values += analysis.duplicate_first_column_values;
        self.totals.parse_errors += analysis.parse_errors;
        self.files.push(CsvFileStats {
            relative_path: record.relative_path.clone(),
            encoding: analysis.encoding,
            decode_errors: analysis.decode_errors,
            delimiter: analysis.delimiter,
            rows: analysis.rows,
            min_columns: analysis.min_columns,
            max_columns: analysis.max_columns,
            blank_rows: analysis.blank_rows,
            duplicate_first_column_values: analysis.duplicate_first_column_values,
            parse_errors: analysis.parse_errors,
            special_kind: special_csv_kind(relative).map(str::to_string),
        });
    }

    fn finish(self) -> CsvStats {
        CsvStats {
            schema_version: "content-audit-csv-stats/v1".to_string(),
            csv_files: self.csv_files,
            csv_directory_files: self.csv_directory_files,
            bytes: self.bytes,
            special_files: self.special_files.into_iter().collect(),
            body_read: true,
            row_column_stats_collected: true,
            encoding_counts: self.encoding_counts,
            decode_error_files: self.decode_error_files,
            totals: self.totals,
            files: self.files,
        }
    }
}

impl ResourcesStatsBuilder {
    fn finish(self) -> ResourcesStats {
        let duplicate_hash_groups = duplicate_groups(&self.files, |file| file.sha256.to_string());
        let duplicate_file_names =
            duplicate_named_groups(&self.files, |file| file.file_name.to_lowercase());
        let same_stem_different_extensions =
            duplicate_named_groups(&self.files, |file| file.stem.to_lowercase())
                .into_iter()
                .filter(|group| {
                    let extensions: BTreeSet<_> = group
                        .paths
                        .iter()
                        .filter_map(|path| {
                            Path::new(path)
                                .extension()
                                .map(|value| value.to_string_lossy().to_lowercase())
                        })
                        .collect();
                    extensions.len() > 1
                })
                .collect();
        let unknown_author_files = self
            .files
            .iter()
            .filter(|file| file.author == "unknown")
            .count();
        let unknown_license_files = self
            .files
            .iter()
            .filter(|file| file.license == "unknown")
            .count();
        ResourcesStats {
            schema_version: "content-audit-resources/v1".to_string(),
            resource_directory_files: self.resource_directory_files,
            resource_directory_non_asset_files: self.resource_directory_non_asset_files,
            image_files: self.image_files,
            audio_files: self.audio_files,
            font_files: self.font_files,
            bytes: self.bytes,
            body_read: true,
            hash_collected: true,
            files: self.files,
            duplicate_hash_groups,
            duplicate_file_names,
            same_stem_different_extensions,
            unresolved_reference_candidates: self.unresolved_reference_candidates,
            unknown_author_files: unknown_author_files as u64,
            unknown_license_files: unknown_license_files as u64,
        }
    }
}

fn add_count(counts: &mut BTreeMap<String, u64>, key: &str, value: u64) {
    *counts.entry(key.to_string()).or_insert(0) += value;
}

fn duplicate_groups(
    files: &[ResourceFileStats],
    key_fn: impl Fn(&ResourceFileStats) -> String,
) -> Vec<Vec<String>> {
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for file in files {
        groups
            .entry(key_fn(file))
            .or_default()
            .push(file.relative_path.clone());
    }
    groups
        .into_values()
        .filter(|paths| paths.len() > 1)
        .collect()
}

fn duplicate_named_groups(
    files: &[ResourceFileStats],
    key_fn: impl Fn(&ResourceFileStats) -> String,
) -> Vec<DuplicateNameGroup> {
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for file in files {
        groups
            .entry(key_fn(file))
            .or_default()
            .push(file.relative_path.clone());
    }
    groups
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(key, paths)| DuplicateNameGroup { key, paths })
        .collect()
}

fn classify_file(relative: &Path, extension: &str) -> &'static str {
    let file_name = relative
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    if file_name.contains("cache") || extension == ".cache" {
        return "runtime_state";
    }

    match extension {
        ".erb" => "erb",
        ".erh" => "erb_header",
        ".csv" => "csv",
        ".webp" | ".png" | ".jpg" | ".jpeg" => "resource_image",
        ".mp3" | ".mid" | ".wav" | ".ogg" | ".flac" => "resource_audio",
        ".ttf" | ".ttc" | ".otf" => "font",
        ".txt" | ".md" | ".pdf" | ".docx" | ".xlsx" | ".xls" => "document",
        ".config" | ".cfg" | ".xml" | ".json" | ".toml" => "config",
        ".py" | ".bat" | ".ps1" => "tool_script",
        ".exe" | ".dll" => "runtime_binary",
        ".sav" | ".log" | ".dat" => "runtime_state",
        ".zip" | ".7z" | ".rar" => "archive",
        _ => "unknown",
    }
}

fn is_special_csv(relative: &Path) -> bool {
    special_csv_kind(relative).is_some()
}

fn special_csv_kind(relative: &Path) -> Option<&'static str> {
    let file_name = relative
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    match file_name.as_str() {
        "_rename.csv" => Some("rename"),
        "_replace.csv" => Some("replace"),
        "variablesize.csv" => Some("variable_size"),
        _ => None,
    }
}

fn top_component(path: &Path) -> String {
    path.components()
        .next()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .unwrap_or_default()
}

fn normalized_extension(path: &Path) -> String {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|ext| format!(".{}", ext.to_ascii_lowercase()))
        .unwrap_or_default()
}

fn path_depth(path: &Path) -> usize {
    path.components()
        .filter(|component| matches!(component, Component::Normal(_)))
        .count()
}

fn normalize_path(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    if let Some(stripped) = text.strip_prefix("//?/") {
        stripped.to_string()
    } else {
        text
    }
}

fn now_rfc3339() -> String {
    format_system_time(SystemTime::now())
}

fn format_system_time(system_time: SystemTime) -> String {
    let datetime: OffsetDateTime = system_time.into();
    datetime
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(windows)]
fn is_reparse_point_or_symlink(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point_or_symlink(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn write_reports(out_dir: &Path, report: &AuditReport) -> AuditResult<()> {
    fs::create_dir_all(out_dir)?;
    write_json(out_dir.join("summary.json"), &report.summary)?;
    write_jsonl(out_dir.join("files.jsonl"), &report.files)?;
    write_json(out_dir.join("directories.json"), &report.directories)?;
    write_json(out_dir.join("extensions.json"), &report.summary.extensions)?;
    write_json(out_dir.join("risks.json"), &report.summary.risks)?;
    write_json(out_dir.join("erb-stats.json"), &report.erb_stats)?;
    write_json(out_dir.join("csv-stats.json"), &report.csv_stats)?;
    write_json(out_dir.join("resources.json"), &report.resources)?;
    write_markdown(out_dir.join("summary.md"), report)?;
    Ok(())
}

fn write_json(path: PathBuf, value: &impl Serialize) -> AuditResult<()> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(BufWriter::new(file), value)?;
    Ok(())
}

fn write_jsonl(path: PathBuf, records: &[FileRecord]) -> AuditResult<()> {
    let mut writer = BufWriter::new(File::create(path)?);
    for record in records {
        serde_json::to_writer(&mut writer, record)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

fn write_markdown(path: PathBuf, report: &AuditReport) -> AuditResult<()> {
    let mut writer = BufWriter::new(File::create(path)?);
    writeln!(writer, "# ERAtw Content Audit Summary")?;
    writeln!(writer)?;
    writeln!(writer, "- Schema: `{}`", report.summary.schema_version)?;
    writeln!(writer, "- Source: `{}`", report.summary.source_root)?;
    writeln!(writer, "- Generated: `{}`", report.summary.generated_at)?;
    writeln!(
        writer,
        "- Source body emitted: `false` (ERB/CSV aggregate statistics and resource hashes collected)"
    )?;
    writeln!(writer)?;
    writeln!(writer, "## Totals")?;
    writeln!(writer)?;
    writeln!(writer, "| Metric | Value |")?;
    writeln!(writer, "| --- | ---: |")?;
    writeln!(writer, "| Files | {} |", report.summary.totals.files)?;
    writeln!(
        writer,
        "| Directories | {} |",
        report.summary.totals.directories
    )?;
    writeln!(writer, "| Bytes | {} |", report.summary.totals.bytes)?;
    writeln!(
        writer,
        "| Excluded directories | {} |",
        report.summary.totals.excluded_directories
    )?;
    writeln!(
        writer,
        "| Reparse points skipped | {} |",
        report.summary.totals.reparse_points_skipped
    )?;
    writeln!(writer)?;
    writeln!(writer, "## Top Extensions")?;
    writeln!(writer)?;
    writeln!(writer, "| Extension | Files | Bytes |")?;
    writeln!(writer, "| --- | ---: | ---: |")?;
    for extension in report.summary.extensions.iter().take(20) {
        writeln!(
            writer,
            "| `{}` | {} | {} |",
            extension.extension, extension.files, extension.bytes
        )?;
    }
    writeln!(writer)?;
    writeln!(writer, "## Risks")?;
    writeln!(writer)?;
    if report.summary.risks.is_empty() {
        writeln!(writer, "No risks reported.")?;
    } else {
        writeln!(writer, "| Severity | Code | Path |")?;
        writeln!(writer, "| --- | --- | --- |")?;
        for risk in &report.summary.risks {
            writeln!(
                writer,
                "| `{}` | `{}` | `{}` |",
                risk.severity, risk.code, risk.relative_path
            )?;
        }
    }
    Ok(())
}

pub fn default_out_dir() -> PathBuf {
    let compact = now_rfc3339().replace([':', '-'], "").replace('T', "-");
    PathBuf::from("reports").join("content-audit").join(compact)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scans_all_readonly_layers_without_git_or_source_output() {
        let root = temp_dir("scan");
        fs::create_dir_all(root.join("CSV")).unwrap();
        fs::create_dir_all(root.join("ERB")).unwrap();
        fs::create_dir_all(root.join("resources")).unwrap();
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join("CSV").join("Talent.csv"), b"legacy fixture").unwrap();
        fs::write(root.join("ERB").join("main.ERB"), b"legacy fixture").unwrap();
        fs::write(root.join("resources").join("001_face.webp"), b"webp").unwrap();
        fs::write(root.join("tool.py"), b"print('not executed')").unwrap();
        fs::write(root.join(".git").join("config"), b"ignored").unwrap();

        let out_root = temp_dir("output");
        let out = out_root.join("reports");
        let options =
            AuditOptions::new(root.clone(), out.clone()).with_allowed_source(root.clone());
        let report = run_audit(&options).unwrap();

        assert_eq!(report.summary.totals.files, 4);
        assert_eq!(report.summary.totals.excluded_directories, 1);
        assert_eq!(report.erb_stats.erb_files, 1);
        assert_eq!(report.csv_stats.csv_files, 1);
        assert_eq!(report.resources.image_files, 1);
        assert!(report.erb_stats.token_stats_collected);
        assert!(report.csv_stats.row_column_stats_collected);
        assert!(report.resources.hash_collected);
        assert!(report
            .summary
            .risks
            .iter()
            .any(|risk| risk.code == "TOOL_SCRIPT_PRESENT"));
        assert!(out.join("summary.json").exists());
        assert!(out.join("files.jsonl").exists());

        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(out_root).unwrap();
    }

    #[test]
    fn rejects_source_outside_allowlist() {
        let allowed = temp_dir("allowed");
        let source = temp_dir("source");
        let out = source.join("out");
        let options = AuditOptions::new(source.clone(), out).with_allowed_source(allowed.clone());
        let err = run_audit(&options).unwrap_err();
        assert!(matches!(err, AuditError::UnsafeSource(_)));
        fs::remove_dir_all(allowed).unwrap();
        fs::remove_dir_all(source).unwrap();
    }

    #[test]
    fn rejects_output_inside_readonly_source() {
        let root = temp_dir("readonly-output");
        let options =
            AuditOptions::new(root.clone(), root.join("reports")).with_allowed_source(root.clone());
        let err = run_audit(&options).unwrap_err();
        assert!(matches!(err, AuditError::UnsafeOutput(_)));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_existing_output_directory() {
        let root = temp_dir("existing-output-source");
        let out = temp_dir("existing-output");
        let options =
            AuditOptions::new(root.clone(), out.clone()).with_allowed_source(root.clone());
        let err = run_audit(&options).unwrap_err();
        assert!(matches!(err, AuditError::UnsafeOutput(_)));
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(out).unwrap();
    }

    #[test]
    fn classifies_known_extensions() {
        assert_eq!(classify_file(Path::new("ERB/main.erb"), ".erb"), "erb");
        assert_eq!(
            classify_file(Path::new("resources/001.webp"), ".webp"),
            "resource_image"
        );
        assert_eq!(classify_file(Path::new("run.bat"), ".bat"), "tool_script");
        assert_eq!(
            classify_file(Path::new("save.sav"), ".sav"),
            "runtime_state"
        );
        assert_eq!(classify_file(Path::new("unknown"), ""), "unknown");
    }

    fn temp_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "eratw_next_content_audit_{label}_{}_{}",
            std::process::id(),
            now_rfc3339().replace([':', '-', 'T', 'Z'], "")
        ));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        fs::create_dir_all(&path).unwrap();
        path
    }
}
