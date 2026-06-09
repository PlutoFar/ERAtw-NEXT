use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModManifest {
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub engine_version: String,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<ModCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModCapability {
    Content,
    Theme,
    RulesExtension,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ModValidationError {
    #[error("namespace is required")]
    MissingNamespace,
    #[error("unsafe capability is not allowed by default: {0}")]
    UnsafeCapability(String),
}

pub fn validate_manifest(manifest: &ModManifest) -> Result<(), ModValidationError> {
    if manifest.namespace.trim().is_empty() {
        return Err(ModValidationError::MissingNamespace);
    }

    Ok(())
}
