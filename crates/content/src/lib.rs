use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentPackageManifest {
    pub schema_version: String,
    pub namespace: String,
    pub package_id: String,
    pub version: String,
    pub dependencies: Vec<String>,
}

impl ContentPackageManifest {
    pub fn new(namespace: impl Into<String>, package_id: impl Into<String>) -> Self {
        Self {
            schema_version: "content-package/v0".to_string(),
            namespace: namespace.into(),
            package_id: package_id.into(),
            version: "0.1.0".to_string(),
            dependencies: Vec::new(),
        }
    }
}
