use crate::{ResourceAsset, ResourceMediaType};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Read},
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceResolutionReport {
    pub root: String,
    pub entries: Vec<ResourceResolution>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceResolution {
    pub resource_id: String,
    pub source_path: String,
    pub resolved_path: Option<String>,
    pub media_type: ResourceMediaType,
    pub status: ResourceResolutionStatus,
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
    resolve_resources(resources, root, false)
}

pub fn inspect_resource_files(
    resources: &[ResourceAsset],
    root: impl AsRef<Path>,
) -> ResourceResolutionReport {
    resolve_resources(resources, root, true)
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
) -> ResourceResolutionReport {
    let root = root.as_ref();
    let entries = resources
        .iter()
        .map(|resource| resolve_resource(root, resource, inspect_files))
        .collect();

    ResourceResolutionReport {
        root: root.to_string_lossy().to_string(),
        entries,
    }
}

fn resolve_resource(
    root: &Path,
    resource: &ResourceAsset,
    inspect_files: bool,
) -> ResourceResolution {
    let fallback = fallback_for_media_type(&resource.media_type);
    let Some(path) = resolve_resource_path(root, &resource.source_path) else {
        return ResourceResolution {
            resource_id: resource.resource_id.clone(),
            source_path: resource.source_path.clone(),
            resolved_path: None,
            media_type: resource.media_type.clone(),
            status: ResourceResolutionStatus::UnsafePath,
            fallback,
            expected_sha256: resource.sha256.clone(),
            actual_sha256: None,
        };
    };

    if !inspect_files {
        return ResourceResolution {
            resource_id: resource.resource_id.clone(),
            source_path: resource.source_path.clone(),
            resolved_path: Some(path.to_string_lossy().to_string()),
            media_type: resource.media_type.clone(),
            status: ResourceResolutionStatus::Planned,
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
        fallback,
        expected_sha256: resource.sha256.clone(),
        actual_sha256,
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
        assert_eq!(report.entries[0].status, ResourceResolutionStatus::Planned);
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
