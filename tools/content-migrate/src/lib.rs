//! M2 draft content package generator and validator.
//!
//! The generator consumes M1 audit reports only. It never reads legacy ERB/CSV
//! bodies and never copies binary assets into the draft package.

use eratw_next_content_audit::{
    AuditSummary, FileRecord, ResourceFileStats as AuditResourceFileStats, ResourcesStats,
    SUMMARY_SCHEMA_VERSION,
};
use jsonschema::{Draft, JSONSchema};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub const PACKAGE_SCHEMA_VERSION: &str = "content-package/v1-draft";
pub const MIGRATION_REPORT_SCHEMA_VERSION: &str = "migration-report/v1-draft";
pub const VALIDATION_REPORT_SCHEMA_VERSION: &str = "content-package-validation/v1-draft";
pub const GENERATOR_NAME: &str = "eratw-next-migration-draft";
pub const GENERATOR_VERSION: &str = "0.2.0";

#[derive(Debug)]
pub enum MigrationError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidInput(String),
    UnsafeOutput(String),
    Validation(Vec<ValidationIssue>),
}

impl Display for MigrationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Json(err) => write!(f, "JSON error: {err}"),
            Self::InvalidInput(message) => write!(f, "invalid input: {message}"),
            Self::UnsafeOutput(message) => write!(f, "unsafe output: {message}"),
            Self::Validation(issues) => write!(
                f,
                "generated package failed validation with {} error(s)",
                issues.len()
            ),
        }
    }
}

impl Error for MigrationError {}

impl From<std::io::Error> for MigrationError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for MigrationError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

pub type MigrationResult<T> = Result<T, MigrationError>;

#[derive(Debug, Clone)]
pub struct MigrationOptions {
    pub audit_dir: PathBuf,
    pub out_dir: PathBuf,
    pub repo_root: PathBuf,
}

impl MigrationOptions {
    pub fn new(audit_dir: PathBuf, out_dir: PathBuf) -> Self {
        Self {
            audit_dir,
            out_dir,
            repo_root: repo_root(),
        }
    }

