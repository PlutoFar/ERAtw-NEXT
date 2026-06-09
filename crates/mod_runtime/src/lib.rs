use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModManifest {
    pub namespace: String,
    pub name: String,
    pub version: String,
    #[serde(alias = "engineVersion")]
    pub engine_version: String,
    #[serde(default, alias = "loadOrder")]
    pub load_order: i16,
    #[serde(default)]
    pub dependencies: Vec<ModDependency>,
    #[serde(default)]
    pub conflicts: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<ModCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "ModDependencyWire")]
pub struct ModDependency {
    pub namespace: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default = "default_required")]
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
enum ModDependencyWire {
    Namespace(String),
    Object {
        namespace: String,
        #[serde(default)]
        version: Option<String>,
        #[serde(default = "default_required")]
        required: bool,
    },
}

impl From<ModDependencyWire> for ModDependency {
    fn from(value: ModDependencyWire) -> Self {
        match value {
            ModDependencyWire::Namespace(namespace) => Self {
                namespace,
                version: None,
                required: true,
            },
            ModDependencyWire::Object {
                namespace,
                version,
                required,
            } => Self {
                namespace,
                version,
                required,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModCapability {
    Content,
    Theme,
    RulesExtension,
    LocalFileAccess,
    NetworkAccess,
    SystemCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModLoadPlan {
    pub manifests: Vec<ModManifest>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ModValidationError {
    #[error("namespace is required")]
    MissingNamespace,
    #[error("mod name is required: {0}")]
    MissingName(String),
    #[error("mod version is required: {0}")]
    MissingVersion(String),
    #[error("engine version is required: {0}")]
    MissingEngineVersion(String),
    #[error("mod engine version is incompatible: {manifest_namespace} expected {expected_engine_version} found {actual_engine_version}")]
    IncompatibleEngineVersion {
        manifest_namespace: String,
        expected_engine_version: String,
        actual_engine_version: String,
    },
    #[error("duplicate dependency declaration: {manifest_namespace} -> {dependency_namespace}")]
    DuplicateDependency {
        manifest_namespace: String,
        dependency_namespace: String,
    },
    #[error("duplicate conflict declaration: {manifest_namespace} -> {conflict_namespace}")]
    DuplicateConflict {
        manifest_namespace: String,
        conflict_namespace: String,
    },
    #[error("unsafe capability is not allowed by default: {manifest_namespace} -> {capability}")]
    UnsafeCapability {
        manifest_namespace: String,
        capability: String,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ModLoadError {
    #[error(transparent)]
    Validation(#[from] ModValidationError),
    #[error("duplicate mod namespace: {0}")]
    DuplicateNamespace(String),
    #[error("required mod dependency is missing: {manifest_namespace} -> {dependency_namespace}")]
    MissingDependency {
        manifest_namespace: String,
        dependency_namespace: String,
    },
    #[error(
        "mod dependency version mismatch: {manifest_namespace} -> {dependency_namespace} expected {expected_version} found {actual_version}"
    )]
    DependencyVersionMismatch {
        manifest_namespace: String,
        dependency_namespace: String,
        expected_version: String,
        actual_version: String,
    },
    #[error("mod conflict detected: {left_namespace} <-> {right_namespace}")]
    Conflict {
        left_namespace: String,
        right_namespace: String,
    },
    #[error("mod dependency cycle detected: {0}")]
    DependencyCycle(String),
}

pub fn validate_manifest(manifest: &ModManifest) -> Result<(), ModValidationError> {
    if manifest.namespace.trim().is_empty() {
        return Err(ModValidationError::MissingNamespace);
    }

    if manifest.name.trim().is_empty() {
        return Err(ModValidationError::MissingName(manifest.namespace.clone()));
    }

    if manifest.version.trim().is_empty() {
        return Err(ModValidationError::MissingVersion(
            manifest.namespace.clone(),
        ));
    }

    if manifest.engine_version.trim().is_empty() {
        return Err(ModValidationError::MissingEngineVersion(
            manifest.namespace.clone(),
        ));
    }

    let mut dependency_namespaces = BTreeSet::new();
    for dependency in &manifest.dependencies {
        if dependency.namespace.trim().is_empty() {
            return Err(ModValidationError::DuplicateDependency {
                manifest_namespace: manifest.namespace.clone(),
                dependency_namespace: String::new(),
            });
        }

        if !dependency_namespaces.insert(dependency.namespace.as_str()) {
            return Err(ModValidationError::DuplicateDependency {
                manifest_namespace: manifest.namespace.clone(),
                dependency_namespace: dependency.namespace.clone(),
            });
        }
    }

    let mut conflicts = BTreeSet::new();
    for conflict in &manifest.conflicts {
        if conflict.trim().is_empty() || !conflicts.insert(conflict.as_str()) {
            return Err(ModValidationError::DuplicateConflict {
                manifest_namespace: manifest.namespace.clone(),
                conflict_namespace: conflict.clone(),
            });
        }
    }

    for capability in &manifest.capabilities {
        if is_unsafe_capability(capability) {
            return Err(ModValidationError::UnsafeCapability {
                manifest_namespace: manifest.namespace.clone(),
                capability: capability_label(capability).to_string(),
            });
        }
    }

    Ok(())
}

pub fn validate_manifest_for_engine(
    manifest: &ModManifest,
    engine_version: &str,
) -> Result<(), ModValidationError> {
    validate_manifest(manifest)?;

    if manifest.engine_version != engine_version {
        return Err(ModValidationError::IncompatibleEngineVersion {
            manifest_namespace: manifest.namespace.clone(),
            expected_engine_version: manifest.engine_version.clone(),
            actual_engine_version: engine_version.to_string(),
        });
    }

    Ok(())
}

pub fn plan_load_order(manifests: Vec<ModManifest>) -> Result<ModLoadPlan, ModLoadError> {
    plan_load_order_for_engine(manifests, None)
}

pub fn plan_load_order_for_engine(
    manifests: Vec<ModManifest>,
    engine_version: Option<&str>,
) -> Result<ModLoadPlan, ModLoadError> {
    let mut by_namespace = BTreeMap::new();
    for manifest in manifests {
        if let Some(engine_version) = engine_version {
            validate_manifest_for_engine(&manifest, engine_version)?;
        } else {
            validate_manifest(&manifest)?;
        }
        if by_namespace.contains_key(&manifest.namespace) {
            return Err(ModLoadError::DuplicateNamespace(manifest.namespace.clone()));
        }
        by_namespace.insert(manifest.namespace.clone(), manifest);
    }

    ensure_dependencies_exist(&by_namespace)?;
    ensure_no_conflicts(&by_namespace)?;

    let mut indegrees: BTreeMap<String, usize> = by_namespace
        .keys()
        .map(|namespace| (namespace.clone(), 0))
        .collect();
    let mut dependents: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for manifest in by_namespace.values() {
        for dependency in &manifest.dependencies {
            if by_namespace.contains_key(&dependency.namespace) {
                *indegrees
                    .get_mut(&manifest.namespace)
                    .expect("manifest indegree exists") += 1;
                dependents
                    .entry(dependency.namespace.clone())
                    .or_default()
                    .push(manifest.namespace.clone());
            }
        }
    }

    let mut ordered = Vec::new();
    while ordered.len() < by_namespace.len() {
        let Some(next_namespace) = next_ready_namespace(&by_namespace, &indegrees) else {
            let cycle_start = indegrees
                .iter()
                .find_map(|(namespace, indegree)| (*indegree > 0).then_some(namespace.clone()))
                .expect("cycle has at least one node");
            return Err(ModLoadError::DependencyCycle(cycle_start));
        };

        indegrees.remove(&next_namespace);
        let manifest = by_namespace
            .get(&next_namespace)
            .expect("ready manifest exists")
            .clone();
        ordered.push(manifest);

        for dependent in dependents.remove(&next_namespace).unwrap_or_default() {
            if let Some(indegree) = indegrees.get_mut(&dependent) {
                *indegree = indegree.saturating_sub(1);
            }
        }
    }

    Ok(ModLoadPlan { manifests: ordered })
}

fn next_ready_namespace(
    manifests: &BTreeMap<String, ModManifest>,
    indegrees: &BTreeMap<String, usize>,
) -> Option<String> {
    indegrees
        .iter()
        .filter(|(_, indegree)| **indegree == 0)
        .map(|(namespace, _)| manifests.get(namespace).expect("manifest exists"))
        .min_by(|left, right| {
            left.load_order
                .cmp(&right.load_order)
                .then_with(|| left.namespace.cmp(&right.namespace))
        })
        .map(|manifest| manifest.namespace.clone())
}

fn ensure_dependencies_exist(
    manifests: &BTreeMap<String, ModManifest>,
) -> Result<(), ModLoadError> {
    for manifest in manifests.values() {
        for dependency in &manifest.dependencies {
            let Some(found) = manifests.get(&dependency.namespace) else {
                if dependency.required {
                    return Err(ModLoadError::MissingDependency {
                        manifest_namespace: manifest.namespace.clone(),
                        dependency_namespace: dependency.namespace.clone(),
                    });
                }
                continue;
            };

            if let Some(expected_version) = &dependency.version {
                if found.version != *expected_version {
                    return Err(ModLoadError::DependencyVersionMismatch {
                        manifest_namespace: manifest.namespace.clone(),
                        dependency_namespace: dependency.namespace.clone(),
                        expected_version: expected_version.clone(),
                        actual_version: found.version.clone(),
                    });
                }
            }
        }
    }

    Ok(())
}

fn ensure_no_conflicts(manifests: &BTreeMap<String, ModManifest>) -> Result<(), ModLoadError> {
    for manifest in manifests.values() {
        for conflict in &manifest.conflicts {
            if manifests.contains_key(conflict) {
                return Err(ModLoadError::Conflict {
                    left_namespace: manifest.namespace.clone(),
                    right_namespace: conflict.clone(),
                });
            }
        }
    }

    Ok(())
}

fn is_unsafe_capability(capability: &ModCapability) -> bool {
    matches!(
        capability,
        ModCapability::LocalFileAccess
            | ModCapability::NetworkAccess
            | ModCapability::SystemCommand
    )
}

fn capability_label(capability: &ModCapability) -> &'static str {
    match capability {
        ModCapability::Content => "content",
        ModCapability::Theme => "theme",
        ModCapability::RulesExtension => "rules_extension",
        ModCapability::LocalFileAccess => "local_file_access",
        ModCapability::NetworkAccess => "network_access",
        ModCapability::SystemCommand => "system_command",
    }
}

fn default_required() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_only_manifest_validates() {
        let manifest = manifest("example.character");

        validate_manifest(&manifest).unwrap();
    }

    #[test]
    fn unsafe_capabilities_are_rejected_by_default() {
        let mut manifest = manifest("example.unsafe");
        manifest.capabilities = vec![ModCapability::NetworkAccess];

        let result = validate_manifest(&manifest);

        assert_eq!(
            result,
            Err(ModValidationError::UnsafeCapability {
                manifest_namespace: "example.unsafe".to_string(),
                capability: "network_access".to_string(),
            })
        );
    }

    #[test]
    fn duplicate_dependencies_are_rejected() {
        let mut manifest = manifest("example.duplicate");
        manifest.dependencies = vec![
            dependency("core.base", None),
            dependency("core.base", Some("1.0.0")),
        ];

        let result = validate_manifest(&manifest);

        assert_eq!(
            result,
            Err(ModValidationError::DuplicateDependency {
                manifest_namespace: "example.duplicate".to_string(),
                dependency_namespace: "core.base".to_string(),
            })
        );
    }

    #[test]
    fn manifest_can_be_validated_against_engine_version() {
        let manifest = manifest("example.character");

        validate_manifest_for_engine(&manifest, "0.1.0-m0").unwrap();

        let result = validate_manifest_for_engine(&manifest, "0.2.0");

        assert_eq!(
            result,
            Err(ModValidationError::IncompatibleEngineVersion {
                manifest_namespace: "example.character".to_string(),
                expected_engine_version: "0.1.0-m0".to_string(),
                actual_engine_version: "0.2.0".to_string(),
            })
        );
    }

    #[test]
    fn load_plan_orders_dependencies_first_then_load_order() {
        let mut base = manifest("core.base");
        base.load_order = -10;
        let mut late = manifest("example.late");
        late.load_order = 10;
        late.dependencies = vec![dependency("core.base", Some("0.1.0"))];
        let mut middle = manifest("example.middle");
        middle.dependencies = vec![dependency("core.base", None)];

        let plan = plan_load_order(vec![late, middle, base]).unwrap();

        assert_eq!(
            namespaces(&plan),
            vec!["core.base", "example.middle", "example.late"]
        );
    }

    #[test]
    fn load_plan_rejects_missing_required_dependency() {
        let mut manifest = manifest("example.character");
        manifest.dependencies = vec![dependency("core.base", None)];

        let result = plan_load_order(vec![manifest]);

        assert_eq!(
            result,
            Err(ModLoadError::MissingDependency {
                manifest_namespace: "example.character".to_string(),
                dependency_namespace: "core.base".to_string(),
            })
        );
    }

    #[test]
    fn load_plan_allows_missing_optional_dependency() {
        let mut manifest = manifest("example.character");
        let mut optional = dependency("core.optional", None);
        optional.required = false;
        manifest.dependencies = vec![optional];

        let plan = plan_load_order(vec![manifest]).unwrap();

        assert_eq!(namespaces(&plan), vec!["example.character"]);
    }

    #[test]
    fn load_plan_rejects_dependency_version_mismatch() {
        let base = manifest("core.base");
        let mut addon = manifest("example.addon");
        addon.dependencies = vec![dependency("core.base", Some("9.9.9"))];

        let result = plan_load_order(vec![base, addon]);

        assert_eq!(
            result,
            Err(ModLoadError::DependencyVersionMismatch {
                manifest_namespace: "example.addon".to_string(),
                dependency_namespace: "core.base".to_string(),
                expected_version: "9.9.9".to_string(),
                actual_version: "0.1.0".to_string(),
            })
        );
    }

    #[test]
    fn load_plan_rejects_declared_conflicts() {
        let mut left = manifest("example.left");
        left.conflicts = vec!["example.right".to_string()];
        let right = manifest("example.right");

        let result = plan_load_order(vec![left, right]);

        assert_eq!(
            result,
            Err(ModLoadError::Conflict {
                left_namespace: "example.left".to_string(),
                right_namespace: "example.right".to_string(),
            })
        );
    }

    #[test]
    fn load_plan_rejects_dependency_cycles() {
        let mut left = manifest("example.left");
        left.dependencies = vec![dependency("example.right", None)];
        let mut right = manifest("example.right");
        right.dependencies = vec![dependency("example.left", None)];

        let result = plan_load_order(vec![left, right]);

        assert_eq!(
            result,
            Err(ModLoadError::DependencyCycle("example.left".to_string()))
        );
    }

    #[test]
    fn legacy_manifest_json_aliases_still_decode() {
        let encoded = serde_json::json!({
            "namespace": "example.minimal_character",
            "name": "最小角色 Mod",
            "version": "0.1.0",
            "engineVersion": "0.1.0-m0",
            "loadOrder": 3,
            "dependencies": ["core.base"],
            "capabilities": ["content"]
        });

        let manifest: ModManifest = serde_json::from_value(encoded).unwrap();

        assert_eq!(manifest.dependencies, vec![dependency("core.base", None)]);
        assert_eq!(manifest.load_order, 3);
        validate_manifest(&manifest).unwrap();
    }

    fn manifest(namespace: &str) -> ModManifest {
        ModManifest {
            namespace: namespace.to_string(),
            name: namespace.to_string(),
            version: "0.1.0".to_string(),
            engine_version: "0.1.0-m0".to_string(),
            load_order: 0,
            dependencies: Vec::new(),
            conflicts: Vec::new(),
            capabilities: vec![ModCapability::Content],
        }
    }

    fn dependency(namespace: &str, version: Option<&str>) -> ModDependency {
        ModDependency {
            namespace: namespace.to_string(),
            version: version.map(ToString::to_string),
            required: true,
        }
    }

    fn namespaces(plan: &ModLoadPlan) -> Vec<&str> {
        plan.manifests
            .iter()
            .map(|manifest| manifest.namespace.as_str())
            .collect()
    }
}
