//! M1 readonly content metadata audit.
//!
//! This crate only reads filesystem metadata and writes audit reports. It does
//! not execute files, access the network, or read ERB/CSV/content bodies.

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;
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
}

impl Display for AuditError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Json(err) => write!(f, "JSON error: {err}"),
            Self::InvalidArgs(message) => write!(f, "invalid arguments: {message}"),
            Self::UnsafeSource(message) => write!(f, "unsafe source: {message}"),
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

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Default, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryRecord {
    pub relative_path: String,
    pub depth: usize,
    pub file_count: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionSummary {
    pub extension: String,
    pub files: u64,
    pub bytes: u64,
    pub kinds: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskRecord {
    pub code: String,
    pub severity: String,
    pub relative_path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErbStats {
    pub schema_version: String,
    pub erb_files: u64,
    pub erb_header_files: u64,
    pub bytes: u64,
    pub body_read: bool,
    pub token_stats_collected: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvStats {
    pub schema_version: String,
    pub csv_files: u64,
    pub csv_directory_files: u64,
    pub bytes: u64,
    pub special_files: Vec<String>,
    pub body_read: bool,
    pub row_column_stats_collected: bool,
}

#[derive(Debug, Clone, Serialize)]
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
}

#[derive(Default)]
struct CsvStatsBuilder {
    csv_files: u64,
    csv_directory_files: u64,
    bytes: u64,
    special_files: BTreeSet<String>,
}

#[derive(Default)]
struct ResourcesStatsBuilder {
    resource_directory_files: u64,
    resource_directory_non_asset_files: u64,
    image_files: u64,
    audio_files: u64,
    font_files: u64,
    bytes: u64,
}

pub fn run_audit(options: &AuditOptions) -> AuditResult<AuditReport> {
    if options.profile != DEFAULT_PROFILE {
        return Err(AuditError::InvalidArgs(format!(
            "unsupported profile '{}'; expected '{}'",
            options.profile, DEFAULT_PROFILE
        )));
    }

    let source_root = validate_source(&options.source, &options.allowed_source)?;
    let generated_at = now_rfc3339();
    let mut scanner = Scanner::default();
    scanner.scan_dir(&source_root, Path::new(""))?;

    let mut extensions: Vec<_> = scanner.extensions.into_values().collect();
    extensions.sort_by(|a, b| {
        b.files
            .cmp(&a.files)
            .then_with(|| b.bytes.cmp(&a.bytes))
            .then_with(|| a.extension.cmp(&b.extension))
    });

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
        erb_stats: scanner.erb_stats.finish(),
        csv_stats: scanner.csv_stats.finish(),
        resources: scanner.resources.finish(),
        directories: scanner.directories.into_values().collect(),
        files: scanner.files,
        summary,
    };

    write_reports(&options.out_dir, &report)?;
    Ok(report)
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
                self.track_file(&child_relative, &metadata)?;
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

    fn track_file(&mut self, relative: &Path, metadata: &fs::Metadata) -> AuditResult<()> {
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
        self.track_domain_stats(relative, &record);
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

    fn track_domain_stats(&mut self, relative: &Path, record: &FileRecord) {
        match record.kind.as_str() {
            "erb" => {
                self.erb_stats.erb_files += 1;
                self.erb_stats.bytes += record.size_bytes;
            }
            "erb_header" => {
                self.erb_stats.erb_header_files += 1;
                self.erb_stats.bytes += record.size_bytes;
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
            }
            "resource_image" => {
                self.resources.image_files += 1;
                self.resources.bytes += record.size_bytes;
            }
            "resource_audio" => {
                self.resources.audio_files += 1;
                self.resources.bytes += record.size_bytes;
            }
            "font" => {
                self.resources.font_files += 1;
                self.resources.bytes += record.size_bytes;
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
}

impl ErbStatsBuilder {
    fn finish(self) -> ErbStats {
        ErbStats {
            schema_version: "content-audit-erb-stats/v1".to_string(),
            erb_files: self.erb_files,
            erb_header_files: self.erb_header_files,
            bytes: self.bytes,
            body_read: false,
            token_stats_collected: false,
        }
    }
}

impl CsvStatsBuilder {
    fn finish(self) -> CsvStats {
        CsvStats {
            schema_version: "content-audit-csv-stats/v1".to_string(),
            csv_files: self.csv_files,
            csv_directory_files: self.csv_directory_files,
            bytes: self.bytes,
            special_files: self.special_files.into_iter().collect(),
            body_read: false,
            row_column_stats_collected: false,
        }
    }
}

impl ResourcesStatsBuilder {
    fn finish(self) -> ResourcesStats {
        ResourcesStats {
            schema_version: "content-audit-resources/v1".to_string(),
            resource_directory_files: self.resource_directory_files,
            resource_directory_non_asset_files: self.resource_directory_non_asset_files,
            image_files: self.image_files,
            audio_files: self.audio_files,
            font_files: self.font_files,
            bytes: self.bytes,
            body_read: false,
            hash_collected: false,
        }
    }
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
    let file_name = relative
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(
        file_name.as_str(),
        "_rename.csv" | "_replace.csv" | "variablesize.csv"
    )
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
    writeln!(writer, "- Body read: `false`")?;
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
    let compact = now_rfc3339()
        .replace(':', "")
        .replace('-', "")
        .replace('T', "-")
        .replace('Z', "Z");
    PathBuf::from("reports").join("content-audit").join(compact)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scans_metadata_without_git_or_body_stats() {
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

        let out = root.join("out");
        let options =
            AuditOptions::new(root.clone(), out.clone()).with_allowed_source(root.clone());
        let report = run_audit(&options).unwrap();

        assert_eq!(report.summary.totals.files, 4);
        assert_eq!(report.summary.totals.excluded_directories, 1);
        assert_eq!(report.erb_stats.erb_files, 1);
        assert_eq!(report.csv_stats.csv_files, 1);
        assert_eq!(report.resources.image_files, 1);
        assert!(!report.erb_stats.token_stats_collected);
        assert!(report
            .summary
            .risks
            .iter()
            .any(|risk| risk.code == "TOOL_SCRIPT_PRESENT"));
        assert!(out.join("summary.json").exists());
        assert!(out.join("files.jsonl").exists());

        fs::remove_dir_all(root).unwrap();
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
