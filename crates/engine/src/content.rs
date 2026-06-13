use crate::EngineError;
use jsonschema::{Draft, JSONSchema};
use semver::{Version, VersionReq};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Component, Path, PathBuf};

pub const CONTENT_PACKAGE_INDEX_SCHEMA_VERSION: &str = "content-package-index/v1";
pub const CONTENT_PACKAGE_SCHEMA_VERSION: &str = "content-package/v1-draft";
const CONTENT_API_VERSION: &str = "0.4.0";
const MAX_PACKAGE_FILE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_JSONL_LINE_BYTES: usize = 4 * 1024 * 1024;

const MANIFEST_SCHEMA: &str = include_str!("../../../schemas/content-package.schema.json");
const DICTIONARY_SCHEMA: &str =
    include_str!("../../../schemas/content-dictionary-entry.schema.json");
const CHARACTER_SCHEMA: &str = include_str!("../../../schemas/content-character.schema.json");
const LOCATION_SCHEMA: &str = include_str!("../../../schemas/content-location.schema.json");
const RESOURCE_SCHEMA: &str = include_str!("../../../schemas/content-resource.schema.json");
const DIALOGUE_SOURCE_SCHEMA: &str =
    include_str!("../../../schemas/content-dialogue-source.schema.json");
const DIALOGUE_SCENE_SCHEMA: &str =
    include_str!("../../../schemas/content-dialogue-scene.schema.json");
