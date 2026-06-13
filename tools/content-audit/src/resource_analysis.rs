use regex::Regex;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResourceFileAnalysis {
    pub sha256: String,
    pub numeric_prefix: Option<u64>,
    pub variant_tokens: Vec<String>,
}

pub(crate) fn analyze_resource(path: &Path) -> io::Result<ResourceFileAnalysis> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    let stem = path
        .file_stem()
        .map(|value| value.to_string_lossy())
        .unwrap_or_default();
    let numeric_prefix = numeric_prefix_regex()
        .captures(&stem)
        .and_then(|captures| captures.get(1))
        .and_then(|value| value.as_str().parse().ok());
    let variant_tokens = stem
        .split(['_', '-', ' ', '　'])
        .filter(|token| !token.is_empty() && !token.chars().all(|ch| ch.is_ascii_digit()))
        .map(|token| token.to_lowercase())
        .collect();

    Ok(ResourceFileAnalysis {
        sha256: format!("{:x}", hasher.finalize()),
        numeric_prefix,
        variant_tokens,
    })
}

fn numeric_prefix_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^(\d+)").expect("valid numeric prefix regex"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn hashes_and_extracts_resource_name_metadata() {
        let path = std::env::temp_dir().join(format!(
            "eratw_next_resource_{}_001_face-default.webp",
            std::process::id()
        ));
        fs::write(&path, b"fixture").unwrap();
        let analysis = analyze_resource(&path).unwrap();
        assert_eq!(analysis.numeric_prefix, None);
        assert_eq!(analysis.sha256.len(), 64);
        assert!(analysis.variant_tokens.iter().any(|token| token == "face"));
        fs::remove_file(path).unwrap();
    }
}
