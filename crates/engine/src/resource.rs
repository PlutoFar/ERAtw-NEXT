use crate::{ResourceAsset, ResourceMediaType};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read},
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourcePlanningOptions {
    #[serde(default)]
    pub low_spec: bool,
}

impl Default for ResourcePlanningOptions {
    fn default() -> Self {
        Self { low_spec: false }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceResolutionReport {
    pub root: String,
    #[serde(default)]
    pub low_spec: bool,
    pub entries: Vec<ResourceResolution>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourcePreflightReport {
    pub root: String,
    #[serde(default)]
    pub low_spec: bool,
    pub ready: bool,
    pub resolution: ResourceResolutionReport,
    pub issues: Vec<ResourcePreflightIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourcePublishReport {
    pub root: String,
    #[serde(default)]
    pub low_spec: bool,
    pub ready: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub resolution: ResourceResolutionReport,
    pub issues: Vec<ResourcePublishIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCacheReport {
    pub root: String,
    #[serde(default)]
    pub low_spec: bool,
    pub ready: bool,
    pub cached_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub resolution: ResourceResolutionReport,
    pub entries: Vec<ResourceCacheEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCacheCleanReport {
    pub root: String,
    #[serde(default)]
    pub low_spec: bool,
    pub ready: bool,
    pub cache_root: String,
    pub kept_count: usize,
    pub removed_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub bytes_removed: u64,
    pub resolution: ResourceResolutionReport,
    pub entries: Vec<ResourceCacheCleanEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCacheEntry {
    pub resource_id: String,
    pub source_path: String,
    pub cache_path: Option<String>,
    pub status: ResourceCacheStatus,
    pub bytes_copied: u64,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceCacheStatus {
    Cached,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCacheCleanEntry {
    pub path: String,
    pub status: ResourceCacheCleanStatus,
    pub bytes_removed: u64,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceCacheCleanStatus {
    Kept,
    Removed,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourcePreflightIssue {
    pub code: ResourcePreflightIssueCode,
    pub resource_id: String,
    pub source_path: String,
    pub message: String,
    pub fallback: ResourceFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourcePublishIssue {
    pub severity: ResourcePublishIssueSeverity,
    pub code: ResourcePublishIssueCode,
    pub resource_id: String,
    pub source_path: String,
    pub message: String,
    pub fallback: ResourceFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourcePublishIssueSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourcePublishIssueCode {
    Missing,
    UnsafePath,
    HashMismatch,
    IoError,
    EmptyLicense,
    UnknownLicense,
    EmptyAuthor,
    UnknownAuthor,
    MissingSha256,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourcePreflightIssueCode {
    Missing,
    UnsafePath,
    HashMismatch,
    IoError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResourceLoadStrategy {
    #[default]
    Eager,
    Deferred,
    ThumbnailOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceResolution {
    pub resource_id: String,
    pub source_path: String,
    pub resolved_path: Option<String>,
    pub media_type: ResourceMediaType,
    pub status: ResourceResolutionStatus,
    #[serde(default)]
    pub load_strategy: ResourceLoadStrategy,
    #[serde(default)]
    pub cache_key: String,
    #[serde(default)]
    pub cache_path: Option<String>,
    #[serde(default)]
    pub thumbnail_path: Option<String>,
    pub fallback: ResourceFallback,
    pub expected_sha256: Option<String>,
    pub actual_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceResolutionStatus {
    Planned,
    Ready,
    Missing,
    UnsafePath,
    HashMismatch,
    IoError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceFallback {
    PlaceholderImage,
    SilentAudio,
    DefaultFont,
    MissingResource,
}

pub fn plan_resource_loads(
    resources: &[ResourceAsset],
    root: impl AsRef<Path>,
) -> ResourceResolutionReport {
    plan_resource_loads_with_options(resources, root, ResourcePlanningOptions::default())
}

pub fn plan_resource_loads_with_options(
    resources: &[ResourceAsset],
    root: impl AsRef<Path>,
    options: ResourcePlanningOptions,
) -> ResourceResolutionReport {
    resolve_resources(resources, root, false, options)
}

pub fn inspect_resource_files(
    resources: &[ResourceAsset],
    root: impl AsRef<Path>,
) -> ResourceResolutionReport {
    inspect_resource_files_with_options(resources, root, ResourcePlanningOptions::default())
}

pub fn inspect_resource_files_with_options(
    resources: &[ResourceAsset],
    root: impl AsRef<Path>,
    options: ResourcePlanningOptions,
) -> ResourceResolutionReport {
    resolve_resources(resources, root, true, options)
}

pub fn preflight_resource_loads(
    resources: &[ResourceAsset],
    root: impl AsRef<Path>,
) -> ResourcePreflightReport {
    preflight_resource_loads_with_options(resources, root, ResourcePlanningOptions::default())
}

pub fn preflight_resource_loads_with_options(
    resources: &[ResourceAsset],
    root: impl AsRef<Path>,
    options: ResourcePlanningOptions,
) -> ResourcePreflightReport {
    let resolution = inspect_resource_files_with_options(resources, root, options);
    let issues = resolution
        .entries
        .iter()
        .filter_map(preflight_issue_from_resolution)
        .collect::<Vec<_>>();

    ResourcePreflightReport {
        root: resolution.root.clone(),
        low_spec: resolution.low_spec,
        ready: issues.is_empty(),
        resolution,
        issues,
    }
}

pub fn audit_resource_publication(
    resources: &[ResourceAsset],
    root: impl AsRef<Path>,
) -> ResourcePublishReport {
    audit_resource_publication_with_options(resources, root, ResourcePlanningOptions::default())
}

pub fn audit_resource_publication_with_options(
    resources: &[ResourceAsset],
    root: impl AsRef<Path>,
    options: ResourcePlanningOptions,
) -> ResourcePublishReport {
    let resolution = inspect_resource_files_with_options(resources, root, options);
    let mut issues = Vec::new();

    for (resource, entry) in resources.iter().zip(&resolution.entries) {
        if let Some(issue) = publish_issue_from_resolution(entry) {
            issues.push(issue);
        }
        collect_publish_metadata_issues(resource, entry, &mut issues);
    }

    let error_count = issues
        .iter()
        .filter(|issue| issue.severity == ResourcePublishIssueSeverity::Error)
        .count();
    let warning_count = issues
        .iter()
        .filter(|issue| issue.severity == ResourcePublishIssueSeverity::Warning)
        .count();

    ResourcePublishReport {
        root: resolution.root.clone(),
        low_spec: resolution.low_spec,
        ready: error_count == 0,
        error_count,
        warning_count,
        resolution,
        issues,
    }
}

pub fn cache_resource_loads(
    resources: &[ResourceAsset],
    root: impl AsRef<Path>,
) -> ResourceCacheReport {
    cache_resource_loads_with_options(resources, root, ResourcePlanningOptions::default())
}

pub fn cache_resource_loads_with_options(
    resources: &[ResourceAsset],
    root: impl AsRef<Path>,
    options: ResourcePlanningOptions,
) -> ResourceCacheReport {
    let root = root.as_ref();
    let resolution = inspect_resource_files_with_options(resources, root, options);
    let entries = resolution
        .entries
        .iter()
        .map(|entry| cache_resource_entry(root, entry))
        .collect::<Vec<_>>();
    let cached_count = entries
        .iter()
        .filter(|entry| entry.status == ResourceCacheStatus::Cached)
        .count();
    let skipped_count = entries
        .iter()
        .filter(|entry| entry.status == ResourceCacheStatus::Skipped)
        .count();
    let failed_count = entries
        .iter()
        .filter(|entry| entry.status == ResourceCacheStatus::Failed)
        .count();

    ResourceCacheReport {
        root: resolution.root.clone(),
        low_spec: resolution.low_spec,
        ready: failed_count == 0,
        cached_count,
        skipped_count,
        failed_count,
        resolution,
        entries,
    }
}

pub fn clean_resource_cache(
    resources: &[ResourceAsset],
    root: impl AsRef<Path>,
) -> ResourceCacheCleanReport {
    clean_resource_cache_with_options(resources, root, ResourcePlanningOptions::default())
}

pub fn clean_resource_cache_with_options(
    resources: &[ResourceAsset],
    root: impl AsRef<Path>,
    options: ResourcePlanningOptions,
) -> ResourceCacheCleanReport {
    let root = root.as_ref();
    let resolution = plan_resource_loads_with_options(resources, root, options);
    let cache_root = root.join(".eratw-cache");
    let resource_targets =
        planned_cache_file_names(&resolution, |entry| entry.cache_path.as_deref());
    let thumbnail_targets =
        planned_cache_file_names(&resolution, |entry| entry.thumbnail_path.as_deref());
    let mut entries = Vec::new();

    clean_resource_cache_dir(
        root,
        &cache_root.join("resources"),
        &resource_targets,
        &mut entries,
    );
    clean_resource_cache_dir(
        root,
        &cache_root.join("thumbnails"),
        &thumbnail_targets,
        &mut entries,
    );
    entries.sort_by(|left, right| left.path.cmp(&right.path));

    let kept_count = entries
        .iter()
        .filter(|entry| entry.status == ResourceCacheCleanStatus::Kept)
        .count();
    let removed_count = entries
        .iter()
        .filter(|entry| entry.status == ResourceCacheCleanStatus::Removed)
        .count();
    let skipped_count = entries
        .iter()
        .filter(|entry| entry.status == ResourceCacheCleanStatus::Skipped)
        .count();
    let failed_count = entries
        .iter()
        .filter(|entry| entry.status == ResourceCacheCleanStatus::Failed)
        .count();
    let bytes_removed = entries
        .iter()
        .filter(|entry| entry.status == ResourceCacheCleanStatus::Removed)
        .map(|entry| entry.bytes_removed)
        .sum();

    ResourceCacheCleanReport {
        root: resolution.root.clone(),
        low_spec: resolution.low_spec,
        ready: failed_count == 0,
        cache_root: cache_root.to_string_lossy().to_string(),
        kept_count,
        removed_count,
        skipped_count,
        failed_count,
        bytes_removed,
        resolution,
        entries,
    }
}

pub fn is_safe_resource_source_path(source_path: &str) -> bool {
    normalize_resource_path(source_path).is_some()
}

pub fn resolve_resource_path(root: impl AsRef<Path>, source_path: &str) -> Option<PathBuf> {
    normalize_resource_path(source_path).map(|relative_path| root.as_ref().join(relative_path))
}

fn resolve_resources(
    resources: &[ResourceAsset],
    root: impl AsRef<Path>,
    inspect_files: bool,
    options: ResourcePlanningOptions,
) -> ResourceResolutionReport {
    let root = root.as_ref();
    let entries = resources
        .iter()
        .map(|resource| resolve_resource(root, resource, inspect_files, &options))
        .collect();

    ResourceResolutionReport {
        root: root.to_string_lossy().to_string(),
        low_spec: options.low_spec,
        entries,
    }
}

fn resolve_resource(
    root: &Path,
    resource: &ResourceAsset,
    inspect_files: bool,
    options: &ResourcePlanningOptions,
) -> ResourceResolution {
    let fallback = fallback_for_media_type(&resource.media_type);
    let load_strategy = load_strategy_for_media_type(&resource.media_type, options.low_spec);
    let cache_key = cache_key_for_resource(resource);
    let Some(path) = resolve_resource_path(root, &resource.source_path) else {
        return ResourceResolution {
            resource_id: resource.resource_id.clone(),
            source_path: resource.source_path.clone(),
            resolved_path: None,
            media_type: resource.media_type.clone(),
            status: ResourceResolutionStatus::UnsafePath,
            load_strategy,
            cache_key,
            cache_path: None,
            thumbnail_path: None,
            fallback,
            expected_sha256: resource.sha256.clone(),
            actual_sha256: None,
        };
    };

    let cache_path = Some(cache_path_for_resource(root, resource, &cache_key));
    let thumbnail_path = thumbnail_path_for_resource(root, &cache_key, &load_strategy);

    if !inspect_files {
        return ResourceResolution {
            resource_id: resource.resource_id.clone(),
            source_path: resource.source_path.clone(),
            resolved_path: Some(path.to_string_lossy().to_string()),
            media_type: resource.media_type.clone(),
            status: ResourceResolutionStatus::Planned,
            load_strategy,
            cache_key,
            cache_path,
            thumbnail_path,
            fallback,
            expected_sha256: resource.sha256.clone(),
            actual_sha256: None,
        };
    }

    let status;
    let actual_sha256;
    if !path.is_file() {
        status = ResourceResolutionStatus::Missing;
        actual_sha256 = None;
    } else {
        match ensure_resolved_path_stays_in_root(root, &path) {
            Ok(true) => match sha256_file(&path) {
                Ok(hash) => {
                    status = if resource
                        .sha256
                        .as_ref()
                        .is_some_and(|expected| !expected.eq_ignore_ascii_case(&hash))
                    {
                        ResourceResolutionStatus::HashMismatch
                    } else {
                        ResourceResolutionStatus::Ready
                    };
                    actual_sha256 = Some(hash);
                }
                Err(_) => {
                    status = ResourceResolutionStatus::IoError;
                    actual_sha256 = None;
                }
            },
            Ok(false) => {
                status = ResourceResolutionStatus::UnsafePath;
                actual_sha256 = None;
            }
            Err(_) => {
                status = ResourceResolutionStatus::IoError;
                actual_sha256 = None;
            }
        }
    }

    ResourceResolution {
        resource_id: resource.resource_id.clone(),
        source_path: resource.source_path.clone(),
        resolved_path: Some(path.to_string_lossy().to_string()),
        media_type: resource.media_type.clone(),
        status,
        load_strategy,
        cache_key,
        cache_path,
        thumbnail_path,
        fallback,
        expected_sha256: resource.sha256.clone(),
        actual_sha256,
    }
}

fn preflight_issue_from_resolution(entry: &ResourceResolution) -> Option<ResourcePreflightIssue> {
    let code = match entry.status {
        ResourceResolutionStatus::Planned | ResourceResolutionStatus::Ready => return None,
        ResourceResolutionStatus::Missing => ResourcePreflightIssueCode::Missing,
        ResourceResolutionStatus::UnsafePath => ResourcePreflightIssueCode::UnsafePath,
        ResourceResolutionStatus::HashMismatch => ResourcePreflightIssueCode::HashMismatch,
        ResourceResolutionStatus::IoError => ResourcePreflightIssueCode::IoError,
    };

    Some(ResourcePreflightIssue {
        code,
        resource_id: entry.resource_id.clone(),
        source_path: entry.source_path.clone(),
        message: preflight_issue_message(entry),
        fallback: entry.fallback.clone(),
    })
}

fn publish_issue_from_resolution(entry: &ResourceResolution) -> Option<ResourcePublishIssue> {
    let code = match entry.status {
        ResourceResolutionStatus::Planned | ResourceResolutionStatus::Ready => return None,
        ResourceResolutionStatus::Missing => ResourcePublishIssueCode::Missing,
        ResourceResolutionStatus::UnsafePath => ResourcePublishIssueCode::UnsafePath,
        ResourceResolutionStatus::HashMismatch => ResourcePublishIssueCode::HashMismatch,
        ResourceResolutionStatus::IoError => ResourcePublishIssueCode::IoError,
    };

    Some(ResourcePublishIssue {
        severity: ResourcePublishIssueSeverity::Error,
        code,
        resource_id: entry.resource_id.clone(),
        source_path: entry.source_path.clone(),
        message: preflight_issue_message(entry),
        fallback: entry.fallback.clone(),
    })
}

fn collect_publish_metadata_issues(
    resource: &ResourceAsset,
    entry: &ResourceResolution,
    issues: &mut Vec<ResourcePublishIssue>,
) {
    let license = resource.license.trim();
    if license.is_empty() {
        issues.push(publish_metadata_issue(
            ResourcePublishIssueSeverity::Error,
            ResourcePublishIssueCode::EmptyLicense,
            resource,
            entry,
            format!("resource license is empty: {}", resource.resource_id),
        ));
    } else if license.eq_ignore_ascii_case("unknown") {
        issues.push(publish_metadata_issue(
            ResourcePublishIssueSeverity::Error,
            ResourcePublishIssueCode::UnknownLicense,
            resource,
            entry,
            format!("resource license is unknown: {}", resource.resource_id),
        ));
    }

    let author = resource.author.trim();
    if author.is_empty() {
        issues.push(publish_metadata_issue(
            ResourcePublishIssueSeverity::Error,
            ResourcePublishIssueCode::EmptyAuthor,
            resource,
            entry,
            format!("resource author is empty: {}", resource.resource_id),
        ));
    } else if author.eq_ignore_ascii_case("unknown") {
        issues.push(publish_metadata_issue(
            ResourcePublishIssueSeverity::Error,
            ResourcePublishIssueCode::UnknownAuthor,
            resource,
            entry,
            format!("resource author is unknown: {}", resource.resource_id),
        ));
    }

    if resource
        .sha256
        .as_ref()
        .map(|sha256| sha256.trim().is_empty())
        .unwrap_or(true)
    {
        issues.push(publish_metadata_issue(
            ResourcePublishIssueSeverity::Warning,
            ResourcePublishIssueCode::MissingSha256,
            resource,
            entry,
            format!("resource sha256 is missing: {}", resource.resource_id),
        ));
    }
}

fn publish_metadata_issue(
    severity: ResourcePublishIssueSeverity,
    code: ResourcePublishIssueCode,
    resource: &ResourceAsset,
    entry: &ResourceResolution,
    message: String,
) -> ResourcePublishIssue {
    ResourcePublishIssue {
        severity,
        code,
        resource_id: resource.resource_id.clone(),
        source_path: resource.source_path.clone(),
        message,
        fallback: entry.fallback.clone(),
    }
}

fn cache_resource_entry(root: &Path, entry: &ResourceResolution) -> ResourceCacheEntry {
    let Some(source_path) = entry.resolved_path.as_deref() else {
        return skipped_cache_entry(entry, "resource path is unsafe");
    };
    let Some(cache_path) = entry.cache_path.as_deref() else {
        return skipped_cache_entry(entry, "resource has no cache target");
    };

    if entry.status != ResourceResolutionStatus::Ready {
        return skipped_cache_entry(
            entry,
            &format!("resource is not ready for caching: {:?}", entry.status),
        );
    }

    let source_path = Path::new(source_path);
    let cache_path = Path::new(cache_path);
    let result = (|| -> io::Result<u64> {
        prepare_cache_file_target(root, cache_path)?;
        fs::copy(source_path, cache_path)
    })();

    match result {
        Ok(bytes_copied) => ResourceCacheEntry {
            resource_id: entry.resource_id.clone(),
            source_path: entry.source_path.clone(),
            cache_path: entry.cache_path.clone(),
            status: ResourceCacheStatus::Cached,
            bytes_copied,
            message: "resource cached".to_string(),
        },
        Err(error) => ResourceCacheEntry {
            resource_id: entry.resource_id.clone(),
            source_path: entry.source_path.clone(),
            cache_path: entry.cache_path.clone(),
            status: ResourceCacheStatus::Failed,
            bytes_copied: 0,
            message: format!("resource cache failed: {error}"),
        },
    }
}

fn planned_cache_file_names(
    resolution: &ResourceResolutionReport,
    path_for_entry: impl Fn(&ResourceResolution) -> Option<&str>,
) -> BTreeSet<String> {
    resolution
        .entries
        .iter()
        .filter_map(path_for_entry)
        .filter_map(|path| Path::new(path).file_name())
        .map(|file_name| file_name.to_string_lossy().to_string())
        .collect()
}

fn clean_resource_cache_dir(
    root: &Path,
    cache_dir: &Path,
    planned_file_names: &BTreeSet<String>,
    entries: &mut Vec<ResourceCacheCleanEntry>,
) {
    let metadata = match fs::symlink_metadata(cache_dir) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return,
        Err(error) => {
            entries.push(failed_clean_entry(
                cache_dir,
                format!("resource cache directory could not be inspected: {error}"),
            ));
            return;
        }
    };

    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        entries.push(failed_clean_entry(
            cache_dir,
            "resource cache path is not a safe directory".to_string(),
        ));
        return;
    }

    if let Err(error) = ensure_cache_path_stays_in_root(root, cache_dir) {
        entries.push(failed_clean_entry(
            cache_dir,
            format!("resource cache directory is outside the content root: {error}"),
        ));
        return;
    }

    let read_dir = match fs::read_dir(cache_dir) {
        Ok(read_dir) => read_dir,
        Err(error) => {
            entries.push(failed_clean_entry(
                cache_dir,
                format!("resource cache directory could not be read: {error}"),
            ));
            return;
        }
    };

    for candidate in read_dir {
        match candidate {
            Ok(candidate) => clean_resource_cache_candidate(
                candidate.path(),
                &candidate.file_name().to_string_lossy(),
                planned_file_names,
                entries,
            ),
            Err(error) => entries.push(failed_clean_entry(
                cache_dir,
                format!("resource cache entry could not be read: {error}"),
            )),
        }
    }
}

fn clean_resource_cache_candidate(
    path: PathBuf,
    file_name: &str,
    planned_file_names: &BTreeSet<String>,
    entries: &mut Vec<ResourceCacheCleanEntry>,
) {
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) => {
            entries.push(failed_clean_entry(
                &path,
                format!("resource cache entry could not be inspected: {error}"),
            ));
            return;
        }
    };

    if metadata.file_type().is_symlink() {
        match fs::remove_file(&path) {
            Ok(()) => entries.push(ResourceCacheCleanEntry {
                path: path.to_string_lossy().to_string(),
                status: ResourceCacheCleanStatus::Removed,
                bytes_removed: metadata.len(),
                message: "unsafe resource cache symlink removed".to_string(),
            }),
            Err(error) => entries.push(ResourceCacheCleanEntry {
                path: path.to_string_lossy().to_string(),
                status: ResourceCacheCleanStatus::Failed,
                bytes_removed: 0,
                message: format!("unsafe resource cache symlink could not be removed: {error}"),
            }),
        }
        return;
    }

    if metadata.is_dir() {
        entries.push(ResourceCacheCleanEntry {
            path: path.to_string_lossy().to_string(),
            status: ResourceCacheCleanStatus::Skipped,
            bytes_removed: 0,
            message: "resource cache entry is a directory".to_string(),
        });
        return;
    }

    if planned_file_names.contains(file_name) {
        entries.push(ResourceCacheCleanEntry {
            path: path.to_string_lossy().to_string(),
            status: ResourceCacheCleanStatus::Kept,
            bytes_removed: 0,
            message: "resource cache entry is current".to_string(),
        });
        return;
    }

    let bytes_removed = metadata.len();
    match fs::remove_file(&path) {
        Ok(()) => entries.push(ResourceCacheCleanEntry {
            path: path.to_string_lossy().to_string(),
            status: ResourceCacheCleanStatus::Removed,
            bytes_removed,
            message: "stale resource cache entry removed".to_string(),
        }),
        Err(error) => entries.push(ResourceCacheCleanEntry {
            path: path.to_string_lossy().to_string(),
            status: ResourceCacheCleanStatus::Failed,
            bytes_removed: 0,
            message: format!("stale resource cache entry could not be removed: {error}"),
        }),
    }
}

fn failed_clean_entry(path: &Path, message: String) -> ResourceCacheCleanEntry {
    ResourceCacheCleanEntry {
        path: path.to_string_lossy().to_string(),
        status: ResourceCacheCleanStatus::Failed,
        bytes_removed: 0,
        message,
    }
}

fn prepare_cache_file_target(root: &Path, cache_path: &Path) -> io::Result<()> {
    if let Some(parent) = cache_path.parent() {
        create_cache_directory(root, parent)?;
    }

    match fs::symlink_metadata(cache_path) {
        Ok(metadata) if metadata.file_type().is_symlink() || metadata.is_dir() => {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "resource cache target is not a safe file",
            ))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn create_cache_directory(root: &Path, directory: &Path) -> io::Result<()> {
    let relative_directory = directory.strip_prefix(root).map_err(|_| {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            "resource cache directory is not under the content root",
        )
    })?;
    let mut current = root.to_path_buf();

    for component in relative_directory.components() {
        let Component::Normal(part) = component else {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "resource cache directory has an unsafe component",
            ));
        };

        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "resource cache directory is not a safe directory",
                ));
            }
            Ok(_) => ensure_cache_path_stays_in_root(root, &current)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(&current)?;
                ensure_cache_path_stays_in_root(root, &current)?;
            }
            Err(error) => return Err(error),
        }
    }

    Ok(())
}

fn ensure_cache_path_stays_in_root(root: &Path, path: &Path) -> io::Result<()> {
    if ensure_resolved_path_stays_in_root(root, path)? {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "path escapes the content root",
        ))
    }
}

fn skipped_cache_entry(entry: &ResourceResolution, message: &str) -> ResourceCacheEntry {
    ResourceCacheEntry {
        resource_id: entry.resource_id.clone(),
        source_path: entry.source_path.clone(),
        cache_path: entry.cache_path.clone(),
        status: ResourceCacheStatus::Skipped,
        bytes_copied: 0,
        message: message.to_string(),
    }
}

fn preflight_issue_message(entry: &ResourceResolution) -> String {
    match entry.status {
        ResourceResolutionStatus::Missing => format!(
            "resource file is missing: {} -> {}",
            entry.resource_id,
            entry
                .resolved_path
                .as_deref()
                .unwrap_or(entry.source_path.as_str())
        ),
        ResourceResolutionStatus::UnsafePath => format!(
            "resource path is unsafe: {} -> {}",
            entry.resource_id, entry.source_path
        ),
        ResourceResolutionStatus::HashMismatch => format!(
            "resource hash mismatch: {} expected {} found {}",
            entry.resource_id,
            entry.expected_sha256.as_deref().unwrap_or("<none>"),
            entry.actual_sha256.as_deref().unwrap_or("<unavailable>")
        ),
        ResourceResolutionStatus::IoError => format!(
            "resource file could not be inspected: {} -> {}",
            entry.resource_id,
            entry
                .resolved_path
                .as_deref()
                .unwrap_or(entry.source_path.as_str())
        ),
        ResourceResolutionStatus::Planned | ResourceResolutionStatus::Ready => {
            format!("resource is ready: {}", entry.resource_id)
        }
    }
}

fn normalize_resource_path(source_path: &str) -> Option<PathBuf> {
    if source_path.trim().is_empty() {
        return None;
    }

    if source_path.contains(':') {
        return None;
    }

    let mut normalized = PathBuf::new();
    for component in Path::new(source_path).components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => return None,
        }
    }

    (!normalized.as_os_str().is_empty()).then_some(normalized)
}

fn ensure_resolved_path_stays_in_root(root: &Path, path: &Path) -> io::Result<bool> {
    let root = fs::canonicalize(root)?;
    let path = fs::canonicalize(path)?;
    Ok(path.starts_with(root))
}

fn fallback_for_media_type(media_type: &ResourceMediaType) -> ResourceFallback {
    match media_type {
        ResourceMediaType::Image => ResourceFallback::PlaceholderImage,
        ResourceMediaType::Audio => ResourceFallback::SilentAudio,
        ResourceMediaType::Font => ResourceFallback::DefaultFont,
        ResourceMediaType::Other => ResourceFallback::MissingResource,
    }
}

fn load_strategy_for_media_type(
    media_type: &ResourceMediaType,
    low_spec: bool,
) -> ResourceLoadStrategy {
    if !low_spec {
        return ResourceLoadStrategy::Eager;
    }

    match media_type {
        ResourceMediaType::Image => ResourceLoadStrategy::ThumbnailOnly,
        ResourceMediaType::Audio | ResourceMediaType::Other => ResourceLoadStrategy::Deferred,
        ResourceMediaType::Font => ResourceLoadStrategy::Eager,
    }
}

fn cache_key_for_resource(resource: &ResourceAsset) -> String {
    let readable_name = sanitize_cache_name(&resource.resource_id);
    let identity = format!(
        "{}\n{}\n{}\n{}",
        resource.resource_id,
        normalize_resource_cache_source(&resource.source_path),
        media_type_key(&resource.media_type),
        resource.sha256.as_deref().unwrap_or_default()
    );
    format!("{readable_name}-{}", fnv1a64_hex(&identity))
}

fn cache_path_for_resource(root: &Path, resource: &ResourceAsset, cache_key: &str) -> String {
    root.join(".eratw-cache")
        .join("resources")
        .join(format!(
            "{cache_key}.{}",
            cache_file_extension(resource)
                .unwrap_or_else(|| default_cache_extension(&resource.media_type).to_string())
        ))
        .to_string_lossy()
        .to_string()
}

fn thumbnail_path_for_resource(
    root: &Path,
    cache_key: &str,
    load_strategy: &ResourceLoadStrategy,
) -> Option<String> {
    matches!(load_strategy, ResourceLoadStrategy::ThumbnailOnly).then(|| {
        root.join(".eratw-cache")
            .join("thumbnails")
            .join(format!("{cache_key}.webp"))
            .to_string_lossy()
            .to_string()
    })
}

fn media_type_key(media_type: &ResourceMediaType) -> &'static str {
    match media_type {
        ResourceMediaType::Image => "image",
        ResourceMediaType::Audio => "audio",
        ResourceMediaType::Font => "font",
        ResourceMediaType::Other => "other",
    }
}

fn cache_file_extension(resource: &ResourceAsset) -> Option<String> {
    Path::new(&resource.source_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .filter(|extension| {
            !extension.is_empty()
                && extension.len() <= 12
                && extension
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
        })
}

fn default_cache_extension(media_type: &ResourceMediaType) -> &'static str {
    match media_type {
        ResourceMediaType::Image => "webp",
        ResourceMediaType::Audio => "ogg",
        ResourceMediaType::Font => "ttf",
        ResourceMediaType::Other => "bin",
    }
}

fn normalize_resource_cache_source(source_path: &str) -> String {
    source_path.replace('\\', "/")
}

fn sanitize_cache_name(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut previous_was_separator = false;

    for character in value.chars() {
        let next = if character.is_ascii_alphanumeric() {
            previous_was_separator = false;
            Some(character.to_ascii_lowercase())
        } else if matches!(character, '.' | '-' | '_') {
            if previous_was_separator {
                None
            } else {
                previous_was_separator = true;
                Some(character)
            }
        } else if previous_was_separator {
            None
        } else {
            previous_was_separator = true;
            Some('_')
        };

        if let Some(character) = next {
            output.push(character);
        }
    }

    let output = output.trim_matches(&['.', '-', '_'][..]);
    if output.is_empty() {
        "resource".to_string()
    } else {
        output.to_string()
    }
}

fn fnv1a64_hex(input: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

fn sha256_file(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }

    Ok(hex_lower(&digest.finalize()))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResourceMediaType;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn resource_paths_reject_absolute_parent_and_empty_sources() {
        assert!(!is_safe_resource_source_path("../outside.webp"));
        assert!(!is_safe_resource_source_path("/absolute.webp"));
        assert!(!is_safe_resource_source_path("C:\\absolute.webp"));
        assert!(!is_safe_resource_source_path(""));
        assert!(is_safe_resource_source_path("./assets/heroine.webp"));
    }

    #[test]
    fn plan_resource_loads_resolves_safe_paths_without_file_io() {
        let report = plan_resource_loads(
            &[resource(
                "portrait",
                "assets/heroine.webp",
                ResourceMediaType::Image,
                None,
            )],
            "mods/sample",
        );

        assert_eq!(report.entries.len(), 1);
        assert!(!report.low_spec);
        assert_eq!(report.entries[0].status, ResourceResolutionStatus::Planned);
        assert_eq!(report.entries[0].load_strategy, ResourceLoadStrategy::Eager);
        assert!(report.entries[0].cache_key.starts_with("portrait-"));
        assert!(report.entries[0]
            .cache_path
            .as_ref()
            .unwrap()
            .contains(".eratw-cache"));
        assert_eq!(report.entries[0].thumbnail_path, None);
        assert_eq!(
            report.entries[0].fallback,
            ResourceFallback::PlaceholderImage
        );
        assert!(
            report.entries[0]
                .resolved_path
                .as_ref()
                .unwrap()
                .ends_with("mods/sample\\assets\\heroine.webp")
                || report.entries[0]
                    .resolved_path
                    .as_ref()
                    .unwrap()
                    .ends_with("mods/sample/assets/heroine.webp")
        );
    }

    #[test]
    fn low_spec_resource_plan_uses_thumbnails_and_deferred_loads() {
        let report = plan_resource_loads_with_options(
            &[
                resource(
                    "portrait.heroine",
                    "assets/heroine.webp",
                    ResourceMediaType::Image,
                    None,
                ),
                resource("voice", "assets/voice.ogg", ResourceMediaType::Audio, None),
                resource("font", "assets/ui.ttf", ResourceMediaType::Font, None),
                resource("data", "assets/data.bin", ResourceMediaType::Other, None),
                resource("unsafe", "../outside.webp", ResourceMediaType::Image, None),
            ],
            "mods/sample",
            ResourcePlanningOptions { low_spec: true },
        );

        assert!(report.low_spec);
        assert_eq!(
            report
                .entries
                .iter()
                .map(|entry| &entry.load_strategy)
                .collect::<Vec<_>>(),
            vec![
                &ResourceLoadStrategy::ThumbnailOnly,
                &ResourceLoadStrategy::Deferred,
                &ResourceLoadStrategy::Eager,
                &ResourceLoadStrategy::Deferred,
                &ResourceLoadStrategy::ThumbnailOnly,
            ]
        );
        assert!(report.entries[0]
            .thumbnail_path
            .as_ref()
            .unwrap()
            .ends_with(".webp"));
        assert!(report.entries[0]
            .thumbnail_path
            .as_ref()
            .unwrap()
            .contains(".eratw-cache"));
        assert_eq!(report.entries[1].thumbnail_path, None);
        assert!(report.entries[1]
            .cache_path
            .as_ref()
            .unwrap()
            .ends_with(".ogg"));
        assert_eq!(report.entries[4].resolved_path, None);
        assert_eq!(report.entries[4].cache_path, None);
        assert_eq!(report.entries[4].thumbnail_path, None);
    }

    #[test]
    fn inspect_resource_files_reports_ready_missing_and_hash_mismatch() {
        let dir = temp_resource_dir("resource_probe");
        fs::create_dir_all(dir.join("assets")).unwrap();
        fs::write(dir.join("assets/ready.txt"), b"ready").unwrap();
        fs::write(dir.join("assets/mismatch.txt"), b"mismatch").unwrap();
        let ready_hash = sha256_file(&dir.join("assets/ready.txt")).unwrap();

        let report = inspect_resource_files(
            &[
                resource(
                    "ready",
                    "assets/ready.txt",
                    ResourceMediaType::Other,
                    Some(ready_hash),
                ),
                resource(
                    "missing",
                    "assets/missing.txt",
                    ResourceMediaType::Audio,
                    None,
                ),
                resource(
                    "mismatch",
                    "assets/mismatch.txt",
                    ResourceMediaType::Font,
                    Some("0000".to_string()),
                ),
                resource("unsafe", "../outside.txt", ResourceMediaType::Image, None),
            ],
            &dir,
        );

        assert_eq!(report.entries[0].status, ResourceResolutionStatus::Ready);
        assert_eq!(report.entries[1].status, ResourceResolutionStatus::Missing);
        assert_eq!(report.entries[1].fallback, ResourceFallback::SilentAudio);
        assert_eq!(
            report.entries[2].status,
            ResourceResolutionStatus::HashMismatch
        );
        assert_eq!(report.entries[2].fallback, ResourceFallback::DefaultFont);
        assert_eq!(
            report.entries[3].status,
            ResourceResolutionStatus::UnsafePath
        );
        assert_eq!(report.entries[3].resolved_path, None);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn preflight_resource_loads_reports_ready_resources() {
        let dir = temp_resource_dir("resource_preflight_ready");
        fs::create_dir_all(dir.join("assets")).unwrap();
        fs::write(dir.join("assets/ready.txt"), b"ready").unwrap();
        let ready_hash = sha256_file(&dir.join("assets/ready.txt")).unwrap();

        let report = preflight_resource_loads(
            &[resource(
                "ready",
                "assets/ready.txt",
                ResourceMediaType::Image,
                Some(ready_hash),
            )],
            &dir,
        );

        assert!(report.ready);
        assert!(report.issues.is_empty());
        assert_eq!(
            report.resolution.entries[0].status,
            ResourceResolutionStatus::Ready
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn preflight_resource_loads_reports_blocking_issues() {
        let dir = temp_resource_dir("resource_preflight_blocked");
        fs::create_dir_all(dir.join("assets")).unwrap();
        fs::write(dir.join("assets/mismatch.txt"), b"mismatch").unwrap();

        let report = preflight_resource_loads(
            &[
                resource(
                    "missing",
                    "assets/missing.txt",
                    ResourceMediaType::Audio,
                    None,
                ),
                resource(
                    "mismatch",
                    "assets/mismatch.txt",
                    ResourceMediaType::Font,
                    Some("0000".to_string()),
                ),
                resource("unsafe", "../outside.txt", ResourceMediaType::Image, None),
            ],
            &dir,
        );

        assert!(!report.ready);
        assert_eq!(
            report
                .issues
                .iter()
                .map(|issue| &issue.code)
                .collect::<Vec<_>>(),
            vec![
                &ResourcePreflightIssueCode::Missing,
                &ResourcePreflightIssueCode::HashMismatch,
                &ResourcePreflightIssueCode::UnsafePath
            ]
        );
        assert_eq!(report.issues[0].fallback, ResourceFallback::SilentAudio);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn audit_resource_publication_allows_ready_resources_with_sha_warnings() {
        let dir = temp_resource_dir("resource_publish_ready");
        fs::create_dir_all(dir.join("assets")).unwrap();
        fs::write(dir.join("assets/ready.txt"), b"ready").unwrap();

        let report = audit_resource_publication(
            &[resource(
                "ready",
                "assets/ready.txt",
                ResourceMediaType::Other,
                None,
            )],
            &dir,
        );

        assert!(report.ready);
        assert_eq!(report.error_count, 0);
        assert_eq!(report.warning_count, 1);
        assert_eq!(
            report.issues[0].severity,
            ResourcePublishIssueSeverity::Warning
        );
        assert_eq!(
            report.issues[0].code,
            ResourcePublishIssueCode::MissingSha256
        );
        assert_eq!(
            report.resolution.entries[0].status,
            ResourceResolutionStatus::Ready
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn audit_resource_publication_blocks_file_and_metadata_errors() {
        let dir = temp_resource_dir("resource_publish_blocked");
        fs::create_dir_all(dir.join("assets")).unwrap();
        fs::write(dir.join("assets/mismatch.txt"), b"mismatch").unwrap();
        let mut unknown = resource(
            "unknown",
            "assets/missing.txt",
            ResourceMediaType::Audio,
            None,
        );
        unknown.license = "unknown".to_string();
        unknown.author = "".to_string();

        let report = audit_resource_publication(
            &[
                unknown,
                resource(
                    "mismatch",
                    "assets/mismatch.txt",
                    ResourceMediaType::Image,
                    Some("0000".to_string()),
                ),
            ],
            &dir,
        );

        assert!(!report.ready);
        assert_eq!(report.error_count, 4);
        assert_eq!(report.warning_count, 1);
        assert_eq!(
            report
                .issues
                .iter()
                .map(|issue| &issue.code)
                .collect::<Vec<_>>(),
            vec![
                &ResourcePublishIssueCode::Missing,
                &ResourcePublishIssueCode::UnknownLicense,
                &ResourcePublishIssueCode::EmptyAuthor,
                &ResourcePublishIssueCode::MissingSha256,
                &ResourcePublishIssueCode::HashMismatch,
            ]
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cache_resource_loads_copies_ready_resources_to_cache() {
        let dir = temp_resource_dir("resource_cache_ready");
        fs::create_dir_all(dir.join("assets")).unwrap();
        fs::write(dir.join("assets/ready.txt"), b"ready").unwrap();
        let ready_hash = sha256_file(&dir.join("assets/ready.txt")).unwrap();

        let report = cache_resource_loads(
            &[resource(
                "ready",
                "assets/ready.txt",
                ResourceMediaType::Other,
                Some(ready_hash),
            )],
            &dir,
        );

        assert!(report.ready);
        assert_eq!(report.cached_count, 1);
        assert_eq!(report.skipped_count, 0);
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.entries[0].status, ResourceCacheStatus::Cached);
        assert_eq!(report.entries[0].bytes_copied, 5);
        let cache_path = PathBuf::from(report.entries[0].cache_path.as_ref().unwrap());
        assert_eq!(fs::read(cache_path).unwrap(), b"ready");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cache_resource_loads_skips_blocked_resources() {
        let dir = temp_resource_dir("resource_cache_blocked");
        fs::create_dir_all(dir.join("assets")).unwrap();
        fs::write(dir.join("assets/mismatch.txt"), b"mismatch").unwrap();

        let report = cache_resource_loads(
            &[
                resource(
                    "missing",
                    "assets/missing.txt",
                    ResourceMediaType::Audio,
                    None,
                ),
                resource(
                    "mismatch",
                    "assets/mismatch.txt",
                    ResourceMediaType::Font,
                    Some("0000".to_string()),
                ),
                resource("unsafe", "../outside.txt", ResourceMediaType::Image, None),
            ],
            &dir,
        );

        assert!(report.ready);
        assert_eq!(report.cached_count, 0);
        assert_eq!(report.skipped_count, 3);
        assert_eq!(report.failed_count, 0);
        assert!(report
            .entries
            .iter()
            .all(|entry| entry.status == ResourceCacheStatus::Skipped));
        assert!(!dir.join(".eratw-cache/resources").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn clean_resource_cache_removes_stale_resource_and_thumbnail_files() {
        let dir = temp_resource_dir("resource_cache_clean_stale");
        fs::create_dir_all(dir.join("assets")).unwrap();
        fs::write(dir.join("assets/ready.webp"), b"ready").unwrap();
        let resources = vec![resource(
            "ready",
            "assets/ready.webp",
            ResourceMediaType::Image,
            None,
        )];
        let plan = plan_resource_loads_with_options(
            &resources,
            &dir,
            ResourcePlanningOptions { low_spec: true },
        );
        let current_cache_path = PathBuf::from(plan.entries[0].cache_path.as_ref().unwrap());
        let current_thumbnail_path =
            PathBuf::from(plan.entries[0].thumbnail_path.as_ref().unwrap());
        let stale_cache_path = dir.join(".eratw-cache/resources/stale.bin");
        let stale_thumbnail_path = dir.join(".eratw-cache/thumbnails/stale.webp");
        let nested_dir = dir.join(".eratw-cache/resources/nested");
        fs::create_dir_all(current_cache_path.parent().unwrap()).unwrap();
        fs::create_dir_all(current_thumbnail_path.parent().unwrap()).unwrap();
        fs::create_dir_all(&nested_dir).unwrap();
        fs::write(&current_cache_path, b"keep-resource").unwrap();
        fs::write(&current_thumbnail_path, b"keep-thumbnail").unwrap();
        fs::write(&stale_cache_path, b"remove-resource").unwrap();
        fs::write(&stale_thumbnail_path, b"remove-thumbnail").unwrap();

        let report = clean_resource_cache_with_options(
            &resources,
            &dir,
            ResourcePlanningOptions { low_spec: true },
        );

        assert!(report.ready);
        assert!(report.low_spec);
        assert_eq!(report.kept_count, 2);
        assert_eq!(report.removed_count, 2);
        assert_eq!(report.skipped_count, 1);
        assert_eq!(report.failed_count, 0);
        assert_eq!(
            report.bytes_removed,
            b"remove-resource".len() as u64 + b"remove-thumbnail".len() as u64
        );
        assert!(current_cache_path.exists());
        assert!(current_thumbnail_path.exists());
        assert!(!stale_cache_path.exists());
        assert!(!stale_thumbnail_path.exists());
        assert!(nested_dir.exists());
        assert_eq!(
            report.resolution.entries[0].status,
            ResourceResolutionStatus::Planned
        );
        assert!(report
            .entries
            .iter()
            .any(|entry| entry.status == ResourceCacheCleanStatus::Skipped
                && entry.path.ends_with("nested")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn clean_resource_cache_keeps_cache_for_changed_resource_identity_separate() {
        let dir = temp_resource_dir("resource_cache_clean_identity");
        let old_resource = resource(
            "portrait",
            "assets/portrait.webp",
            ResourceMediaType::Image,
            Some("old-hash".to_string()),
        );
        let new_resource = resource(
            "portrait",
            "assets/portrait.webp",
            ResourceMediaType::Image,
            Some("new-hash".to_string()),
        );
        let old_plan = plan_resource_loads(&[old_resource], &dir);
        let new_plan = plan_resource_loads(&[new_resource.clone()], &dir);
        let old_cache_path = PathBuf::from(old_plan.entries[0].cache_path.as_ref().unwrap());
        let new_cache_path = PathBuf::from(new_plan.entries[0].cache_path.as_ref().unwrap());
        fs::create_dir_all(old_cache_path.parent().unwrap()).unwrap();
        fs::write(&old_cache_path, b"old").unwrap();
        fs::write(&new_cache_path, b"new").unwrap();

        let report = clean_resource_cache(&[new_resource], &dir);

        assert_eq!(report.kept_count, 1);
        assert_eq!(report.removed_count, 1);
        assert!(new_cache_path.exists());
        assert!(!old_cache_path.exists());

        let _ = fs::remove_dir_all(dir);
    }

    fn resource(
        resource_id: &str,
        source_path: &str,
        media_type: ResourceMediaType,
        sha256: Option<String>,
    ) -> ResourceAsset {
        ResourceAsset {
            resource_id: resource_id.to_string(),
            source_path: source_path.to_string(),
            media_type,
            license: "project-demo".to_string(),
            author: "ERAtw-NEXT".to_string(),
            usage: Vec::new(),
            character_bindings: Vec::new(),
            tags: Vec::new(),
            sha256,
        }
    }

    fn temp_resource_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("eratw_next_{label}_{nonce}"))
    }
}