    pub fn with_repo_root(mut self, repo_root: PathBuf) -> Self {
        self.repo_root = repo_root;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewState {
    pub status: String,
    pub notes: Vec<String>,
    pub blocking_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionTrace {
    pub tool: String,
    pub version: String,
    pub confidence: String,
    pub requires_review: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceTrace {
    pub source_root_id: String,
    pub relative_path: String,
    pub legacy_id: Option<String>,
    pub line_range: Option<[u64; 2]>,
    pub content_hash: Option<String>,
    pub conversion: ConversionTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageManifest {
    pub schema_version: String,
    pub package_id: String,
    pub display_name: String,
    pub version: String,
    pub engine_version: String,
    pub source: PackageSource,
    pub dependencies: Vec<String>,
    pub conflicts: Vec<String>,
    pub capabilities: Vec<String>,
    pub review: ReviewState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageSource {
    pub kind: String,
    pub source_root_id: String,
    pub source_root_hint: String,
    pub generated_from_audit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryEntry {
    pub id: String,
    pub dictionary_id: String,
    pub legacy_key: String,
    pub display_name: String,
    pub aliases: Vec<String>,
    pub value_type: String,
    pub category: String,
    pub source_trace: SourceTrace,
    pub review: ReviewState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterDraft {
    pub id: String,
    pub legacy: CharacterLegacy,
    pub display_name: LocalizedName,
    pub profile: CharacterProfile,
    pub dictionary_refs: CharacterDictionaryRefs,
    pub resource_refs: Vec<String>,
    pub dialogue_source_refs: Vec<String>,
    pub source_trace: SourceTrace,
    pub review: ReviewState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterLegacy {
    pub numeric_id: u64,
    pub raw_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalizedName {
    pub primary: String,
    pub ja: Option<String>,
    pub zh_hans: Option<String>,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterProfile {
    pub species: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterDictionaryRefs {
    pub talents: Vec<String>,
    pub base_stats: Vec<String>,
    pub abilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDraft {
    pub id: String,
    pub legacy: ResourceLegacy,
    pub media_type: String,
    pub source_path: String,
    pub usage: Vec<String>,
    pub character_refs: Vec<String>,
    pub hash: Option<String>,
    pub metadata: ResourceMetadata,
    pub source_trace: SourceTrace,
    pub review: ReviewState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceLegacy {
    pub numeric_prefix: Option<u64>,
    pub variant_tokens: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetadata {
    pub author: String,
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogueSourceDraft {
    pub id: String,
    pub kind: String,
    pub legacy: DialogueLegacy,
    pub source_path: String,
    pub candidate_speaker_refs: Vec<String>,
    pub candidate_scene_kind: String,
    pub conversion_plan: DialogueConversionPlan,
    pub source_trace: SourceTrace,
    pub review: ReviewState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogueLegacy {
    pub character_numeric_id: Option<u64>,
    pub file_pattern: String,
    pub category_tokens: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogueConversionPlan {
    pub strategy: String,
    pub requires_erb_subset: bool,
    pub requires_manual_review: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFileIndex {
    pub relative_path: String,
    pub kind: String,
    pub size_bytes: u64,
    pub mapped_object_ids: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnmappedItem {
    pub code: String,
    pub source_path: String,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationReport {
    pub schema_version: String,
    pub source_root_id: String,
    pub generated_at: String,
    pub summary: MigrationSummary,
    pub unmapped_items: Vec<UnmappedItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationSummary {
    pub source_files_seen: u64,
    pub objects_generated: u64,
    pub objects_needing_review: u64,
    pub unmapped_items: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub code: String,
    pub severity: String,
    pub object_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub schema_version: String,
    pub valid: bool,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
    pub counts: BTreeMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct GeneratedPackage {
    pub manifest: PackageManifest,
    pub dictionaries: Vec<DictionaryEntry>,
    pub characters: Vec<CharacterDraft>,
    pub resources: Vec<ResourceDraft>,
    pub dialogue_sources: Vec<DialogueSourceDraft>,
    pub source_files: Vec<SourceFileIndex>,
    pub unmapped_items: Vec<UnmappedItem>,
    pub migration_report: MigrationReport,
    pub validation_report: ValidationReport,
}

struct AuditInput {
    summary: AuditSummary,
    files: Vec<FileRecord>,
    resources: ResourcesStats,
}

#[derive(Default)]
struct CharacterCandidate {
    source_paths: BTreeSet<String>,
    content_hashes: BTreeSet<String>,
    resource_refs: BTreeSet<String>,
    dialogue_refs: BTreeSet<String>,
}

pub fn run_migration(options: &MigrationOptions) -> MigrationResult<GeneratedPackage> {
    validate_output_path(
        &options.out_dir,
        &options.repo_root,
        Some(&options.audit_dir),
    )?;
    let input = load_audit(&options.audit_dir)?;
    let mut package = generate_package(&input);
    package.validation_report = validate_package(&package);
    if !package.validation_report.valid {
        return Err(MigrationError::Validation(
            package.validation_report.errors.clone(),
        ));
    }
    write_package(&options.out_dir, &package)?;
    Ok(package)
}

fn load_audit(audit_dir: &Path) -> MigrationResult<AuditInput> {
    let summary: AuditSummary = read_json(audit_dir.join("summary.json"))?;
    if summary.schema_version != SUMMARY_SCHEMA_VERSION {
        return Err(MigrationError::InvalidInput(format!(
            "expected audit schema '{SUMMARY_SCHEMA_VERSION}', got '{}'",
            summary.schema_version
        )));
    }
    let resources: ResourcesStats = read_json(audit_dir.join("resources.json"))?;
    let files = read_jsonl(audit_dir.join("files.jsonl"))?;
    if files.len() as u64 != summary.totals.files {
        return Err(MigrationError::InvalidInput(format!(
            "files.jsonl has {} records but summary reports {}",
            files.len(),
            summary.totals.files
        )));
    }
    Ok(AuditInput {
        summary,
        files,
        resources,
    })
}

fn generate_package(input: &AuditInput) -> GeneratedPackage {
    let generated_at = now_rfc3339();
    let manifest = PackageManifest {
        schema_version: PACKAGE_SCHEMA_VERSION.to_string(),
        package_id: "eratw.core.draft".to_string(),
        display_name: "ERAtw Core Draft".to_string(),
        version: "0.2.0-draft".to_string(),
        engine_version: ">=0.1.0".to_string(),
        source: PackageSource {
            kind: "legacy-eratw".to_string(),
            source_root_id: "eratw-content".to_string(),
            source_root_hint: input.summary.source_root.clone(),
            generated_from_audit: SUMMARY_SCHEMA_VERSION.to_string(),
        },
        dependencies: Vec::new(),
        conflicts: Vec::new(),
        capabilities: vec![
            "draft.source_index".to_string(),
            "draft.dictionary_sources".to_string(),
            "draft.resource_bindings".to_string(),
            "draft.dialogue_sources".to_string(),
        ],
        review: review(
            "generated",
            vec!["Draft package is not playable.".to_string()],
            Vec::new(),
        ),
    };

    let mut source_mappings: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let dictionaries = generate_dictionaries(&input.files, &mut source_mappings);
    let resources = generate_resources(&input.resources.files, &mut source_mappings);
    let dialogue_sources = generate_dialogue_sources(&input.files, &mut source_mappings);
    let characters = generate_characters(&resources, &dialogue_sources, &mut source_mappings);
    let (source_files, unmapped_items) = generate_source_index(&input.files, &source_mappings);
    let objects_generated =
        dictionaries.len() + resources.len() + dialogue_sources.len() + characters.len();
    let objects_needing_review = dictionaries
        .iter()
        .filter(|item| item.review.status == "needs_review")
        .count()
        + resources
            .iter()
            .filter(|item| item.review.status == "needs_review")
            .count()
        + dialogue_sources
            .iter()
            .filter(|item| item.review.status == "needs_review")
            .count()
        + characters
            .iter()
            .filter(|item| item.review.status == "needs_review")
            .count();

    let migration_report = MigrationReport {
        schema_version: MIGRATION_REPORT_SCHEMA_VERSION.to_string(),
        source_root_id: "eratw-content".to_string(),
        generated_at,
        summary: MigrationSummary {
            source_files_seen: input.files.len() as u64,
            objects_generated: objects_generated as u64,
            objects_needing_review: objects_needing_review as u64,
            unmapped_items: unmapped_items.len() as u64,
        },
        unmapped_items: unmapped_items.clone(),
    };

    let mut package = GeneratedPackage {
        manifest,
        dictionaries,
        characters,
        resources,
        dialogue_sources,
        source_files,
        unmapped_items,
        migration_report,
        validation_report: empty_validation_report(),
    };
    package.validation_report = validate_package(&package);
    package
}

fn generate_dictionaries(
    files: &[FileRecord],
    mappings: &mut BTreeMap<String, Vec<String>>,
) -> Vec<DictionaryEntry> {
    let mut entries = Vec::new();
    for file in files.iter().filter(|file| file.kind == "csv") {
        let stem = Path::new(&file.relative_path)
            .file_stem()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_default();
        let Some((dictionary_slug, value_type, category)) = dictionary_mapping(&stem) else {
            continue;
        };
        let path_hash = stable_hash(&file.relative_path);
        let id = format!(
            "core.dictionary.{dictionary_slug}.source.{}",
            &path_hash[..8]
        );
        mappings
            .entry(file.relative_path.clone())
            .or_default()
            .push(id.clone());
        entries.push(DictionaryEntry {
            id,
            dictionary_id: format!("core.dictionary.{dictionary_slug}"),
            legacy_key: stem.clone(),
            display_name: format!("Legacy {} Dictionary Source", stem),
            aliases: Vec::new(),
            value_type: value_type.to_string(),
            category: category.to_string(),
            source_trace: source_trace(&file.relative_path, Some(stem), None, "medium", true),
            review: review(
                "needs_review",
                vec![
                    "M2 indexes the CSV source but does not read or convert row content."
                        .to_string(),
                ],
                Vec::new(),
            ),
        });
    }
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    entries
}

fn generate_resources(
    audit_resources: &[AuditResourceFileStats],
    mappings: &mut BTreeMap<String, Vec<String>>,
) -> Vec<ResourceDraft> {
    let mut resources = Vec::new();
    let mut used_ids = BTreeSet::new();
    for resource in audit_resources {
        let path_hash = stable_hash(&resource.relative_path);
        let base = match resource.numeric_prefix {
            Some(prefix) => format!(
                "core.resource.{prefix:03}.{}.{}",
                slugify(&resource.stem),
                &path_hash[..8]
            ),
            None => format!(
                "core.resource.unbound.{}.{}",
                &resource.sha256[..12.min(resource.sha256.len())],
                &path_hash[..8]
            ),
        };
        let id = unique_id(base, &resource.relative_path, &mut used_ids);
        let character_refs = resource
            .numeric_prefix
            .map(|prefix| vec![character_id(prefix)])
            .unwrap_or_default();
        mappings
            .entry(resource.relative_path.clone())
            .or_default()
            .push(id.clone());
        resources.push(ResourceDraft {
            id,
            legacy: ResourceLegacy {
                numeric_prefix: resource.numeric_prefix,
                variant_tokens: resource.variant_tokens.clone(),
            },
            media_type: media_type(&resource.media_type).to_string(),
            source_path: resource.relative_path.clone(),
            usage: infer_resource_usage(&resource.variant_tokens),
            character_refs,
            hash: Some(format!("sha256:{}", resource.sha256)),
            metadata: ResourceMetadata {
                author: resource.author.clone(),
                license: resource.license.clone(),
            },
            source_trace: source_trace(
                &resource.relative_path,
                resource.numeric_prefix.map(|value| value.to_string()),
                Some(resource.sha256.clone()),
                if resource.numeric_prefix.is_some() {
                    "medium"
                } else {
                    "low"
                },
                true,
            ),
            review: review(
                "needs_review",
                Vec::new(),
                vec!["license_unknown".to_string(), "author_unknown".to_string()],
            ),
        });
    }
    resources.sort_by(|a, b| a.id.cmp(&b.id));
    resources
}

fn generate_dialogue_sources(
    files: &[FileRecord],
    mappings: &mut BTreeMap<String, Vec<String>>,
) -> Vec<DialogueSourceDraft> {
    let mut sources = Vec::new();
    let mut used_ids = BTreeSet::new();
    for file in files.iter().filter(|file| file.kind == "erb") {
        let file_name = Path::new(&file.relative_path)
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_default();
        if !looks_like_dialogue_source(&file.relative_path) {
            continue;
        }
        let numeric_id = extract_character_id(&file.relative_path);
        let hash = stable_hash(&file.relative_path);
        let base = numeric_id
            .map(|value| format!("core.dialogue_source.kojo.{value:03}.{}", &hash[..8]))
            .unwrap_or_else(|| format!("core.dialogue_source.pending.{}", &hash[..12]));
        let id = unique_id(base, &hash, &mut used_ids);
        mappings
            .entry(file.relative_path.clone())
            .or_default()
            .push(id.clone());
        sources.push(DialogueSourceDraft {
            id,
            kind: "legacy_erb_source".to_string(),
            legacy: DialogueLegacy {
                character_numeric_id: numeric_id,
                file_pattern: file_name,
                category_tokens: dialogue_category_tokens(&file.relative_path),
            },
            source_path: file.relative_path.clone(),
            candidate_speaker_refs: numeric_id
                .map(|value| vec![character_id(value)])
                .unwrap_or_default(),
            candidate_scene_kind: infer_scene_kind(&file.relative_path).to_string(),
            conversion_plan: DialogueConversionPlan {
                strategy: "manual_or_subset_erb".to_string(),
                requires_erb_subset: true,
                requires_manual_review: true,
            },
            source_trace: source_trace(
                &file.relative_path,
                numeric_id.map(|value| value.to_string()),
                None,
                "low",
                true,
            ),
            review: review(
                "needs_review",
                vec!["ERB body is indexed but not converted in M2.".to_string()],
                Vec::new(),
            ),
        });
    }
    sources.sort_by(|a, b| a.id.cmp(&b.id));
    sources
}

fn generate_characters(
    resources: &[ResourceDraft],
    dialogue_sources: &[DialogueSourceDraft],
    mappings: &mut BTreeMap<String, Vec<String>>,
) -> Vec<CharacterDraft> {
    let mut candidates: BTreeMap<u64, CharacterCandidate> = BTreeMap::new();
    for resource in resources {
        if let Some(numeric_id) = resource.legacy.numeric_prefix {
            let candidate = candidates.entry(numeric_id).or_default();
            candidate.source_paths.insert(resource.source_path.clone());
            if let Some(hash) = &resource.hash {
                candidate.content_hashes.insert(hash.clone());
            }
            candidate.resource_refs.insert(resource.id.clone());
        }
    }
    for dialogue in dialogue_sources {
        if let Some(numeric_id) = dialogue.legacy.character_numeric_id {
            let candidate = candidates.entry(numeric_id).or_default();
            candidate.source_paths.insert(dialogue.source_path.clone());
            candidate.dialogue_refs.insert(dialogue.id.clone());
        }
    }

    let mut characters = Vec::new();
    for (numeric_id, candidate) in candidates {
        let id = character_id(numeric_id);
        for source_path in &candidate.source_paths {
            mappings
                .entry(source_path.clone())
                .or_default()
                .push(id.clone());
        }
        let source_path = candidate
            .source_paths
            .iter()
            .next()
            .cloned()
            .unwrap_or_else(|| "migration/pending".to_string());
        let content_hash = candidate.content_hashes.iter().next().cloned();
        characters.push(CharacterDraft {
            id,
            legacy: CharacterLegacy {
                numeric_id,
                raw_keys: vec![numeric_id.to_string(), format!("{numeric_id:03}")],
            },
            display_name: LocalizedName {
                primary: format!("Character {numeric_id:03}"),
                ja: None,
                zh_hans: None,
                aliases: Vec::new(),
            },
            profile: CharacterProfile {
                species: None,
                title: None,
                description: None,
            },
            dictionary_refs: CharacterDictionaryRefs {
                talents: Vec::new(),
                base_stats: Vec::new(),
                abilities: Vec::new(),
            },
            resource_refs: candidate.resource_refs.into_iter().collect(),
            dialogue_source_refs: candidate.dialogue_refs.into_iter().collect(),
            source_trace: source_trace(
                &source_path,
                Some(numeric_id.to_string()),
                content_hash,
                "medium",
                true,
            ),
            review: review(
                "needs_review",
                vec!["Display name and profile are placeholders.".to_string()],
                Vec::new(),
            ),
        });
    }
    characters
}

fn generate_source_index(
    files: &[FileRecord],
    mappings: &BTreeMap<String, Vec<String>>,
) -> (Vec<SourceFileIndex>, Vec<UnmappedItem>) {
    let mut source_files = Vec::with_capacity(files.len());
    let mut unmapped_items = Vec::new();
    for file in files {
        let mapped_object_ids = mappings
            .get(&file.relative_path)
            .cloned()
            .unwrap_or_default();
        let status = if mapped_object_ids.is_empty() {
            let code = match file.kind.as_str() {
                "csv" => "CSV_SEMANTICS_UNKNOWN",
                "erb" | "erb_header" => "ERB_SOURCE_NOT_MAPPED",
                "resource_image" | "resource_audio" | "font" => "RESOURCE_NOT_MAPPED",
                _ => "SOURCE_FILE_NOT_MAPPED",
            };
            unmapped_items.push(UnmappedItem {
                code: code.to_string(),
                source_path: file.relative_path.clone(),
                severity: "warning".to_string(),
                message: "Source file was indexed but no M2 draft object was generated."
                    .to_string(),
            });
            "unmapped"
        } else {
            "mapped"
        };
        source_files.push(SourceFileIndex {
            relative_path: file.relative_path.clone(),
            kind: file.kind.clone(),
            size_bytes: file.size_bytes,
            mapped_object_ids,
            status: status.to_string(),
        });
    }
    (source_files, unmapped_items)
}

pub fn validate_package(package: &GeneratedPackage) -> ValidationReport {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let character_ids: BTreeSet<_> = package
        .characters
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    let resource_ids: BTreeSet<_> = package
        .resources
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    let dialogue_ids: BTreeSet<_> = package
        .dialogue_sources
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    let all_object_ids: BTreeSet<_> = package
        .dictionaries
        .iter()
        .map(|item| item.id.as_str())
        .chain(package.characters.iter().map(|item| item.id.as_str()))
        .chain(package.resources.iter().map(|item| item.id.as_str()))
        .chain(package.dialogue_sources.iter().map(|item| item.id.as_str()))
        .collect();
    let source_index: BTreeMap<_, _> = package
        .source_files
        .iter()
        .map(|file| (file.relative_path.as_str(), file))
        .collect();

    validate_schema_object(
        "content-package.schema.json",
        &package.manifest,
        Some(package.manifest.package_id.clone()),
        &mut errors,
    );
    validate_schema_objects(
        "content-dictionary-entry.schema.json",
        &package.dictionaries,
        |item| item.id.clone(),
        &mut errors,
    );
    validate_schema_objects(
        "content-character.schema.json",
        &package.characters,
        |item| item.id.clone(),
        &mut errors,
    );
    validate_schema_objects(
        "content-resource.schema.json",
        &package.resources,
        |item| item.id.clone(),
        &mut errors,
    );
    validate_schema_objects(
        "content-dialogue-source.schema.json",
        &package.dialogue_sources,
        |item| item.id.clone(),
        &mut errors,
    );
    validate_schema_object(
        "migration-report.schema.json",
        &package.migration_report,
        None,
        &mut errors,
    );
    validate_schema_objects(
        "content-source-file.schema.json",
        &package.source_files,
        |item| item.relative_path.clone(),
        &mut errors,
    );
    validate_schema_objects(
        "content-unmapped-item.schema.json",
        &package.unmapped_items,
        |item| item.source_path.clone(),
        &mut errors,
    );

    validate_unique_ids(
        package
            .dictionaries
            .iter()
            .map(|item| item.id.as_str())
            .chain(package.characters.iter().map(|item| item.id.as_str()))
            .chain(package.resources.iter().map(|item| item.id.as_str()))
            .chain(package.dialogue_sources.iter().map(|item| item.id.as_str())),
        &mut errors,
    );

    for character in &package.characters {
        validate_source_trace(
            &character.id,
            &character.source_trace,
            &source_index,
            &mut errors,
        );
        for reference in &character.resource_refs {
            if !resource_ids.contains(reference.as_str()) {
                errors.push(issue(
                    "MISSING_RESOURCE_REFERENCE",
                    "blocker",
                    Some(character.id.clone()),
                    format!("Character references missing resource '{reference}'."),
                ));
            }
        }
        for reference in &character.dialogue_source_refs {
            if !dialogue_ids.contains(reference.as_str()) {
                errors.push(issue(
                    "MISSING_DIALOGUE_REFERENCE",
                    "blocker",
                    Some(character.id.clone()),
                    format!("Character references missing dialogue source '{reference}'."),
                ));
            }
        }
    }
    for resource in &package.resources {
        validate_source_trace(
            &resource.id,
            &resource.source_trace,
            &source_index,
            &mut errors,
        );
        for reference in &resource.character_refs {
            if !character_ids.contains(reference.as_str()) {
                errors.push(issue(
                    "MISSING_CHARACTER_REFERENCE",
                    "blocker",
                    Some(resource.id.clone()),
                    format!("Resource references missing character '{reference}'."),
                ));
            }
        }
        if resource.metadata.license == "unknown" {
            warnings.push(issue(
                "LICENSE_UNKNOWN",
                "warning",
                Some(resource.id.clone()),
                "Resource license requires review.".to_string(),
            ));
        }
        if resource.metadata.author == "unknown" {
            warnings.push(issue(
                "AUTHOR_UNKNOWN",
                "warning",
                Some(resource.id.clone()),
                "Resource author requires review.".to_string(),
            ));
        }
    }
    for dialogue in &package.dialogue_sources {
        validate_source_trace(
            &dialogue.id,
            &dialogue.source_trace,
            &source_index,
            &mut errors,
        );
        for reference in &dialogue.candidate_speaker_refs {
            if !character_ids.contains(reference.as_str()) {
                errors.push(issue(
                    "MISSING_CHARACTER_REFERENCE",
                    "blocker",
                    Some(dialogue.id.clone()),
                    format!("Dialogue source references missing character '{reference}'."),
                ));
            }
        }
    }
    for dictionary in &package.dictionaries {
        validate_source_trace(
            &dictionary.id,
            &dictionary.source_trace,
            &source_index,
            &mut errors,
        );
    }

    let accounted = package.source_files.len();
    if source_index.len() != accounted {
        errors.push(issue(
            "DUPLICATE_SOURCE_INDEX_PATH",
            "blocker",
            None,
            "Source file index contains duplicate relative paths.".to_string(),
        ));
    }
    for source_file in &package.source_files {
        let expected_status = if source_file.mapped_object_ids.is_empty() {
            "unmapped"
        } else {
            "mapped"
        };
        if source_file.status != expected_status {
            errors.push(issue(
                "SOURCE_STATUS_MISMATCH",
                "blocker",
                Some(source_file.relative_path.clone()),
                format!(
                    "Source status '{}' does not match its object mappings.",
                    source_file.status
                ),
            ));
        }
        for object_id in &source_file.mapped_object_ids {
            if !all_object_ids.contains(object_id.as_str()) {
                errors.push(issue(
                    "DANGLING_SOURCE_MAPPING",
                    "blocker",
                    Some(source_file.relative_path.clone()),
                    format!("Source index references missing object '{object_id}'."),
                ));
            }
        }
    }
    for item in &package.unmapped_items {
        match source_index.get(item.source_path.as_str()) {
            None => errors.push(issue(
                "UNMAPPED_SOURCE_MISSING",
                "blocker",
                Some(item.source_path.clone()),
                "Unmapped item does not resolve to the source index.".to_string(),
            )),
            Some(source_file) if source_file.status != "unmapped" => errors.push(issue(
                "UNMAPPED_SOURCE_STATUS_MISMATCH",
                "blocker",
                Some(item.source_path.clone()),
                "Unmapped item points to a source marked as mapped.".to_string(),
            )),
            Some(_) => {}
        }
    }
    if package.migration_report.summary.source_files_seen != accounted as u64 {
        errors.push(issue(
            "SOURCE_COUNT_MISMATCH",
            "blocker",
            None,
            "Migration report source file count does not match source index.".to_string(),
        ));
    }
    if package.migration_report.summary.unmapped_items != package.unmapped_items.len() as u64 {
        errors.push(issue(
            "UNMAPPED_COUNT_MISMATCH",
            "blocker",
            None,
            "Migration report unmapped count does not match unmapped item list.".to_string(),
        ));
    }
    if package.migration_report.unmapped_items.len() != package.unmapped_items.len() {
        errors.push(issue(
            "MIGRATION_REPORT_UNMAPPED_LIST_MISMATCH",
            "blocker",
            None,
            "Migration report embedded unmapped list does not match package output.".to_string(),
        ));
    }
    let objects_generated = package.dictionaries.len()
        + package.characters.len()
        + package.resources.len()
        + package.dialogue_sources.len();
    if package.migration_report.summary.objects_generated != objects_generated as u64 {
        errors.push(issue(
            "OBJECT_COUNT_MISMATCH",
            "blocker",
            None,
            "Migration report object count does not match generated objects.".to_string(),
        ));
    }
    let objects_needing_review = package
        .dictionaries
        .iter()
        .map(|item| item.review.status.as_str())
        .chain(
            package
                .characters
                .iter()
                .map(|item| item.review.status.as_str()),
        )
        .chain(
            package
                .resources
                .iter()
                .map(|item| item.review.status.as_str()),
        )
        .chain(
            package
                .dialogue_sources
                .iter()
                .map(|item| item.review.status.as_str()),
        )
        .filter(|status| matches!(*status, "needs_review" | "blocked"))
        .count();
    if package.migration_report.summary.objects_needing_review != objects_needing_review as u64 {
        errors.push(issue(
            "REVIEW_COUNT_MISMATCH",
            "blocker",
            None,
            "Migration report review count does not match generated objects.".to_string(),
        ));
    }

    let mut counts = BTreeMap::new();
    counts.insert(
        "dictionaries".to_string(),
        package.dictionaries.len() as u64,
    );
    counts.insert("characters".to_string(), package.characters.len() as u64);
    counts.insert("resources".to_string(), package.resources.len() as u64);
    counts.insert(
        "dialogueSources".to_string(),
        package.dialogue_sources.len() as u64,
    );
    counts.insert("sourceFiles".to_string(), package.source_files.len() as u64);
    counts.insert(
        "unmappedItems".to_string(),
        package.unmapped_items.len() as u64,
    );
    let mut report = ValidationReport {
        schema_version: VALIDATION_REPORT_SCHEMA_VERSION.to_string(),
        valid: errors.is_empty(),
        errors,
        warnings,
        counts,
    };
    let mut report_schema_errors = Vec::new();
    validate_schema_object(
        "content-package-validation.schema.json",
        &report,
        None,
        &mut report_schema_errors,
    );
    if !report_schema_errors.is_empty() {
        report.valid = false;
        report.errors.extend(report_schema_errors);
    }
    report
}

fn validate_schema_objects<T: Serialize>(
    schema_file: &str,
    objects: &[T],
    id_fn: impl Fn(&T) -> String,
    errors: &mut Vec<ValidationIssue>,
) {
    for object in objects {
        validate_schema_object(schema_file, object, Some(id_fn(object)), errors);
    }
}

fn validate_schema_object<T: Serialize>(
    schema_file: &str,
    object: &T,
    object_id: Option<String>,
    errors: &mut Vec<ValidationIssue>,
) {
    let schema_path = repo_root().join("schemas").join(schema_file);
    let schema: serde_json::Value = match read_json(schema_path) {
        Ok(schema) => schema,
        Err(err) => {
            errors.push(issue(
                "SCHEMA_LOAD_FAILED",
                "blocker",
                object_id,
                format!("Could not load schema '{schema_file}': {err}"),
            ));
            return;
        }
    };
    let compiled = match JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema)
    {
        Ok(compiled) => compiled,
        Err(err) => {
            errors.push(issue(
                "SCHEMA_COMPILE_FAILED",
                "blocker",
                object_id,
                format!("Could not compile schema '{schema_file}': {err}"),
            ));
            return;
        }
    };
    let instance = match serde_json::to_value(object) {
        Ok(instance) => instance,
        Err(err) => {
            errors.push(issue(
                "SCHEMA_SERIALIZATION_FAILED",
                "blocker",
                object_id,
                format!("Could not serialize object for '{schema_file}': {err}"),
            ));
            return;
        }
    };
    if let Err(validation_errors) = compiled.validate(&instance) {
        for error in validation_errors {
            errors.push(issue(
                "SCHEMA_VALIDATION_FAILED",
                "blocker",
                object_id.clone(),
                format!(
                    "Object failed '{schema_file}' at '{}'.",
                    error.instance_path
                ),
            ));
        }
    };
}

fn validate_unique_ids<'a>(ids: impl Iterator<Item = &'a str>, errors: &mut Vec<ValidationIssue>) {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            errors.push(issue(
                "DUPLICATE_ID",
                "blocker",
                Some(id.to_string()),
                "Generated object ID is not unique.".to_string(),
            ));
        }
    }
}

fn validate_source_trace(
    object_id: &str,
    source_trace: &SourceTrace,
    source_index: &BTreeMap<&str, &SourceFileIndex>,
    errors: &mut Vec<ValidationIssue>,
) {
    if source_trace.source_root_id != "eratw-content"
        || source_trace.relative_path.trim().is_empty()
        || source_trace.conversion.tool.trim().is_empty()
    {
        errors.push(issue(
            "INVALID_SOURCE_TRACE",
            "blocker",
            Some(object_id.to_string()),
            "Generated object lacks a complete source trace.".to_string(),
        ));
        return;
    }
    match source_index.get(source_trace.relative_path.as_str()) {
        None => errors.push(issue(
            "MISSING_SOURCE_TRACE_PATH",
            "blocker",
            Some(object_id.to_string()),
            format!(
                "Source trace path '{}' is absent from the source index.",
                source_trace.relative_path
            ),
        )),
        Some(source_file)
            if !source_file
                .mapped_object_ids
                .iter()
                .any(|mapped_id| mapped_id == object_id) =>
        {
            errors.push(issue(
                "SOURCE_TRACE_MAPPING_MISSING",
                "blocker",
                Some(object_id.to_string()),
                "Source trace path does not map back to the generated object.".to_string(),
            ));
        }
        Some(_) => {}
    }
}

fn write_package(out_dir: &Path, package: &GeneratedPackage) -> MigrationResult<()> {
    fs::create_dir_all(out_dir)?;
    for directory in [
        "dictionaries",
        "characters",
        "locations",
        "resources",
        "dialogue",
        "migration",
    ] {
        fs::create_dir_all(out_dir.join(directory))?;
    }
    write_json(out_dir.join("manifest.json"), &package.manifest)?;
    write_jsonl(
        out_dir.join("dictionaries/legacy-csv-dictionaries.jsonl"),
        &package.dictionaries,
    )?;
    write_jsonl(
        out_dir.join("characters/characters.jsonl"),
        &package.characters,
    )?;
    write_empty_jsonl(out_dir.join("locations/locations.jsonl"))?;
    write_jsonl(
        out_dir.join("resources/resources.jsonl"),
        &package.resources,
    )?;
    write_jsonl(
        out_dir.join("dialogue/dialogue-sources.jsonl"),
        &package.dialogue_sources,
    )?;
    write_empty_jsonl(out_dir.join("dialogue/dialogue-scenes.jsonl"))?;
    write_jsonl(
        out_dir.join("migration/source-files.jsonl"),
        &package.source_files,
    )?;
    write_jsonl(
        out_dir.join("migration/unmapped-items.jsonl"),
        &package.unmapped_items,
    )?;
    write_json(
        out_dir.join("migration/migration-report.json"),
        &package.migration_report,
    )?;
    write_json(
        out_dir.join("migration/validation-report.json"),
        &package.validation_report,
    )?;
    Ok(())
}

fn validate_output_path(
    out_dir: &Path,
    repo_root: &Path,
    audit_dir: Option<&Path>,
) -> MigrationResult<()> {
    if !out_dir.is_absolute() {
        return Err(MigrationError::UnsafeOutput(
            "--out must be an absolute path outside the engine repository".to_string(),
        ));
    }
    if looks_like_unc_path(out_dir) {
        return Err(MigrationError::UnsafeOutput(
            "UNC/network output paths are not allowed".to_string(),
        ));
    }
    if out_dir
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(MigrationError::UnsafeOutput(
            "--out must not contain parent-directory traversal".to_string(),
        ));
    }
    if out_dir.exists() {
        return Err(MigrationError::UnsafeOutput(
            "--out must not already exist".to_string(),
        ));
    }

    let resolved_out = resolve_new_path(out_dir)?;
    let resolved_repo = repo_root.canonicalize().map_err(|error| {
        MigrationError::UnsafeOutput(format!("engine repository does not resolve: {error}"))
    })?;
    if path_is_within(&resolved_out, &resolved_repo) {
        return Err(MigrationError::UnsafeOutput(
            "draft output must not be written inside ERAtw-NEXT".to_string(),
        ));
    }
    if let Some(audit_dir) = audit_dir {
        let resolved_audit = audit_dir.canonicalize().map_err(|error| {
            MigrationError::InvalidInput(format!("audit directory does not resolve: {error}"))
        })?;
        if path_is_within(&resolved_out, &resolved_audit) {
            return Err(MigrationError::UnsafeOutput(
                "draft output must not be written inside the M1 audit input".to_string(),
            ));
        }
    }
    Ok(())
}

fn resolve_new_path(path: &Path) -> MigrationResult<PathBuf> {
    let mut ancestor = path;
    let mut suffix = Vec::new();
    while !ancestor.exists() {
        let name = ancestor.file_name().ok_or_else(|| {
            MigrationError::UnsafeOutput("output path has no existing ancestor".to_string())
        })?;
        suffix.push(name.to_os_string());
        ancestor = ancestor.parent().ok_or_else(|| {
            MigrationError::UnsafeOutput("output path has no existing ancestor".to_string())
        })?;
    }
    let mut resolved = ancestor.canonicalize()?;
    for component in suffix.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn path_is_within(path: &Path, parent: &Path) -> bool {
    let path = comparable_path(path);
    let parent = comparable_path(parent);
    path == parent || path.starts_with(&(parent + "/"))
}

fn looks_like_unc_path(path: &Path) -> bool {
    let text = path.as_os_str().to_string_lossy();
    text.starts_with(r"\\") || text.starts_with("//")
}

fn dictionary_mapping(stem: &str) -> Option<(&'static str, &'static str, &'static str)> {
    match stem.to_ascii_lowercase().as_str() {
        "abl" => Some(("ability", "number", "character_ability")),
        "base" => Some(("base", "number", "character_base")),
        "cflag" => Some(("cflag", "flag", "character_flag")),
        "cstr" => Some(("cstr", "string", "character_string")),
        "equip" => Some(("equip", "enum", "equipment")),
        "exp" => Some(("experience", "number", "character_experience")),
        "flag" => Some(("flag", "flag", "global_flag")),
        "item" => Some(("item", "enum", "item")),
        "palam" => Some(("parameter", "number", "character_parameter")),
        "str" => Some(("string", "string", "global_string")),
        "talent" => Some(("talent", "flag", "character_trait")),
        "tcvar" => Some(("tcvar", "unknown", "temporary_character_variable")),
        "tequip" => Some(("tequip", "enum", "temporary_equipment")),
        "tflag" => Some(("tflag", "flag", "temporary_flag")),
        "train" => Some(("train", "enum", "training_command")),
        "variablesize" => Some(("variable_size", "number", "runtime_configuration")),
        _ => None,
    }
}

fn media_type(kind: &str) -> &'static str {
    match kind {
        "resource_image" => "image",
        "resource_audio" => "audio",
        "font" => "font",
        _ => "unknown",
    }
}

fn infer_resource_usage(tokens: &[String]) -> Vec<String> {
    let mut usage = BTreeSet::new();
    for token in tokens {
        let lower = token.to_lowercase();
        if lower.contains('顔') || lower.contains("face") {
            usage.insert("portrait".to_string());
        }
        if lower.contains("立ち") || lower.contains("stand") {
            usage.insert("standing_sprite".to_string());
        }
    }
    usage.into_iter().collect()
}

fn looks_like_dialogue_source(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("kojo") || lower.contains("口上") || character_id_regex().is_match(&lower)
}

fn extract_character_id(path: &str) -> Option<u64> {
    character_id_regex()
        .captures(path)
        .and_then(|captures| captures.get(1))
        .and_then(|value| value.as_str().parse().ok())
}

fn character_id_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?i)(?:^|[^a-z0-9])k(?:ojo)?[_-]?(\d{1,4})(?:[^0-9]|$)")
            .expect("valid character id regex")
    })
}

fn dialogue_category_tokens(path: &str) -> Vec<String> {
    let lower = path.to_lowercase();
    let mut tokens = Vec::new();
    for (needle, token) in [
        ("daily", "daily"),
        ("日常", "daily"),
        ("event", "event"),
        ("イベント", "event"),
        ("train", "training"),
        ("調教", "training"),
        ("com", "command"),
    ] {
        if lower.contains(needle) && !tokens.contains(&token.to_string()) {
            tokens.push(token.to_string());
        }
    }
    if tokens.is_empty() {
        tokens.push("unknown".to_string());
    }
    tokens
}

fn infer_scene_kind(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    if lower.contains("daily") || lower.contains("日常") {
        "daily"
    } else if lower.contains("event") || lower.contains("イベント") {
        "event"
    } else if lower.contains("train") || lower.contains("調教") {
        "training"
    } else {
        "unknown"
    }
}

fn character_id(numeric_id: u64) -> String {
    format!("core.character.{numeric_id:03}")
}

fn source_trace(
    relative_path: &str,
    legacy_id: Option<String>,
    content_hash: Option<String>,
    confidence: &str,
    requires_review: bool,
) -> SourceTrace {
    SourceTrace {
        source_root_id: "eratw-content".to_string(),
        relative_path: relative_path.to_string(),
        legacy_id,
        line_range: None,
        content_hash,
        conversion: ConversionTrace {
            tool: GENERATOR_NAME.to_string(),
            version: GENERATOR_VERSION.to_string(),
            confidence: confidence.to_string(),
            requires_review,
        },
    }
}

fn review(status: &str, notes: Vec<String>, blocking_issues: Vec<String>) -> ReviewState {
    ReviewState {
        status: status.to_string(),
        notes,
        blocking_issues,
    }
}

fn unique_id(base: String, salt: &str, used: &mut BTreeSet<String>) -> String {
    if used.insert(base.clone()) {
        return base;
    }
    let suffix = stable_hash(salt);
    for attempt in 1_u64.. {
        let candidate = format!("{base}.{}.{attempt}", &suffix[..8]);
        if used.insert(candidate.clone()) {
            return candidate;
        }
    }
    unreachable!("u64 ID collision attempts exhausted")
}

fn slugify(value: &str) -> String {
    let mut output = String::new();
    let mut previous_separator = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
            previous_separator = false;
        } else if !previous_separator {
            output.push('_');
            previous_separator = true;
        }
    }
    output.trim_matches('_').to_string()
}

fn stable_hash(value: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn issue(
    code: &str,
    severity: &str,
    object_id: Option<String>,
    message: String,
) -> ValidationIssue {
    ValidationIssue {
        code: code.to_string(),
        severity: severity.to_string(),
        object_id,
        message,
    }
}

fn empty_validation_report() -> ValidationReport {
    ValidationReport {
        schema_version: VALIDATION_REPORT_SCHEMA_VERSION.to_string(),
        valid: false,
        errors: Vec::new(),
        warnings: Vec::new(),
        counts: BTreeMap::new(),
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("content-migrate must live under tools/")
        .to_path_buf()
}

fn comparable_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_lowercase()
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: PathBuf) -> MigrationResult<T> {
    Ok(serde_json::from_reader(BufReader::new(File::open(path)?))?)
}

fn read_jsonl<T: for<'de> Deserialize<'de>>(path: PathBuf) -> MigrationResult<Vec<T>> {
    let reader = BufReader::new(File::open(path)?);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if !line.trim().is_empty() {
            records.push(serde_json::from_str(&line)?);
        }
    }
    Ok(records)
}

fn write_json(path: PathBuf, value: &impl Serialize) -> MigrationResult<()> {
    serde_json::to_writer_pretty(BufWriter::new(File::create(path)?), value)?;
    Ok(())
}

fn write_jsonl<T: Serialize>(path: PathBuf, records: &[T]) -> MigrationResult<()> {
    let mut writer = BufWriter::new(File::create(path)?);
    for record in records {
        serde_json::to_writer(&mut writer, record)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

fn write_empty_jsonl(path: PathBuf) -> MigrationResult<()> {
    File::create(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use eratw_next_content_audit::{run_audit, AuditOptions};

    #[test]
    fn generates_and_validates_draft_package_from_audit_reports() {
        let root = temp_dir("source");
        fs::create_dir_all(root.join("CSV/sub")).unwrap();
        fs::create_dir_all(root.join("ERB")).unwrap();
        fs::create_dir_all(root.join("resources/sub")).unwrap();
        fs::write(root.join("CSV/Talent.csv"), b"1,Talent").unwrap();
        fs::write(root.join("CSV/CFLAG.csv"), b"1,Flag").unwrap();
        fs::write(root.join("CSV/sub/CFLAG.csv"), b"1,Flag").unwrap();
        fs::write(
            root.join("ERB/M_KOJO_K1_DAILY.ERB"),
            b"@TEST\nPRINTFORM 001_face.webp\n",
        )
        .unwrap();
        fs::write(root.join("resources/001_face.webp"), b"image fixture").unwrap();
        fs::write(root.join("resources/021_face.webp"), b"duplicate image").unwrap();
        fs::write(root.join("resources/sub/021_face.webp"), b"duplicate image").unwrap();

        let audit_root = temp_dir("audit");
        let audit_dir = audit_root.join("report");
        run_audit(
            &AuditOptions::new(root.clone(), audit_dir.clone()).with_allowed_source(root.clone()),
        )
        .unwrap();
        let out_dir = temp_dir("output").join("content-package");
        let fake_repo = temp_dir("repo");
        let package = run_migration(
            &MigrationOptions::new(audit_dir.clone(), out_dir.clone())
                .with_repo_root(fake_repo.clone()),
        )
        .unwrap();

        assert!(package.validation_report.valid);
        assert_eq!(package.dictionaries.len(), 3);
        assert_eq!(package.resources.len(), 3);
        assert_eq!(package.characters.len(), 2);
        assert_eq!(package.dialogue_sources.len(), 1);
        let object_ids: Vec<_> = package
            .dictionaries
            .iter()
            .map(|item| item.id.as_str())
            .chain(package.resources.iter().map(|item| item.id.as_str()))
            .collect();
        assert_eq!(
            object_ids.iter().copied().collect::<BTreeSet<_>>().len(),
            object_ids.len()
        );
        assert!(out_dir.join("manifest.json").exists());
        assert!(out_dir.join("migration/validation-report.json").exists());

        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(audit_root).unwrap();
        fs::remove_dir_all(out_dir.parent().unwrap()).unwrap();
        fs::remove_dir_all(fake_repo).unwrap();
    }

    #[test]
    fn rejects_output_inside_engine_repository() {
        let repo = temp_dir("repo-guard");
        let options = MigrationOptions::new(repo.join("audit"), repo.join("generated"))
            .with_repo_root(repo.clone());
        let err = run_migration(&options).unwrap_err();
        assert!(matches!(err, MigrationError::UnsafeOutput(_)));
        fs::remove_dir_all(repo).unwrap();
    }

    #[test]
    fn rejects_existing_output_directory() {
        let repo = temp_dir("existing-output-repo");
        let audit = temp_dir("existing-output-audit");
        let out = temp_dir("existing-output-target");
        let options =
            MigrationOptions::new(audit.clone(), out.clone()).with_repo_root(repo.clone());
        let err = run_migration(&options).unwrap_err();
        assert!(matches!(err, MigrationError::UnsafeOutput(_)));
        fs::remove_dir_all(repo).unwrap();
        fs::remove_dir_all(audit).unwrap();
        fs::remove_dir_all(out).unwrap();
    }

    #[test]
    fn rejects_output_inside_audit_input() {
        let repo = temp_dir("audit-output-repo");
        let audit = temp_dir("audit-output-input");
        let options = MigrationOptions::new(audit.clone(), audit.join("content-package"))
            .with_repo_root(repo.clone());
        let err = run_migration(&options).unwrap_err();
        assert!(matches!(err, MigrationError::UnsafeOutput(_)));
        fs::remove_dir_all(repo).unwrap();
        fs::remove_dir_all(audit).unwrap();
    }

    #[test]
    fn validator_rejects_missing_cross_reference() {
        let mut package = empty_package();
        package.resources.push(ResourceDraft {
            id: "core.resource.001.face".to_string(),
            legacy: ResourceLegacy {
                numeric_prefix: Some(1),
                variant_tokens: vec!["face".to_string()],
            },
            media_type: "image".to_string(),
            source_path: "resources/001_face.webp".to_string(),
            usage: vec!["portrait".to_string()],
            character_refs: vec!["core.character.001".to_string()],
            hash: None,
            metadata: ResourceMetadata {
                author: "unknown".to_string(),
                license: "unknown".to_string(),
            },
            source_trace: source_trace(
                "resources/001_face.webp",
                Some("1".to_string()),
                None,
                "low",
                true,
            ),
            review: review("needs_review", Vec::new(), Vec::new()),
        });
        let report = validate_package(&package);
        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|issue| issue.code == "MISSING_CHARACTER_REFERENCE"));
        assert!(report
            .errors
            .iter()
            .any(|issue| issue.code == "MISSING_SOURCE_TRACE_PATH"));
    }

    fn empty_package() -> GeneratedPackage {
        GeneratedPackage {
            manifest: PackageManifest {
                schema_version: PACKAGE_SCHEMA_VERSION.to_string(),
                package_id: "eratw.test.draft".to_string(),
                display_name: "Test".to_string(),
                version: "0.0.0".to_string(),
                engine_version: ">=0.1.0".to_string(),
                source: PackageSource {
                    kind: "legacy-eratw".to_string(),
                    source_root_id: "eratw-content".to_string(),
                    source_root_hint: "test".to_string(),
                    generated_from_audit: SUMMARY_SCHEMA_VERSION.to_string(),
                },
                dependencies: Vec::new(),
                conflicts: Vec::new(),
                capabilities: Vec::new(),
                review: review("generated", Vec::new(), Vec::new()),
            },
            dictionaries: Vec::new(),
            characters: Vec::new(),
            resources: Vec::new(),
            dialogue_sources: Vec::new(),
            source_files: Vec::new(),
            unmapped_items: Vec::new(),
            migration_report: MigrationReport {
                schema_version: MIGRATION_REPORT_SCHEMA_VERSION.to_string(),
                source_root_id: "eratw-content".to_string(),
                generated_at: now_rfc3339(),
                summary: MigrationSummary {
                    source_files_seen: 0,
                    objects_generated: 0,
                    objects_needing_review: 0,
                    unmapped_items: 0,
                },
                unmapped_items: Vec::new(),
            },
            validation_report: empty_validation_report(),
        }
    }

    fn temp_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "eratw_next_migrate_{label}_{}_{}",
            std::process::id(),
            stable_hash(&now_rfc3339())
        ));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        fs::create_dir_all(&path).unwrap();
        path
    }
}