#[cfg(test)]
const CONTENT_PACKAGE_INDEX_SCHEMA: &str =
    include_str!("../../../schemas/content-package-index.schema.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewState {
    pub status: String,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub blocking_issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentPackageManifest {
    pub schema_version: String,
    pub package_id: String,
    pub display_name: String,
    pub version: String,
    pub engine_version: String,
    pub dependencies: Vec<String>,
    pub conflicts: Vec<String>,
    pub capabilities: Vec<String>,
    pub review: ReviewState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PackageIdentity {
    pub package_id: String,
    pub display_name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageCounts {
    pub dictionaries: u64,
    pub characters: u64,
    pub locations: u64,
    pub resources: u64,
    pub dialogue_sources: u64,
    pub dialogue_scenes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterIndexEntry {
    pub id: String,
    pub display_name: String,
    pub review_status: String,
    pub resource_count: u64,
    pub dialogue_source_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationIndexEntry {
    pub id: String,
    pub display_name: String,
    pub kind: String,
    pub tags: Vec<String>,
    pub connections: Vec<String>,
    pub review_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceIndexEntry {
    pub id: String,
    pub media_type: String,
    pub source_path: String,
    pub usage: Vec<String>,
    pub author: String,
    pub license: String,
    pub review_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentPackageIndex {
    pub schema_version: String,
    pub root_path: String,
    pub package: PackageIdentity,
    pub engine_requirement: String,
    pub capabilities: Vec<String>,
    pub review_status: String,
    pub playable: bool,
    pub counts: PackageCounts,
    pub characters: Vec<CharacterIndexEntry>,
    pub locations: Vec<LocationIndexEntry>,
    pub resources: Vec<ResourceIndexEntry>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LoadedContentPackage {
    pub root: PathBuf,
    pub manifest: ContentPackageManifest,
    pub index: ContentPackageIndex,
    pub runtime_locations: Vec<RuntimeLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLocation {
    pub id: String,
    pub connections: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DictionaryRecord {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CharacterRecord {
    id: String,
    display_name: LocalizedName,
    dictionary_refs: CharacterDictionaryRefs,
    resource_refs: Vec<String>,
    dialogue_source_refs: Vec<String>,
    review: ReviewState,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CharacterDictionaryRefs {
    talents: Vec<String>,
    base_stats: Vec<String>,
    abilities: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalizedName {
    primary: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocationRecord {
    id: String,
    display_name: LocalizedName,
    kind: String,
    tags: Vec<String>,
    connections: Vec<String>,
    review: ReviewState,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResourceRecord {
    id: String,
    media_type: String,
    source_path: String,
    usage: Vec<String>,
    character_refs: Vec<String>,
    metadata: ResourceMetadata,
    review: ReviewState,
}

#[derive(Debug, Deserialize)]
struct ResourceMetadata {
    author: String,
    license: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DialogueSourceRecord {
    id: String,
    candidate_speaker_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DialogueSceneRecord {
    id: String,
    entry_node_id: String,
    speaker_refs: Vec<String>,
    resource_refs: Vec<String>,
    nodes: Vec<DialogueSceneNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DialogueSceneNode {
    id: String,
    speaker_ref: Option<String>,
}

pub fn load_content_package(root: impl AsRef<Path>) -> Result<LoadedContentPackage, EngineError> {
    load_content_package_with_dependencies(root.as_ref(), &BTreeSet::new())
}

pub(crate) fn load_content_package_with_dependencies(
    root: &Path,
    available_dependencies: &BTreeSet<String>,
) -> Result<LoadedContentPackage, EngineError> {
    let root = validate_package_root(root)?;

    let manifest_schema = compile_schema("content-package.schema.json", MANIFEST_SCHEMA)?;
    let dictionary_schema =
        compile_schema("content-dictionary-entry.schema.json", DICTIONARY_SCHEMA)?;
    let character_schema = compile_schema("content-character.schema.json", CHARACTER_SCHEMA)?;
    let location_schema = compile_schema("content-location.schema.json", LOCATION_SCHEMA)?;
    let resource_schema = compile_schema("content-resource.schema.json", RESOURCE_SCHEMA)?;
    let dialogue_schema = compile_schema(
        "content-dialogue-source.schema.json",
        DIALOGUE_SOURCE_SCHEMA,
    )?;
    let dialogue_scene_schema =
        compile_schema("content-dialogue-scene.schema.json", DIALOGUE_SCENE_SCHEMA)?;

    let manifest: ContentPackageManifest = read_json(&root, "manifest.json", &manifest_schema)?;
    validate_manifest(&manifest, available_dependencies)?;

    let dictionaries: Vec<DictionaryRecord> = read_jsonl(
        &root,
        "dictionaries/legacy-csv-dictionaries.jsonl",
        &dictionary_schema,
    )?;
    let characters: Vec<CharacterRecord> =
        read_jsonl(&root, "characters/characters.jsonl", &character_schema)?;
    let locations: Vec<LocationRecord> =
        read_jsonl(&root, "locations/locations.jsonl", &location_schema)?;
    let resources: Vec<ResourceRecord> =
        read_jsonl(&root, "resources/resources.jsonl", &resource_schema)?;
    let dialogue_sources: Vec<DialogueSourceRecord> =
        read_jsonl(&root, "dialogue/dialogue-sources.jsonl", &dialogue_schema)?;
    let dialogue_scenes: Vec<DialogueSceneRecord> = read_jsonl(
        &root,
        "dialogue/dialogue-scenes.jsonl",
        &dialogue_scene_schema,
    )?;

    validate_references(
        &dictionaries,
        &characters,
        &locations,
        &resources,
        &dialogue_sources,
        &dialogue_scenes,
    )?;

    let playable = manifest
        .capabilities
        .iter()
        .any(|capability| capability == "playable.core")
        && manifest.review.status == "accepted"
        && !locations.is_empty();
    let mut warnings = Vec::new();
    if !playable {
        warnings.push(
            "Package is indexable but not playable; it needs accepted review, playable.core, and at least one location."
                .to_string(),
        );
    }
    if resources
        .iter()
        .any(|resource| resource.metadata.author == "unknown")
    {
        warnings.push("One or more resource authors are unknown.".to_string());
    }
    if resources
        .iter()
        .any(|resource| resource.metadata.license == "unknown")
    {
        warnings.push("One or more resource licenses are unknown.".to_string());
    }

    let index = ContentPackageIndex {
        schema_version: CONTENT_PACKAGE_INDEX_SCHEMA_VERSION.to_string(),
        root_path: normalize_path(&root),
        package: PackageIdentity {
            package_id: manifest.package_id.clone(),
            display_name: manifest.display_name.clone(),
            version: manifest.version.clone(),
        },
        engine_requirement: manifest.engine_version.clone(),
        capabilities: manifest.capabilities.clone(),
        review_status: manifest.review.status.clone(),
        playable,
        counts: PackageCounts {
            dictionaries: dictionaries.len() as u64,
            characters: characters.len() as u64,
            locations: locations.len() as u64,
            resources: resources.len() as u64,
            dialogue_sources: dialogue_sources.len() as u64,
            dialogue_scenes: dialogue_scenes.len() as u64,
        },
        characters: characters
            .iter()
            .map(|character| CharacterIndexEntry {
                id: character.id.clone(),
                display_name: character.display_name.primary.clone(),
                review_status: character.review.status.clone(),
                resource_count: character.resource_refs.len() as u64,
                dialogue_source_count: character.dialogue_source_refs.len() as u64,
            })
            .collect(),
        locations: locations
            .iter()
            .map(|location| LocationIndexEntry {
                id: location.id.clone(),
                display_name: location.display_name.primary.clone(),
                kind: location.kind.clone(),
                tags: location.tags.clone(),
                connections: location.connections.clone(),
                review_status: location.review.status.clone(),
            })
            .collect(),
        resources: resources
            .iter()
            .map(|resource| ResourceIndexEntry {
                id: resource.id.clone(),
                media_type: resource.media_type.clone(),
                source_path: resource.source_path.clone(),
                usage: resource.usage.clone(),
                author: resource.metadata.author.clone(),
                license: resource.metadata.license.clone(),
                review_status: resource.review.status.clone(),
            })
            .collect(),
        warnings,
    };

    let runtime_locations = locations
        .into_iter()
        .map(|location| RuntimeLocation {
            id: location.id,
            connections: location.connections.into_iter().collect(),
        })
        .collect();

    Ok(LoadedContentPackage {
        root,
        manifest,
        index,
        runtime_locations,
    })
}

fn validate_package_root(root: &Path) -> Result<PathBuf, EngineError> {
    if !root.is_absolute() {
        return Err(content_error(
            "CONTENT_PATH_NOT_ABSOLUTE",
            "Content package path must be absolute.",
            json!({ "path": normalize_path(root) }),
        ));
    }
    if root
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(content_error(
            "CONTENT_PATH_TRAVERSAL",
            "Content package path must not contain parent traversal.",
            json!({ "path": normalize_path(root) }),
        ));
    }
    if looks_like_unc_path(root) {
        return Err(content_error(
            "CONTENT_PATH_NETWORK",
            "Network content package paths are not allowed.",
            json!({ "path": normalize_path(root) }),
        ));
    }
    let root_metadata = fs::symlink_metadata(root).map_err(|error| {
        content_error(
            "CONTENT_ROOT_UNAVAILABLE",
            "Content package directory cannot be inspected.",
            json!({ "path": normalize_path(root), "error": error.to_string() }),
        )
    })?;
    if is_reparse_point_or_symlink(&root_metadata) {
        return Err(content_error(
            "CONTENT_ROOT_REPARSE_POINT",
            "Content package root must not be a symlink or reparse point.",
            json!({ "path": normalize_path(root) }),
        ));
    }
    let canonical = root.canonicalize().map_err(|error| {
        content_error(
            "CONTENT_ROOT_UNAVAILABLE",
            "Content package directory cannot be resolved.",
            json!({ "path": normalize_path(root), "error": error.to_string() }),
        )
    })?;
    if !canonical.is_dir() {
        return Err(content_error(
            "CONTENT_ROOT_NOT_DIRECTORY",
            "Content package root must be a directory.",
            json!({ "path": normalize_path(&canonical) }),
        ));
    }
    Ok(canonical)
}

fn validate_manifest(
    manifest: &ContentPackageManifest,
    available_dependencies: &BTreeSet<String>,
) -> Result<(), EngineError> {
    if manifest.schema_version != CONTENT_PACKAGE_SCHEMA_VERSION {
        return Err(content_error(
            "CONTENT_SCHEMA_UNSUPPORTED",
            "Content package schema version is not supported.",
            json!({
                "expected": CONTENT_PACKAGE_SCHEMA_VERSION,
                "actual": manifest.schema_version
            }),
        ));
    }
    let requirement = VersionReq::parse(&manifest.engine_version).map_err(|error| {
        content_error(
            "CONTENT_ENGINE_REQUIREMENT_INVALID",
            "Content package engine requirement is invalid.",
            json!({ "requirement": manifest.engine_version, "error": error.to_string() }),
        )
    })?;
    let engine_version = Version::parse(CONTENT_API_VERSION).expect("content API version is valid");
    if !requirement.matches(&engine_version) {
        return Err(content_error(
            "CONTENT_ENGINE_VERSION_MISMATCH",
            "Content package is not compatible with this engine.",
            json!({
                "requirement": manifest.engine_version,
                "engineVersion": CONTENT_API_VERSION
            }),
        ));
    }
    let missing: Vec<_> = manifest
        .dependencies
        .iter()
        .filter(|dependency| !available_dependencies.contains(*dependency))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(content_error(
            "CONTENT_DEPENDENCY_MISSING",
            "Content package dependencies are not loaded.",
            json!({ "missing": missing }),
        ));
    }
    Ok(())
}

fn validate_references(
    dictionaries: &[DictionaryRecord],
    characters: &[CharacterRecord],
    locations: &[LocationRecord],
    resources: &[ResourceRecord],
    dialogue_sources: &[DialogueSourceRecord],
    dialogue_scenes: &[DialogueSceneRecord],
) -> Result<(), EngineError> {
    let dictionary_ids =
        collect_unique_ids("dictionary", dictionaries.iter().map(|item| &item.id))?;
    let character_ids = collect_unique_ids("character", characters.iter().map(|item| &item.id))?;
    let location_ids = collect_unique_ids("location", locations.iter().map(|item| &item.id))?;
    let resource_ids = collect_unique_ids("resource", resources.iter().map(|item| &item.id))?;
    let dialogue_ids = collect_unique_ids(
        "dialogue source",
        dialogue_sources.iter().map(|item| &item.id),
    )?;
    collect_unique_ids(
        "dialogue scene",
        dialogue_scenes.iter().map(|item| &item.id),
    )?;

    let mut issues = Vec::new();
    for character in characters {
        for reference in character
            .dictionary_refs
            .talents
            .iter()
            .chain(&character.dictionary_refs.base_stats)
            .chain(&character.dictionary_refs.abilities)
        {
            if !dictionary_ids.contains(reference) {
                issues.push(json!({
                    "code": "MISSING_DICTIONARY_REFERENCE",
                    "objectId": character.id,
                    "reference": reference
                }));
            }
        }
        collect_missing(
            &mut issues,
            "MISSING_RESOURCE_REFERENCE",
            &character.id,
            &character.resource_refs,
            &resource_ids,
        );
        collect_missing(
            &mut issues,
            "MISSING_DIALOGUE_REFERENCE",
            &character.id,
            &character.dialogue_source_refs,
            &dialogue_ids,
        );
    }
    for resource in resources {
        if !is_safe_relative_path(&resource.source_path) {
            issues.push(json!({
                "code": "UNSAFE_RESOURCE_PATH",
                "objectId": resource.id,
                "path": resource.source_path
            }));
        }
        collect_missing(
            &mut issues,
            "MISSING_CHARACTER_REFERENCE",
            &resource.id,
            &resource.character_refs,
            &character_ids,
        );
    }
    for dialogue in dialogue_sources {
        collect_missing(
            &mut issues,
            "MISSING_CHARACTER_REFERENCE",
            &dialogue.id,
            &dialogue.candidate_speaker_refs,
            &character_ids,
        );
    }
    for scene in dialogue_scenes {
        collect_missing(
            &mut issues,
            "MISSING_CHARACTER_REFERENCE",
            &scene.id,
            &scene.speaker_refs,
            &character_ids,
        );
        collect_missing(
            &mut issues,
            "MISSING_RESOURCE_REFERENCE",
            &scene.id,
            &scene.resource_refs,
            &resource_ids,
        );

        let node_ids = collect_unique_ids(
            "dialogue scene node",
            scene.nodes.iter().map(|node| &node.id),
        )?;
        if !node_ids.contains(&scene.entry_node_id) {
            issues.push(json!({
                "code": "MISSING_DIALOGUE_ENTRY_NODE",
                "objectId": scene.id,
                "reference": scene.entry_node_id
            }));
        }
        for node in &scene.nodes {
            if let Some(speaker_ref) = &node.speaker_ref {
                collect_missing(
                    &mut issues,
                    "MISSING_CHARACTER_REFERENCE",
                    &format!("{}#{}", scene.id, node.id),
                    std::slice::from_ref(speaker_ref),
                    &character_ids,
                );
            }
        }
    }
    for location in locations {
        collect_missing(
            &mut issues,
            "MISSING_LOCATION_REFERENCE",
            &location.id,
            &location.connections,
            &location_ids,
        );
    }
    if !issues.is_empty() {
        return Err(content_error(
            "CONTENT_REFERENCE_INVALID",
            "Content package contains invalid references.",
            json!({ "issues": issues }),
        ));
    }
    Ok(())
}

fn collect_unique_ids<'a>(
    kind: &str,
    ids: impl Iterator<Item = &'a String>,
) -> Result<BTreeSet<String>, EngineError> {
    let mut unique = BTreeSet::new();
    for id in ids {
        if !unique.insert(id.clone()) {
            return Err(content_error(
                "CONTENT_DUPLICATE_ID",
                "Content package contains a duplicate object ID.",
                json!({ "kind": kind, "id": id }),
            ));
        }
    }
    Ok(unique)
}

fn collect_missing(
    issues: &mut Vec<Value>,
    code: &str,
    object_id: &str,
    references: &[String],
    available: &BTreeSet<String>,
) {
    for reference in references {
        if !available.contains(reference) {
            issues.push(json!({
                "code": code,
                "objectId": object_id,
                "reference": reference
            }));
        }
    }
}

fn is_safe_relative_path(value: &str) -> bool {
    let normalized = value.replace('\\', "/");
    !normalized.is_empty()
        && !normalized.starts_with('/')
        && !normalized.starts_with("//")
        && !normalized.contains(':')
        && !normalized
            .split('/')
            .any(|component| component == ".." || component.is_empty())
}

fn looks_like_unc_path(path: &Path) -> bool {
    let value = path.as_os_str().to_string_lossy();
    value.starts_with(r"\\") || value.starts_with("//")
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

fn compile_schema(name: &str, source: &str) -> Result<JSONSchema, EngineError> {
    let schema: Value = serde_json::from_str(source).map_err(|error| {
        content_error(
            "CONTENT_INTERNAL_SCHEMA_INVALID",
            "Embedded content schema is invalid JSON.",
            json!({ "schema": name, "error": error.to_string() }),
        )
    })?;
    JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema)
        .map_err(|error| {
            content_error(
                "CONTENT_INTERNAL_SCHEMA_INVALID",
                "Embedded content schema could not be compiled.",
                json!({ "schema": name, "error": error.to_string() }),
            )
        })
}

fn read_json<T: DeserializeOwned>(
    root: &Path,
    relative: &str,
    schema: &JSONSchema,
) -> Result<T, EngineError> {
    let path = resolve_package_file(root, relative)?;
    let value: Value =
        serde_json::from_reader(BufReader::new(File::open(&path).map_err(|error| {
            content_error(
                "CONTENT_FILE_UNREADABLE",
                "Content package file cannot be opened.",
                json!({ "file": relative, "error": error.to_string() }),
            )
        })?))
        .map_err(|error| {
            content_error(
                "CONTENT_JSON_INVALID",
                "Content package JSON is invalid.",
                json!({ "file": relative, "error": error.to_string() }),
            )
        })?;
    validate_value(relative, &value, schema)?;
    serde_json::from_value(value).map_err(|error| {
        content_error(
            "CONTENT_MODEL_INVALID",
            "Content package object cannot be decoded.",
            json!({ "file": relative, "error": error.to_string() }),
        )
    })
}

fn read_jsonl<T: DeserializeOwned>(
    root: &Path,
    relative: &str,
    schema: &JSONSchema,
) -> Result<Vec<T>, EngineError> {
    let path = resolve_package_file(root, relative)?;
    let reader = BufReader::new(File::open(&path).map_err(|error| {
        content_error(
            "CONTENT_FILE_UNREADABLE",
            "Content package file cannot be opened.",
            json!({ "file": relative, "error": error.to_string() }),
        )
    })?);
    let mut records = Vec::new();
    for (line_index, line) in reader.lines().enumerate() {
        let line = line.map_err(|error| {
            content_error(
                "CONTENT_FILE_UNREADABLE",
                "Content package JSONL line cannot be read.",
                json!({ "file": relative, "line": line_index + 1, "error": error.to_string() }),
            )
        })?;
        if line.len() > MAX_JSONL_LINE_BYTES {
            return Err(content_error(
                "CONTENT_LINE_TOO_LARGE",
                "Content package JSONL line exceeds the size limit.",
                json!({ "file": relative, "line": line_index + 1 }),
            ));
        }
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line).map_err(|error| {
            content_error(
                "CONTENT_JSON_INVALID",
                "Content package JSONL record is invalid.",
                json!({ "file": relative, "line": line_index + 1, "error": error.to_string() }),
            )
        })?;
        validate_value(&format!("{relative}:{}", line_index + 1), &value, schema)?;
        records.push(serde_json::from_value(value).map_err(|error| {
            content_error(
                "CONTENT_MODEL_INVALID",
                "Content package record cannot be decoded.",
                json!({ "file": relative, "line": line_index + 1, "error": error.to_string() }),
            )
        })?);
    }
    Ok(records)
}

fn resolve_package_file(root: &Path, relative: &str) -> Result<PathBuf, EngineError> {
    let candidate = root.join(relative);
    let metadata = fs::symlink_metadata(&candidate).map_err(|error| {
        content_error(
            "CONTENT_FILE_MISSING",
            "Required content package file is missing.",
            json!({ "file": relative, "error": error.to_string() }),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(content_error(
            "CONTENT_FILE_UNSAFE",
            "Content package files must be regular files.",
            json!({ "file": relative }),
        ));
    }
    if metadata.len() > MAX_PACKAGE_FILE_BYTES {
        return Err(content_error(
            "CONTENT_FILE_TOO_LARGE",
            "Content package file exceeds the size limit.",
            json!({ "file": relative, "bytes": metadata.len() }),
        ));
    }
    let canonical = candidate.canonicalize().map_err(|error| {
        content_error(
            "CONTENT_FILE_UNREADABLE",
            "Content package file cannot be resolved.",
            json!({ "file": relative, "error": error.to_string() }),
        )
    })?;
    if !canonical.starts_with(root) {
        return Err(content_error(
            "CONTENT_FILE_ESCAPE",
            "Content package file resolves outside the package root.",
            json!({ "file": relative }),
        ));
    }
    Ok(canonical)
}

fn validate_value(label: &str, value: &Value, schema: &JSONSchema) -> Result<(), EngineError> {
    if let Err(errors) = schema.validate(value) {
        let issues: Vec<_> = errors
            .take(20)
            .map(|error| {
                json!({
                    "instancePath": error.instance_path.to_string(),
                    "message": error.to_string()
                })
            })
            .collect();
        return Err(content_error(
            "CONTENT_SCHEMA_INVALID",
            "Content package object failed schema validation.",
            json!({ "record": label, "issues": issues }),
        ));
    }
    Ok(())
}

fn content_error(code: &str, message: &str, details: Value) -> EngineError {
    EngineError::new(code, message, details)
}

fn normalize_path(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    normalized
        .strip_prefix("//?/")
        .unwrap_or(&normalized)
        .to_string()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::io::Write;

    pub(crate) fn create_playable_package(label: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("eratw_next_content_{label}_{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        for directory in [
            "dictionaries",
            "characters",
            "locations",
            "resources",
            "dialogue",
        ] {
            fs::create_dir_all(root.join(directory)).unwrap();
        }
        fs::write(
            root.join("manifest.json"),
            r#"{
  "schemaVersion":"content-package/v1-draft",
  "packageId":"test.playable",
  "displayName":"Playable Test",
  "version":"1.0.0",
  "engineVersion":">=0.4.0",
  "source":{"kind":"legacy-eratw","sourceRootId":"eratw-content","sourceRootHint":"test","generatedFromAudit":"content-audit-summary/v1"},
  "dependencies":[],
  "conflicts":[],
  "capabilities":["playable.core"],
  "review":{"status":"accepted","notes":[],"blockingIssues":[]}
}"#,
        )
        .unwrap();
        fs::write(
            root.join("characters/characters.jsonl"),
            r#"{"id":"core.character.001","legacy":{"numericId":1,"rawKeys":["1"]},"displayName":{"primary":"Alice","ja":null,"zhHans":null,"aliases":[]},"profile":{"species":null,"title":null,"description":null},"dictionaryRefs":{"talents":[],"baseStats":[],"abilities":[]},"resourceRefs":[],"dialogueSourceRefs":[],"sourceTrace":{"sourceRootId":"eratw-content","relativePath":"test","legacyId":"1","lineRange":null,"contentHash":null,"conversion":{"tool":"test","version":"1","confidence":"high","requiresReview":false}},"review":{"status":"accepted","notes":[],"blockingIssues":[]}}"#,
        )
        .unwrap();
        let mut locations = File::create(root.join("locations/locations.jsonl")).unwrap();
        writeln!(locations, r#"{{"id":"core.location.home","displayName":{{"primary":"Home","zhHans":null,"ja":null}},"kind":"home","tags":["safe"],"connections":["core.location.square"],"sourceTrace":{{}},"review":{{"status":"accepted"}}}}"#).unwrap();
        writeln!(locations, r#"{{"id":"core.location.square","displayName":{{"primary":"Square","zhHans":null,"ja":null}},"kind":"public","tags":[],"connections":["core.location.home"],"sourceTrace":{{}},"review":{{"status":"accepted"}}}}"#).unwrap();
        fs::write(root.join("dictionaries/legacy-csv-dictionaries.jsonl"), "").unwrap();
        fs::write(root.join("resources/resources.jsonl"), "").unwrap();
        fs::write(root.join("dialogue/dialogue-sources.jsonl"), "").unwrap();
        fs::write(root.join("dialogue/dialogue-scenes.jsonl"), "").unwrap();
        root
    }

    #[test]
    fn loads_playable_package_and_builds_indexes() {
        let root = create_playable_package("valid");
        fs::write(
            root.join("dialogue/dialogue-scenes.jsonl"),
            r#"{"id":"core.dialogue.scene.intro","entryNodeId":"start","speakerRefs":["core.character.001"],"resourceRefs":[],"nodes":[{"id":"start","speakerRef":"core.character.001","text":"Hello","choices":[],"effects":[],"conditions":[]}],"sourceTrace":{},"review":{"status":"accepted","notes":[],"blockingIssues":[]}}"#,
        )
        .unwrap();
        let package = load_content_package(&root).unwrap();
        assert!(package.index.playable);
        assert_eq!(package.index.counts.characters, 1);
        assert_eq!(package.index.counts.locations, 2);
        assert_eq!(package.index.counts.dialogue_scenes, 1);
        assert_eq!(package.runtime_locations[0].id, "core.location.home");
        let schema = compile_schema(
            "content-package-index.schema.json",
            CONTENT_PACKAGE_INDEX_SCHEMA,
        )
        .unwrap();
        validate_value(
            "generated content package index",
            &serde_json::to_value(&package.index).unwrap(),
            &schema,
        )
        .unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_invalid_dialogue_scene_references() {
        let root = create_playable_package("bad-dialogue-scene");
        fs::write(
            root.join("dialogue/dialogue-scenes.jsonl"),
            r#"{"id":"core.dialogue.scene.intro","entryNodeId":"missing","speakerRefs":["core.character.missing"],"resourceRefs":["core.resource.missing"],"nodes":[{"id":"start","speakerRef":"core.character.missing","text":"Hello","choices":[],"effects":[],"conditions":[]}],"sourceTrace":{},"review":{"status":"accepted","notes":[],"blockingIssues":[]}}"#,
        )
        .unwrap();
        let error = load_content_package(&root).unwrap_err();
        assert_eq!(error.code, "CONTENT_REFERENCE_INVALID");
        assert_eq!(
            error.details["issues"][0]["code"],
            "MISSING_CHARACTER_REFERENCE"
        );
        assert!(error.details["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["code"] == "MISSING_DIALOGUE_ENTRY_NODE"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_schema_errors_with_stable_code() {
        let root = create_playable_package("bad-schema");
        fs::write(root.join("manifest.json"), "{}").unwrap();
        let error = load_content_package(&root).unwrap_err();
        assert_eq!(error.code, "CONTENT_SCHEMA_INVALID");
        assert!(error.details["issues"].is_array());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_missing_location_reference() {
        let root = create_playable_package("bad-reference");
        fs::write(
            root.join("locations/locations.jsonl"),
            r#"{"id":"core.location.home","displayName":{"primary":"Home","zhHans":null,"ja":null},"kind":"home","tags":[],"connections":["core.location.missing"],"sourceTrace":{},"review":{"status":"accepted"}}"#,
        )
        .unwrap();
        let error = load_content_package(&root).unwrap_err();
        assert_eq!(error.code, "CONTENT_REFERENCE_INVALID");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_missing_dependency_and_future_engine_requirement() {
        let root = create_playable_package("dependency");
        let manifest_path = root.join("manifest.json");
        let mut manifest: Value =
            serde_json::from_reader(File::open(&manifest_path).unwrap()).unwrap();
        manifest["dependencies"] = json!(["other.package"]);
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        let error = load_content_package(&root).unwrap_err();
        assert_eq!(error.code, "CONTENT_DEPENDENCY_MISSING");

        manifest["dependencies"] = json!([]);
        manifest["engineVersion"] = json!(">=99.0.0");
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        let error = load_content_package(&root).unwrap_err();
        assert_eq!(error.code, "CONTENT_ENGINE_VERSION_MISMATCH");
        fs::remove_dir_all(root).unwrap();
    }
}
