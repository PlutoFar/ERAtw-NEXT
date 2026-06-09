use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
};
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModEnablement {
    pub namespace: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisabledMod {
    pub manifest: ModManifest,
    pub reason: DisabledModReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisabledModReason {
    UserDisabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModEnablementPlan {
    pub enabled: ModLoadPlan,
    pub disabled: Vec<DisabledMod>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredMod {
    pub root_path: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest: ModManifest,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModDiscoveryReport {
    pub root_path: PathBuf,
    pub discovered: Vec<DiscoveredMod>,
    pub errors: Vec<ModDiscoveryIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModDiscoveryIssue {
    pub path: PathBuf,
    pub error: ModDiscoveryError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModInstallPlan {
    pub source_root: PathBuf,
    pub install_root: PathBuf,
    pub target_root: PathBuf,
    pub staging_root: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest: ModManifest,
    pub actions: Vec<ModInstallAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModInstallAction {
    CreateDirectory { path: PathBuf },
    CopyDirectory { from: PathBuf, to: PathBuf },
    MoveDirectory { from: PathBuf, to: PathBuf },
    DeleteDirectory { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModInstallReport {
    pub target_root: PathBuf,
    pub manifest: ModManifest,
    pub actions: Vec<ModInstallAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModUninstallPlan {
    pub install_root: PathBuf,
    pub target_root: PathBuf,
    pub staging_root: PathBuf,
    pub namespace: String,
    pub actions: Vec<ModInstallAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModUninstallReport {
    pub namespace: String,
    pub target_root: PathBuf,
    pub actions: Vec<ModInstallAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModProjectValidationReport {
    pub root_path: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest: ModManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModPackageManifest {
    pub schema_version: String,
    pub namespace: String,
    pub version: String,
    pub manifest_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModPackageReport {
    pub source_root: PathBuf,
    pub package_root: PathBuf,
    pub content_root: PathBuf,
    pub package_manifest_path: PathBuf,
    pub manifest: ModManifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModPackageCheckReport {
    pub package_root: PathBuf,
    pub package_manifest_path: PathBuf,
    pub content_root: PathBuf,
    pub content_manifest_path: PathBuf,
    pub package_manifest: ModPackageManifest,
    pub manifest: ModManifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModTemplateOptions {
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub engine_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModTemplateReport {
    pub root_path: PathBuf,
    pub manifest_path: PathBuf,
    pub readme_path: PathBuf,
    pub character_path: PathBuf,
    pub manifest: ModManifest,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModSecurityPolicy {
    pub authorized_unsafe_capabilities: Vec<ModCapability>,
}

impl ModSecurityPolicy {
    pub fn deny_unsafe_capabilities() -> Self {
        Self::default()
    }

    pub fn with_authorized_unsafe_capabilities(mut capabilities: Vec<ModCapability>) -> Self {
        capabilities.sort();
        capabilities.dedup();
        Self {
            authorized_unsafe_capabilities: capabilities,
        }
    }

    fn authorizes_unsafe_capability(&self, capability: &ModCapability) -> bool {
        self.authorized_unsafe_capabilities
            .iter()
            .any(|authorized| authorized == capability)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
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

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ModDiscoveryError {
    #[error("mod io error: {0}")]
    Io(String),
    #[error("mod manifest json error: {0}")]
    Json(String),
    #[error("unsafe mod install namespace: {0}")]
    UnsafeInstallNamespace(String),
    #[error("unsafe mod package version: {0}")]
    UnsafePackageVersion(String),
    #[error("mod template target already exists and is not empty: {0}")]
    TemplateTargetNotEmpty(String),
    #[error("mod package manifest schema is unsupported: {0}")]
    UnsupportedPackageSchema(String),
    #[error(
        "mod package manifest does not match content manifest: expected {expected_namespace} {expected_version} found {actual_namespace} {actual_version}"
    )]
    PackageManifestMismatch {
        expected_namespace: String,
        expected_version: String,
        actual_namespace: String,
        actual_version: String,
    },
    #[error("mod install target already exists: {0}")]
    InstallTargetExists(String),
    #[error("mod install target is missing: {0}")]
    InstallTargetMissing(String),
    #[error("mod install target is not a directory: {0}")]
    InstallTargetNotDirectory(String),
    #[error(transparent)]
    Validation(#[from] ModValidationError),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ModLoadError {
    #[error(transparent)]
    Validation(#[from] ModValidationError),
    #[error("duplicate mod enablement declaration: {0}")]
    DuplicateEnablement(String),
    #[error("unknown mod enablement declaration: {0}")]
    UnknownEnablement(String),
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

impl From<io::Error> for ModDiscoveryError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<serde_json::Error> for ModDiscoveryError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error.to_string())
    }
}

pub fn validate_manifest(manifest: &ModManifest) -> Result<(), ModValidationError> {
    validate_manifest_with_policy(manifest, &ModSecurityPolicy::default())
}

pub fn validate_manifest_with_policy(
    manifest: &ModManifest,
    policy: &ModSecurityPolicy,
) -> Result<(), ModValidationError> {
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
        if is_unsafe_capability(capability) && !policy.authorizes_unsafe_capability(capability) {
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
    validate_manifest_for_engine_with_policy(
        manifest,
        engine_version,
        &ModSecurityPolicy::default(),
    )
}

pub fn validate_manifest_for_engine_with_policy(
    manifest: &ModManifest,
    engine_version: &str,
    policy: &ModSecurityPolicy,
) -> Result<(), ModValidationError> {
    validate_manifest_with_policy(manifest, policy)?;

    if manifest.engine_version != engine_version {
        return Err(ModValidationError::IncompatibleEngineVersion {
            manifest_namespace: manifest.namespace.clone(),
            expected_engine_version: manifest.engine_version.clone(),
            actual_engine_version: engine_version.to_string(),
        });
    }

    Ok(())
}

pub fn read_manifest_file(path: impl AsRef<Path>) -> Result<ModManifest, ModDiscoveryError> {
    read_manifest_file_with_policy(path, &ModSecurityPolicy::default())
}

pub fn read_manifest_file_with_policy(
    path: impl AsRef<Path>,
    policy: &ModSecurityPolicy,
) -> Result<ModManifest, ModDiscoveryError> {
    let encoded = fs::read_to_string(path)?;
    let manifest: ModManifest = serde_json::from_str(&encoded)?;
    validate_manifest_with_policy(&manifest, policy)?;
    Ok(manifest)
}

pub fn parse_mod_capability(value: &str) -> Option<ModCapability> {
    match value.trim() {
        "content" => Some(ModCapability::Content),
        "theme" => Some(ModCapability::Theme),
        "rules_extension" => Some(ModCapability::RulesExtension),
        "local_file_access" => Some(ModCapability::LocalFileAccess),
        "network_access" => Some(ModCapability::NetworkAccess),
        "system_command" => Some(ModCapability::SystemCommand),
        _ => None,
    }
}

pub fn validate_mod_project(
    root: impl AsRef<Path>,
) -> Result<ModProjectValidationReport, ModDiscoveryError> {
    validate_mod_project_for_engine(root, None)
}

pub fn validate_mod_project_for_engine(
    root: impl AsRef<Path>,
    engine_version: Option<&str>,
) -> Result<ModProjectValidationReport, ModDiscoveryError> {
    validate_mod_project_for_engine_with_policy(root, engine_version, &ModSecurityPolicy::default())
}

pub fn validate_mod_project_with_policy(
    root: impl AsRef<Path>,
    policy: &ModSecurityPolicy,
) -> Result<ModProjectValidationReport, ModDiscoveryError> {
    validate_mod_project_for_engine_with_policy(root, None, policy)
}

pub fn validate_mod_project_for_engine_with_policy(
    root: impl AsRef<Path>,
    engine_version: Option<&str>,
    policy: &ModSecurityPolicy,
) -> Result<ModProjectValidationReport, ModDiscoveryError> {
    let root = root.as_ref();
    if is_mod_staging_directory(root) {
        return Err(ModDiscoveryError::UnsafeInstallNamespace(
            root.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string(),
        ));
    }

    let manifest_path = root.join("manifest.json");
    let manifest = read_manifest_file_with_policy(&manifest_path, policy)?;
    if let Some(engine_version) = engine_version {
        validate_manifest_for_engine_with_policy(&manifest, engine_version, policy)?;
    }
    safe_install_namespace(&manifest.namespace)?;

    Ok(ModProjectValidationReport {
        root_path: root.to_path_buf(),
        manifest_path,
        manifest,
    })
}

pub fn package_mod_project(
    source_root: impl AsRef<Path>,
    output_root: impl AsRef<Path>,
) -> Result<ModPackageReport, ModDiscoveryError> {
    package_mod_project_for_engine(source_root, output_root, None)
}

pub fn package_mod_project_for_engine(
    source_root: impl AsRef<Path>,
    output_root: impl AsRef<Path>,
    engine_version: Option<&str>,
) -> Result<ModPackageReport, ModDiscoveryError> {
    package_mod_project_for_engine_with_policy(
        source_root,
        output_root,
        engine_version,
        &ModSecurityPolicy::default(),
    )
}

pub fn package_mod_project_with_policy(
    source_root: impl AsRef<Path>,
    output_root: impl AsRef<Path>,
    policy: &ModSecurityPolicy,
) -> Result<ModPackageReport, ModDiscoveryError> {
    package_mod_project_for_engine_with_policy(source_root, output_root, None, policy)
}

pub fn package_mod_project_for_engine_with_policy(
    source_root: impl AsRef<Path>,
    output_root: impl AsRef<Path>,
    engine_version: Option<&str>,
    policy: &ModSecurityPolicy,
) -> Result<ModPackageReport, ModDiscoveryError> {
    let validation =
        validate_mod_project_for_engine_with_policy(&source_root, engine_version, policy)?;
    let source_root = validation.root_path;
    let output_root = output_root.as_ref();
    let package_version = safe_package_component(&validation.manifest.version)?;
    let package_name = format!("{}-{package_version}", validation.manifest.namespace);
    let package_root = output_root.join(&package_name);
    let staging_root = output_root.join(format!(".packaging-{package_name}"));
    let staging_content_root = staging_root.join("content");
    let staging_manifest_path = staging_root.join("eratw-mod-package.json");
    let content_root = package_root.join("content");
    let package_manifest_path = package_root.join("eratw-mod-package.json");

    let source_absolute = absolute_existing_path(&source_root)?;
    let output_absolute = absolute_path(output_root)?;
    if output_absolute.starts_with(&source_absolute) {
        return Err(ModDiscoveryError::UnsafeInstallNamespace(
            output_root.to_string_lossy().to_string(),
        ));
    }

    if package_root.exists() {
        return Err(ModDiscoveryError::InstallTargetExists(
            package_root.to_string_lossy().to_string(),
        ));
    }

    fs::create_dir_all(output_root)?;
    if staging_root.exists() {
        fs::remove_dir_all(&staging_root)?;
    }

    let package_manifest = ModPackageManifest {
        schema_version: "eratw-mod-package/v0".to_string(),
        namespace: validation.manifest.namespace.clone(),
        version: validation.manifest.version.clone(),
        manifest_path: "content/manifest.json".to_string(),
    };

    let result = (|| -> Result<(), ModDiscoveryError> {
        copy_mod_project_recursively(&source_root, &staging_content_root)?;
        fs::write(
            &staging_manifest_path,
            serde_json::to_string_pretty(&package_manifest)?,
        )?;
        fs::rename(&staging_root, &package_root)?;
        Ok(())
    })();

    if result.is_err() && staging_root.exists() {
        let _ = fs::remove_dir_all(&staging_root);
    }
    result?;

    Ok(ModPackageReport {
        source_root,
        package_root,
        content_root,
        package_manifest_path,
        manifest: validation.manifest,
    })
}

pub fn scaffold_mod_template(
    root: impl AsRef<Path>,
    options: ModTemplateOptions,
) -> Result<ModTemplateReport, ModDiscoveryError> {
    let root = root.as_ref();
    let namespace = safe_install_namespace(&options.namespace)?.to_string();
    let version = safe_package_component(&options.version)?.to_string();
    let name = options.name.trim();
    let engine_version = options.engine_version.trim();
    let manifest_name = if name.is_empty() {
        namespace.clone()
    } else {
        name.to_string()
    };

    let manifest = ModManifest {
        namespace,
        name: manifest_name,
        version,
        engine_version: engine_version.to_string(),
        load_order: 0,
        dependencies: Vec::new(),
        conflicts: Vec::new(),
        capabilities: vec![ModCapability::Content],
    };
    validate_manifest(&manifest)?;

    if root.exists() && (!root.is_dir() || fs::read_dir(root)?.next().is_some()) {
        return Err(ModDiscoveryError::TemplateTargetNotEmpty(
            root.to_string_lossy().to_string(),
        ));
    }

    let manifest_path = root.join("manifest.json");
    let readme_path = root.join("README.md");
    let content_root = root.join("content");
    let character_path = content_root.join("character.json");

    fs::create_dir_all(&content_root)?;
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    fs::write(&readme_path, template_readme(&manifest))?;
    fs::write(&character_path, template_character_json(&manifest))?;

    Ok(ModTemplateReport {
        root_path: root.to_path_buf(),
        manifest_path,
        readme_path,
        character_path,
        manifest,
    })
}

pub fn check_mod_package(
    package_root: impl AsRef<Path>,
) -> Result<ModPackageCheckReport, ModDiscoveryError> {
    check_mod_package_for_engine(package_root, None)
}

pub fn check_mod_package_for_engine(
    package_root: impl AsRef<Path>,
    engine_version: Option<&str>,
) -> Result<ModPackageCheckReport, ModDiscoveryError> {
    check_mod_package_for_engine_with_policy(
        package_root,
        engine_version,
        &ModSecurityPolicy::default(),
    )
}

pub fn check_mod_package_with_policy(
    package_root: impl AsRef<Path>,
    policy: &ModSecurityPolicy,
) -> Result<ModPackageCheckReport, ModDiscoveryError> {
    check_mod_package_for_engine_with_policy(package_root, None, policy)
}

pub fn check_mod_package_for_engine_with_policy(
    package_root: impl AsRef<Path>,
    engine_version: Option<&str>,
    policy: &ModSecurityPolicy,
) -> Result<ModPackageCheckReport, ModDiscoveryError> {
    let package_root = package_root.as_ref();
    let package_manifest_path = package_root.join("eratw-mod-package.json");
    let encoded = fs::read_to_string(&package_manifest_path)?;
    let package_manifest: ModPackageManifest = serde_json::from_str(&encoded)?;
    if package_manifest.schema_version != "eratw-mod-package/v0" {
        return Err(ModDiscoveryError::UnsupportedPackageSchema(
            package_manifest.schema_version,
        ));
    }

    let manifest_relative_path = safe_package_manifest_path(&package_manifest.manifest_path)?;
    let content_manifest_path = package_root.join(manifest_relative_path);
    let content_root = content_manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| package_root.to_path_buf());
    let manifest = read_manifest_file_with_policy(&content_manifest_path, policy)?;
    if let Some(engine_version) = engine_version {
        validate_manifest_for_engine_with_policy(&manifest, engine_version, policy)?;
    }
    safe_install_namespace(&manifest.namespace)?;

    if package_manifest.namespace != manifest.namespace
        || package_manifest.version != manifest.version
    {
        return Err(ModDiscoveryError::PackageManifestMismatch {
            expected_namespace: package_manifest.namespace,
            expected_version: package_manifest.version,
            actual_namespace: manifest.namespace,
            actual_version: manifest.version,
        });
    }

    Ok(ModPackageCheckReport {
        package_root: package_root.to_path_buf(),
        package_manifest_path,
        content_root,
        content_manifest_path,
        package_manifest,
        manifest,
    })
}

pub fn plan_mod_install(
    source_root: impl AsRef<Path>,
    install_root: impl AsRef<Path>,
) -> Result<ModInstallPlan, ModDiscoveryError> {
    plan_mod_install_for_engine(source_root, install_root, None)
}

pub fn plan_mod_install_for_engine(
    source_root: impl AsRef<Path>,
    install_root: impl AsRef<Path>,
    engine_version: Option<&str>,
) -> Result<ModInstallPlan, ModDiscoveryError> {
    plan_mod_install_for_engine_with_policy(
        source_root,
        install_root,
        engine_version,
        &ModSecurityPolicy::default(),
    )
}

pub fn plan_mod_install_with_policy(
    source_root: impl AsRef<Path>,
    install_root: impl AsRef<Path>,
    policy: &ModSecurityPolicy,
) -> Result<ModInstallPlan, ModDiscoveryError> {
    plan_mod_install_for_engine_with_policy(source_root, install_root, None, policy)
}

pub fn plan_mod_install_for_engine_with_policy(
    source_root: impl AsRef<Path>,
    install_root: impl AsRef<Path>,
    engine_version: Option<&str>,
    policy: &ModSecurityPolicy,
) -> Result<ModInstallPlan, ModDiscoveryError> {
    let source_root = source_root.as_ref();
    let install_root = install_root.as_ref();
    let manifest_path = source_root.join("manifest.json");
    let manifest = read_manifest_file_with_policy(&manifest_path, policy).and_then(|manifest| {
        if let Some(engine_version) = engine_version {
            validate_manifest_for_engine_with_policy(&manifest, engine_version, policy)?;
        }
        Ok(manifest)
    })?;
    let namespace = safe_install_namespace(&manifest.namespace)?;
    let target_root = install_root.join(namespace);
    let staging_root = install_root.join(format!(".installing-{namespace}"));

    Ok(ModInstallPlan {
        source_root: source_root.to_path_buf(),
        install_root: install_root.to_path_buf(),
        target_root: target_root.clone(),
        staging_root: staging_root.clone(),
        manifest_path,
        manifest,
        actions: vec![
            ModInstallAction::CreateDirectory {
                path: install_root.to_path_buf(),
            },
            ModInstallAction::CopyDirectory {
                from: source_root.to_path_buf(),
                to: staging_root.clone(),
            },
            ModInstallAction::MoveDirectory {
                from: staging_root,
                to: target_root,
            },
        ],
    })
}

pub fn install_mod(
    source_root: impl AsRef<Path>,
    install_root: impl AsRef<Path>,
) -> Result<ModInstallReport, ModDiscoveryError> {
    install_mod_for_engine(source_root, install_root, None)
}

pub fn install_mod_for_engine(
    source_root: impl AsRef<Path>,
    install_root: impl AsRef<Path>,
    engine_version: Option<&str>,
) -> Result<ModInstallReport, ModDiscoveryError> {
    install_mod_for_engine_with_policy(
        source_root,
        install_root,
        engine_version,
        &ModSecurityPolicy::default(),
    )
}

pub fn install_mod_with_policy(
    source_root: impl AsRef<Path>,
    install_root: impl AsRef<Path>,
    policy: &ModSecurityPolicy,
) -> Result<ModInstallReport, ModDiscoveryError> {
    install_mod_for_engine_with_policy(source_root, install_root, None, policy)
}

pub fn install_mod_for_engine_with_policy(
    source_root: impl AsRef<Path>,
    install_root: impl AsRef<Path>,
    engine_version: Option<&str>,
    policy: &ModSecurityPolicy,
) -> Result<ModInstallReport, ModDiscoveryError> {
    let plan =
        plan_mod_install_for_engine_with_policy(source_root, install_root, engine_version, policy)?;
    execute_mod_install_plan(plan)
}

pub fn install_mod_package(
    package_root: impl AsRef<Path>,
    install_root: impl AsRef<Path>,
) -> Result<ModInstallReport, ModDiscoveryError> {
    install_mod_package_for_engine(package_root, install_root, None)
}

pub fn install_mod_package_for_engine(
    package_root: impl AsRef<Path>,
    install_root: impl AsRef<Path>,
    engine_version: Option<&str>,
) -> Result<ModInstallReport, ModDiscoveryError> {
    install_mod_package_for_engine_with_policy(
        package_root,
        install_root,
        engine_version,
        &ModSecurityPolicy::default(),
    )
}

pub fn install_mod_package_with_policy(
    package_root: impl AsRef<Path>,
    install_root: impl AsRef<Path>,
    policy: &ModSecurityPolicy,
) -> Result<ModInstallReport, ModDiscoveryError> {
    install_mod_package_for_engine_with_policy(package_root, install_root, None, policy)
}

pub fn install_mod_package_for_engine_with_policy(
    package_root: impl AsRef<Path>,
    install_root: impl AsRef<Path>,
    engine_version: Option<&str>,
    policy: &ModSecurityPolicy,
) -> Result<ModInstallReport, ModDiscoveryError> {
    let package = check_mod_package_for_engine_with_policy(package_root, engine_version, policy)?;
    let plan = plan_mod_install_for_engine_with_policy(
        package.content_root,
        install_root,
        engine_version,
        policy,
    )?;
    execute_mod_install_plan(plan)
}

pub fn execute_mod_install_plan(
    plan: ModInstallPlan,
) -> Result<ModInstallReport, ModDiscoveryError> {
    if plan.target_root.exists() {
        return Err(ModDiscoveryError::InstallTargetExists(
            plan.target_root.to_string_lossy().to_string(),
        ));
    }

    if plan.staging_root.exists() {
        fs::remove_dir_all(&plan.staging_root)?;
    }

    let result = (|| -> Result<(), ModDiscoveryError> {
        fs::create_dir_all(&plan.install_root)?;
        copy_directory_recursively(&plan.source_root, &plan.staging_root)?;
        fs::rename(&plan.staging_root, &plan.target_root)?;
        Ok(())
    })();

    if let Err(error) = result {
        let _ = fs::remove_dir_all(&plan.staging_root);
        return Err(error);
    }

    Ok(ModInstallReport {
        target_root: plan.target_root,
        manifest: plan.manifest,
        actions: plan.actions,
    })
}

pub fn plan_mod_uninstall(
    install_root: impl AsRef<Path>,
    namespace: impl AsRef<str>,
) -> Result<ModUninstallPlan, ModDiscoveryError> {
    let install_root = install_root.as_ref();
    let namespace = safe_install_namespace(namespace.as_ref())?;
    let target_root = install_root.join(namespace);
    let staging_root = install_root.join(format!(".uninstalling-{namespace}"));

    Ok(ModUninstallPlan {
        install_root: install_root.to_path_buf(),
        target_root: target_root.clone(),
        staging_root: staging_root.clone(),
        namespace: namespace.to_string(),
        actions: vec![
            ModInstallAction::MoveDirectory {
                from: target_root,
                to: staging_root.clone(),
            },
            ModInstallAction::DeleteDirectory { path: staging_root },
        ],
    })
}

pub fn uninstall_mod(
    install_root: impl AsRef<Path>,
    namespace: impl AsRef<str>,
) -> Result<ModUninstallReport, ModDiscoveryError> {
    let plan = plan_mod_uninstall(install_root, namespace)?;
    execute_mod_uninstall_plan(plan)
}

pub fn execute_mod_uninstall_plan(
    plan: ModUninstallPlan,
) -> Result<ModUninstallReport, ModDiscoveryError> {
    if !plan.target_root.exists() {
        return Err(ModDiscoveryError::InstallTargetMissing(
            plan.target_root.to_string_lossy().to_string(),
        ));
    }
    if !plan.target_root.is_dir() {
        return Err(ModDiscoveryError::InstallTargetNotDirectory(
            plan.target_root.to_string_lossy().to_string(),
        ));
    }

    if plan.staging_root.exists() {
        fs::remove_dir_all(&plan.staging_root)?;
    }

    fs::rename(&plan.target_root, &plan.staging_root)?;
    fs::remove_dir_all(&plan.staging_root)?;

    Ok(ModUninstallReport {
        namespace: plan.namespace,
        target_root: plan.target_root,
        actions: plan.actions,
    })
}

pub fn discover_mods(root: impl AsRef<Path>) -> ModDiscoveryReport {
    discover_mods_for_engine(root, None)
}

pub fn discover_mods_for_engine(
    root: impl AsRef<Path>,
    engine_version: Option<&str>,
) -> ModDiscoveryReport {
    discover_mods_for_engine_with_policy(root, engine_version, &ModSecurityPolicy::default())
}

pub fn discover_mods_with_policy(
    root: impl AsRef<Path>,
    policy: &ModSecurityPolicy,
) -> ModDiscoveryReport {
    discover_mods_for_engine_with_policy(root, None, policy)
}

pub fn discover_mods_for_engine_with_policy(
    root: impl AsRef<Path>,
    engine_version: Option<&str>,
    policy: &ModSecurityPolicy,
) -> ModDiscoveryReport {
    let root = root.as_ref();
    let mut report = ModDiscoveryReport {
        root_path: root.to_path_buf(),
        discovered: Vec::new(),
        errors: Vec::new(),
    };

    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) => {
            report.errors.push(ModDiscoveryIssue {
                path: root.to_path_buf(),
                error: error.into(),
            });
            return report;
        }
    };

    for entry in entries {
        let Ok(entry) = entry else {
            report.errors.push(ModDiscoveryIssue {
                path: root.to_path_buf(),
                error: ModDiscoveryError::Io("failed to read mod directory entry".to_string()),
            });
            continue;
        };
        let mod_root = entry.path();
        if !mod_root.is_dir() {
            continue;
        }
        if is_mod_staging_directory(&mod_root) {
            continue;
        }

        let manifest_path = mod_root.join("manifest.json");
        match read_manifest_file_with_policy(&manifest_path, policy).and_then(|manifest| {
            if let Some(engine_version) = engine_version {
                validate_manifest_for_engine_with_policy(&manifest, engine_version, policy)?;
            }
            Ok(manifest)
        }) {
            Ok(manifest) => report.discovered.push(DiscoveredMod {
                root_path: mod_root,
                manifest_path,
                manifest,
            }),
            Err(error) => report.errors.push(ModDiscoveryIssue {
                path: manifest_path,
                error,
            }),
        }
    }

    report.discovered.sort_by(|left, right| {
        left.manifest
            .namespace
            .cmp(&right.manifest.namespace)
            .then_with(|| left.root_path.cmp(&right.root_path))
    });
    report
        .errors
        .sort_by(|left, right| left.path.cmp(&right.path));
    report
}

pub fn plan_load_order(manifests: Vec<ModManifest>) -> Result<ModLoadPlan, ModLoadError> {
    plan_load_order_for_engine(manifests, None)
}

pub fn plan_load_order_for_engine(
    manifests: Vec<ModManifest>,
    engine_version: Option<&str>,
) -> Result<ModLoadPlan, ModLoadError> {
    plan_load_order_for_engine_with_policy(manifests, engine_version, &ModSecurityPolicy::default())
}

pub fn plan_load_order_with_policy(
    manifests: Vec<ModManifest>,
    policy: &ModSecurityPolicy,
) -> Result<ModLoadPlan, ModLoadError> {
    plan_load_order_for_engine_with_policy(manifests, None, policy)
}

pub fn plan_load_order_for_engine_with_policy(
    manifests: Vec<ModManifest>,
    engine_version: Option<&str>,
    policy: &ModSecurityPolicy,
) -> Result<ModLoadPlan, ModLoadError> {
    let mut by_namespace = BTreeMap::new();
    for manifest in manifests {
        if let Some(engine_version) = engine_version {
            validate_manifest_for_engine_with_policy(&manifest, engine_version, policy)?;
        } else {
            validate_manifest_with_policy(&manifest, policy)?;
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

pub fn plan_enabled_mods(
    manifests: Vec<ModManifest>,
    enablement: Vec<ModEnablement>,
) -> Result<ModEnablementPlan, ModLoadError> {
    plan_enabled_mods_for_engine(manifests, enablement, None)
}

pub fn plan_enabled_mods_for_engine(
    manifests: Vec<ModManifest>,
    enablement: Vec<ModEnablement>,
    engine_version: Option<&str>,
) -> Result<ModEnablementPlan, ModLoadError> {
    plan_enabled_mods_for_engine_with_policy(
        manifests,
        enablement,
        engine_version,
        &ModSecurityPolicy::default(),
    )
}

pub fn plan_enabled_mods_with_policy(
    manifests: Vec<ModManifest>,
    enablement: Vec<ModEnablement>,
    policy: &ModSecurityPolicy,
) -> Result<ModEnablementPlan, ModLoadError> {
    plan_enabled_mods_for_engine_with_policy(manifests, enablement, None, policy)
}

pub fn plan_enabled_mods_for_engine_with_policy(
    manifests: Vec<ModManifest>,
    enablement: Vec<ModEnablement>,
    engine_version: Option<&str>,
    policy: &ModSecurityPolicy,
) -> Result<ModEnablementPlan, ModLoadError> {
    let requested = requested_enablement(enablement)?;
    let manifest_namespaces = manifest_namespaces(&manifests)?;
    for namespace in requested.keys() {
        if !manifest_namespaces.contains(namespace) {
            return Err(ModLoadError::UnknownEnablement(namespace.clone()));
        }
    }

    let mut enabled = Vec::new();
    let mut disabled = Vec::new();
    for manifest in manifests {
        if requested.get(&manifest.namespace).copied().unwrap_or(true) {
            enabled.push(manifest);
        } else {
            disabled.push(DisabledMod {
                manifest,
                reason: DisabledModReason::UserDisabled,
            });
        }
    }

    let enabled = plan_load_order_for_engine_with_policy(enabled, engine_version, policy)?;
    disabled.sort_by(|left, right| {
        left.manifest
            .namespace
            .cmp(&right.manifest.namespace)
            .then_with(|| left.manifest.load_order.cmp(&right.manifest.load_order))
    });

    Ok(ModEnablementPlan { enabled, disabled })
}

fn manifest_namespaces(manifests: &[ModManifest]) -> Result<BTreeSet<String>, ModLoadError> {
    let mut namespaces = BTreeSet::new();
    for manifest in manifests {
        if !namespaces.insert(manifest.namespace.clone()) {
            return Err(ModLoadError::DuplicateNamespace(manifest.namespace.clone()));
        }
    }
    Ok(namespaces)
}

fn requested_enablement(
    enablement: Vec<ModEnablement>,
) -> Result<BTreeMap<String, bool>, ModLoadError> {
    let mut requested = BTreeMap::new();
    for entry in enablement {
        if requested
            .insert(entry.namespace.clone(), entry.enabled)
            .is_some()
        {
            return Err(ModLoadError::DuplicateEnablement(entry.namespace));
        }
    }
    Ok(requested)
}

fn safe_install_namespace(namespace: &str) -> Result<&str, ModDiscoveryError> {
    let namespace = namespace.trim();
    if namespace.is_empty()
        || namespace == "."
        || namespace == ".."
        || namespace.contains('/')
        || namespace.contains('\\')
        || namespace.contains(':')
    {
        return Err(ModDiscoveryError::UnsafeInstallNamespace(
            namespace.to_string(),
        ));
    }
    Ok(namespace)
}

fn safe_package_component(component: &str) -> Result<&str, ModDiscoveryError> {
    let component = component.trim();
    if component.is_empty()
        || component == "."
        || component == ".."
        || component.contains('/')
        || component.contains('\\')
        || component.contains(':')
    {
        return Err(ModDiscoveryError::UnsafePackageVersion(
            component.to_string(),
        ));
    }
    Ok(component)
}

fn safe_package_manifest_path(path: &str) -> Result<PathBuf, ModDiscoveryError> {
    let normalized = path.replace('\\', "/");
    let parts = normalized
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect::<Vec<_>>();

    if normalized.trim().is_empty()
        || normalized.starts_with('/')
        || normalized.contains(':')
        || parts.is_empty()
        || parts.iter().any(|part| *part == "..")
    {
        return Err(ModDiscoveryError::UnsafeInstallNamespace(path.to_string()));
    }

    let mut safe_path = PathBuf::new();
    for part in parts {
        safe_path.push(part);
    }
    Ok(safe_path)
}

fn is_mod_staging_directory(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with(".installing-") || name.starts_with(".uninstalling-"))
}

fn absolute_existing_path(path: &Path) -> io::Result<PathBuf> {
    path.canonicalize()
}

fn absolute_path(path: &Path) -> io::Result<PathBuf> {
    if path.exists() {
        return path.canonicalize();
    }

    let current_dir = std::env::current_dir()?;
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };
    let mut missing = Vec::<OsString>::new();
    let mut cursor = absolute.as_path();
    while !cursor.exists() {
        let Some(name) = cursor.file_name() else {
            break;
        };
        missing.push(name.to_os_string());
        let Some(parent) = cursor.parent() else {
            break;
        };
        cursor = parent;
    }

    let mut resolved = cursor.canonicalize()?;
    for name in missing.iter().rev() {
        resolved.push(name);
    }
    Ok(resolved)
}

fn copy_directory_recursively(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = to.join(entry.file_name());
        let metadata = entry.file_type()?;
        if metadata.is_dir() {
            copy_directory_recursively(&source_path, &target_path)?;
        } else if metadata.is_file() {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

fn copy_mod_project_recursively(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let source_path = entry.path();
        let file_name = entry.file_name();
        let target_path = to.join(&file_name);
        let metadata = entry.file_type()?;
        if metadata.is_dir() {
            if is_mod_package_excluded_directory(&source_path) {
                continue;
            }
            copy_mod_project_recursively(&source_path, &target_path)?;
        } else if metadata.is_file() {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

fn is_mod_package_excluded_directory(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            is_mod_staging_directory(path)
                || matches!(name, ".git" | "node_modules" | "target" | "dist" | "build")
                || name.starts_with(".packaging-")
        })
}

fn template_readme(manifest: &ModManifest) -> String {
    format!(
        "# {}\n\nNamespace: `{}`\nVersion: `{}`\nEngine: `{}`\n\nThis is an ERAtw-NEXT Mod template generated by `eratw-mod new`.\n",
        manifest.name, manifest.namespace, manifest.version, manifest.engine_version
    )
}

fn template_character_json(manifest: &ModManifest) -> String {
    let character_id = format!("{}.demo", manifest.namespace);
    format!(
        r#"{{
  "schemaVersion": "character/v0",
  "id": "{character_id}",
  "displayName": "{name} 示例角色",
  "initialLocationId": "school_gate",
  "resourceRefs": []
}}
"#,
        name = manifest.name
    )
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
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

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
    fn unsafe_capabilities_can_be_authorized_explicitly() {
        let mut manifest = manifest("example.unsafe");
        manifest.capabilities = vec![ModCapability::NetworkAccess, ModCapability::SystemCommand];
        let policy = ModSecurityPolicy::with_authorized_unsafe_capabilities(vec![
            ModCapability::NetworkAccess,
            ModCapability::SystemCommand,
        ]);

        validate_manifest_with_policy(&manifest, &policy).unwrap();
        validate_manifest_for_engine_with_policy(&manifest, "0.1.0-m0", &policy).unwrap();
        assert_eq!(
            parse_mod_capability("network_access"),
            Some(ModCapability::NetworkAccess)
        );
        assert_eq!(parse_mod_capability("nope"), None);
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
    fn enablement_plan_keeps_disabled_mods_out_of_load_order() {
        let base = manifest("core.base");
        let mut addon = manifest("example.addon");
        addon.dependencies = vec![dependency("core.base", None)];
        let optional = manifest("example.optional");

        let plan = plan_enabled_mods(
            vec![optional, addon, base],
            vec![enablement("example.optional", false)],
        )
        .unwrap();

        assert_eq!(
            namespaces(&plan.enabled),
            vec!["core.base", "example.addon"]
        );
        assert_eq!(disabled_namespaces(&plan), vec!["example.optional"]);
        assert_eq!(plan.disabled[0].reason, DisabledModReason::UserDisabled);
    }

    #[test]
    fn enablement_plan_rejects_disabled_required_dependency() {
        let base = manifest("core.base");
        let mut addon = manifest("example.addon");
        addon.dependencies = vec![dependency("core.base", None)];

        let result = plan_enabled_mods(vec![base, addon], vec![enablement("core.base", false)]);

        assert_eq!(
            result,
            Err(ModLoadError::MissingDependency {
                manifest_namespace: "example.addon".to_string(),
                dependency_namespace: "core.base".to_string(),
            })
        );
    }

    #[test]
    fn enablement_plan_allows_disabled_optional_dependency() {
        let optional = manifest("core.optional");
        let mut addon = manifest("example.addon");
        let mut dependency = dependency("core.optional", None);
        dependency.required = false;
        addon.dependencies = vec![dependency];

        let plan = plan_enabled_mods(
            vec![optional, addon],
            vec![enablement("core.optional", false)],
        )
        .unwrap();

        assert_eq!(namespaces(&plan.enabled), vec!["example.addon"]);
        assert_eq!(disabled_namespaces(&plan), vec!["core.optional"]);
    }

    #[test]
    fn enablement_plan_rejects_duplicate_and_unknown_entries() {
        let manifest = manifest("example.character");

        let duplicate = plan_enabled_mods(
            vec![manifest.clone()],
            vec![
                enablement("example.character", true),
                enablement("example.character", false),
            ],
        );
        let unknown = plan_enabled_mods(vec![manifest], vec![enablement("example.missing", false)]);

        assert_eq!(
            duplicate,
            Err(ModLoadError::DuplicateEnablement(
                "example.character".to_string()
            ))
        );
        assert_eq!(
            unknown,
            Err(ModLoadError::UnknownEnablement(
                "example.missing".to_string()
            ))
        );
    }

    #[test]
    fn enablement_plan_rejects_duplicate_manifest_namespaces_before_disabling() {
        let left = manifest("example.duplicate");
        let right = manifest("example.duplicate");

        let result = plan_enabled_mods(
            vec![left, right],
            vec![enablement("example.duplicate", false)],
        );

        assert_eq!(
            result,
            Err(ModLoadError::DuplicateNamespace(
                "example.duplicate".to_string()
            ))
        );
    }

    #[test]
    fn enablement_plan_validates_enabled_manifests_against_engine_version() {
        let manifest = manifest("example.character");

        let result = plan_enabled_mods_for_engine(vec![manifest], Vec::new(), Some("9.9.9"));

        assert_eq!(
            result,
            Err(ModLoadError::Validation(
                ModValidationError::IncompatibleEngineVersion {
                    manifest_namespace: "example.character".to_string(),
                    expected_engine_version: "0.1.0-m0".to_string(),
                    actual_engine_version: "9.9.9".to_string(),
                }
            ))
        );
    }

    #[test]
    fn enablement_plan_uses_explicit_capability_authorization() {
        let mut manifest = manifest("example.unsafe");
        manifest.capabilities = vec![ModCapability::LocalFileAccess];
        let policy = ModSecurityPolicy::with_authorized_unsafe_capabilities(vec![
            ModCapability::LocalFileAccess,
        ]);

        let denied = plan_enabled_mods(vec![manifest.clone()], Vec::new());
        let allowed = plan_enabled_mods_for_engine_with_policy(
            vec![manifest],
            Vec::new(),
            Some("0.1.0-m0"),
            &policy,
        )
        .unwrap();

        assert!(matches!(
            denied,
            Err(ModLoadError::Validation(
                ModValidationError::UnsafeCapability { .. }
            ))
        ));
        assert_eq!(namespaces(&allowed.enabled), vec!["example.unsafe"]);
    }

    #[test]
    fn validate_mod_project_reads_manifest_and_engine_version() {
        let source_root = temp_mod_dir("project_validate");
        fs::create_dir_all(&source_root).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.project")).unwrap(),
        )
        .unwrap();

        let report = validate_mod_project_for_engine(&source_root, Some("0.1.0-m0")).unwrap();

        assert_eq!(report.root_path, source_root);
        assert_eq!(report.manifest_path, report.root_path.join("manifest.json"));
        assert_eq!(report.manifest.namespace, "example.project");

        let _ = fs::remove_dir_all(report.root_path);
    }

    #[test]
    fn validate_mod_project_uses_explicit_capability_authorization() {
        let source_root = temp_mod_dir("project_validate_policy");
        fs::create_dir_all(&source_root).unwrap();
        let mut manifest = manifest("example.policy");
        manifest.capabilities = vec![ModCapability::NetworkAccess];
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        let policy = ModSecurityPolicy::with_authorized_unsafe_capabilities(vec![
            ModCapability::NetworkAccess,
        ]);

        let denied = validate_mod_project(&source_root);
        let allowed =
            validate_mod_project_for_engine_with_policy(&source_root, Some("0.1.0-m0"), &policy)
                .unwrap();

        assert!(matches!(
            denied,
            Err(ModDiscoveryError::Validation(
                ModValidationError::UnsafeCapability { .. }
            ))
        ));
        assert_eq!(allowed.manifest.namespace, "example.policy");

        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn validate_mod_project_rejects_incompatible_engine_version() {
        let source_root = temp_mod_dir("project_validate_engine");
        fs::create_dir_all(&source_root).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.project")).unwrap(),
        )
        .unwrap();

        let result = validate_mod_project_for_engine(&source_root, Some("9.9.9"));

        assert!(matches!(
            result,
            Err(ModDiscoveryError::Validation(
                ModValidationError::IncompatibleEngineVersion { .. }
            ))
        ));

        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn package_mod_project_copies_project_to_package_directory() {
        let source_root = temp_mod_dir("project_package_source");
        let output_root = temp_mod_dir("project_package_output");
        fs::create_dir_all(source_root.join("assets")).unwrap();
        fs::create_dir_all(source_root.join(".git")).unwrap();
        fs::create_dir_all(source_root.join("dist")).unwrap();
        fs::create_dir_all(source_root.join(".installing-example.package")).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.package")).unwrap(),
        )
        .unwrap();
        fs::write(source_root.join("assets/readme.txt"), "package me").unwrap();
        fs::write(source_root.join(".git/config"), "ignored").unwrap();
        fs::write(source_root.join("dist/generated.txt"), "ignored").unwrap();
        fs::write(
            source_root.join(".installing-example.package/manifest.json"),
            "ignored",
        )
        .unwrap();

        let report =
            package_mod_project_for_engine(&source_root, &output_root, Some("0.1.0-m0")).unwrap();

        assert_eq!(report.source_root, source_root);
        assert_eq!(
            report.package_root,
            output_root.join("example.package-0.1.0")
        );
        assert_eq!(
            fs::read_to_string(report.content_root.join("assets/readme.txt")).unwrap(),
            "package me"
        );
        assert!(!report.content_root.join(".git").exists());
        assert!(!report.content_root.join("dist").exists());
        assert!(!report
            .content_root
            .join(".installing-example.package")
            .exists());
        let package_manifest: ModPackageManifest =
            serde_json::from_str(&fs::read_to_string(&report.package_manifest_path).unwrap())
                .unwrap();
        assert_eq!(package_manifest.schema_version, "eratw-mod-package/v0");
        assert_eq!(package_manifest.namespace, "example.package");
        assert_eq!(package_manifest.version, "0.1.0");
        assert_eq!(package_manifest.manifest_path, "content/manifest.json");
        assert!(!output_root
            .join(".packaging-example.package-0.1.0")
            .exists());

        let _ = fs::remove_dir_all(output_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn package_mod_project_rejects_output_inside_source_root() {
        let source_root = temp_mod_dir("project_package_recursive");
        fs::create_dir_all(&source_root).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.package")).unwrap(),
        )
        .unwrap();

        let result = package_mod_project(&source_root, source_root.join("dist"));

        assert!(matches!(
            result,
            Err(ModDiscoveryError::UnsafeInstallNamespace(_))
        ));
        assert!(!source_root.join("dist").exists());

        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn package_mod_project_rejects_unsafe_version_path_component() {
        let source_root = temp_mod_dir("project_package_unsafe_version");
        let output_root = temp_mod_dir("project_package_unsafe_version_output");
        let mut manifest = manifest("example.package");
        manifest.version = "../escape".to_string();
        fs::create_dir_all(&source_root).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let result = package_mod_project(&source_root, &output_root);

        assert_eq!(
            result,
            Err(ModDiscoveryError::UnsafePackageVersion(
                "../escape".to_string()
            ))
        );
        assert!(!output_root.exists());

        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn check_mod_package_validates_package_manifest_and_content() {
        let source_root = temp_mod_dir("package_check_source");
        let output_root = temp_mod_dir("package_check_output");
        fs::create_dir_all(&source_root).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.package")).unwrap(),
        )
        .unwrap();
        let package =
            package_mod_project_for_engine(&source_root, &output_root, Some("0.1.0-m0")).unwrap();

        let report = check_mod_package_for_engine(&package.package_root, Some("0.1.0-m0")).unwrap();

        assert_eq!(report.package_root, package.package_root);
        assert_eq!(report.package_manifest.namespace, "example.package");
        assert_eq!(report.manifest.namespace, "example.package");
        assert_eq!(report.content_root, report.package_root.join("content"));
        assert_eq!(
            report.content_manifest_path,
            report.package_root.join("content/manifest.json")
        );

        let _ = fs::remove_dir_all(output_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn check_mod_package_rejects_bad_schema_mismatch_and_unsafe_path() {
        let package_root = temp_mod_dir("package_check_bad");
        fs::create_dir_all(package_root.join("content")).unwrap();
        fs::write(
            package_root.join("content/manifest.json"),
            serde_json::to_string_pretty(&manifest("example.package")).unwrap(),
        )
        .unwrap();
        fs::write(
            package_root.join("eratw-mod-package.json"),
            serde_json::to_string_pretty(&ModPackageManifest {
                schema_version: "wrong".to_string(),
                namespace: "example.package".to_string(),
                version: "0.1.0".to_string(),
                manifest_path: "content/manifest.json".to_string(),
            })
            .unwrap(),
        )
        .unwrap();

        let bad_schema = check_mod_package(&package_root);
        fs::write(
            package_root.join("eratw-mod-package.json"),
            serde_json::to_string_pretty(&ModPackageManifest {
                schema_version: "eratw-mod-package/v0".to_string(),
                namespace: "example.other".to_string(),
                version: "0.1.0".to_string(),
                manifest_path: "content/manifest.json".to_string(),
            })
            .unwrap(),
        )
        .unwrap();
        let mismatch = check_mod_package(&package_root);
        fs::write(
            package_root.join("eratw-mod-package.json"),
            serde_json::to_string_pretty(&ModPackageManifest {
                schema_version: "eratw-mod-package/v0".to_string(),
                namespace: "example.package".to_string(),
                version: "0.1.0".to_string(),
                manifest_path: "../manifest.json".to_string(),
            })
            .unwrap(),
        )
        .unwrap();
        let unsafe_path = check_mod_package(&package_root);

        assert_eq!(
            bad_schema,
            Err(ModDiscoveryError::UnsupportedPackageSchema(
                "wrong".to_string()
            ))
        );
        assert!(matches!(
            mismatch,
            Err(ModDiscoveryError::PackageManifestMismatch { .. })
        ));
        assert_eq!(
            unsafe_path,
            Err(ModDiscoveryError::UnsafeInstallNamespace(
                "../manifest.json".to_string()
            ))
        );

        let _ = fs::remove_dir_all(package_root);
    }

    #[test]
    fn check_mod_package_rejects_incompatible_engine_version() {
        let source_root = temp_mod_dir("package_check_engine_source");
        let output_root = temp_mod_dir("package_check_engine_output");
        fs::create_dir_all(&source_root).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.package")).unwrap(),
        )
        .unwrap();
        let package = package_mod_project(&source_root, &output_root).unwrap();

        let result = check_mod_package_for_engine(&package.package_root, Some("9.9.9"));

        assert!(matches!(
            result,
            Err(ModDiscoveryError::Validation(
                ModValidationError::IncompatibleEngineVersion { .. }
            ))
        ));

        let _ = fs::remove_dir_all(output_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn scaffold_mod_template_writes_valid_minimal_project() {
        let root = temp_mod_dir("template_new");

        let report = scaffold_mod_template(
            &root,
            ModTemplateOptions {
                namespace: "example.template".to_string(),
                name: "模板 Mod".to_string(),
                version: "0.1.0".to_string(),
                engine_version: "0.1.0-m0".to_string(),
            },
        )
        .unwrap();

        assert_eq!(report.root_path, root);
        assert_eq!(report.manifest.namespace, "example.template");
        assert!(report.manifest_path.exists());
        assert!(report.readme_path.exists());
        assert!(report.character_path.exists());
        assert!(fs::read_to_string(&report.character_path)
            .unwrap()
            .contains("example.template.demo"));

        let validation =
            validate_mod_project_for_engine(&report.root_path, Some("0.1.0-m0")).unwrap();
        assert_eq!(validation.manifest.namespace, "example.template");

        let _ = fs::remove_dir_all(report.root_path);
    }

    #[test]
    fn scaffold_mod_template_rejects_non_empty_target() {
        let root = temp_mod_dir("template_not_empty");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("keep.txt"), "existing").unwrap();

        let result = scaffold_mod_template(
            &root,
            ModTemplateOptions {
                namespace: "example.template".to_string(),
                name: "Template".to_string(),
                version: "0.1.0".to_string(),
                engine_version: "0.1.0-m0".to_string(),
            },
        );

        assert!(matches!(
            result,
            Err(ModDiscoveryError::TemplateTargetNotEmpty(_))
        ));
        assert_eq!(
            fs::read_to_string(root.join("keep.txt")).unwrap(),
            "existing"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scaffold_mod_template_rejects_file_target() {
        let root = temp_mod_dir("template_file_target");
        fs::write(&root, "existing file").unwrap();

        let result = scaffold_mod_template(
            &root,
            ModTemplateOptions {
                namespace: "example.template".to_string(),
                name: "Template".to_string(),
                version: "0.1.0".to_string(),
                engine_version: "0.1.0-m0".to_string(),
            },
        );

        assert!(matches!(
            result,
            Err(ModDiscoveryError::TemplateTargetNotEmpty(_))
        ));
        assert_eq!(fs::read_to_string(&root).unwrap(), "existing file");

        let _ = fs::remove_file(root);
    }

    #[test]
    fn scaffold_mod_template_rejects_unsafe_namespace_and_version() {
        let root = temp_mod_dir("template_unsafe");

        let unsafe_namespace = scaffold_mod_template(
            &root,
            ModTemplateOptions {
                namespace: "../outside".to_string(),
                name: "Template".to_string(),
                version: "0.1.0".to_string(),
                engine_version: "0.1.0-m0".to_string(),
            },
        );
        let unsafe_version = scaffold_mod_template(
            &root,
            ModTemplateOptions {
                namespace: "example.template".to_string(),
                name: "Template".to_string(),
                version: "../escape".to_string(),
                engine_version: "0.1.0-m0".to_string(),
            },
        );

        assert!(matches!(
            unsafe_namespace,
            Err(ModDiscoveryError::UnsafeInstallNamespace(_))
        ));
        assert_eq!(
            unsafe_version,
            Err(ModDiscoveryError::UnsafePackageVersion(
                "../escape".to_string()
            ))
        );
        assert!(!root.exists());
    }

    #[test]
    fn mod_install_plan_reads_manifest_and_targets_namespace_directory() {
        let source_root = temp_mod_dir("install_source");
        let install_root = temp_mod_dir("install_root");
        fs::create_dir_all(&source_root).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.installable")).unwrap(),
        )
        .unwrap();

        let plan =
            plan_mod_install_for_engine(&source_root, &install_root, Some("0.1.0-m0")).unwrap();

        assert_eq!(plan.source_root, source_root);
        assert_eq!(plan.install_root, install_root);
        assert_eq!(
            plan.target_root,
            plan.install_root.join("example.installable")
        );
        assert_eq!(
            plan.staging_root,
            plan.install_root.join(".installing-example.installable")
        );
        assert_eq!(
            plan.actions,
            vec![
                ModInstallAction::CreateDirectory {
                    path: plan.install_root.clone(),
                },
                ModInstallAction::CopyDirectory {
                    from: plan.source_root.clone(),
                    to: plan.staging_root.clone(),
                },
                ModInstallAction::MoveDirectory {
                    from: plan.staging_root.clone(),
                    to: plan.target_root.clone(),
                },
            ]
        );

        let _ = fs::remove_dir_all(plan.source_root);
    }

    #[test]
    fn mod_install_plan_rejects_incompatible_engine_version() {
        let source_root = temp_mod_dir("install_engine");
        let install_root = temp_mod_dir("install_root_engine");
        fs::create_dir_all(&source_root).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.installable")).unwrap(),
        )
        .unwrap();

        let result = plan_mod_install_for_engine(&source_root, &install_root, Some("9.9.9"));

        assert!(matches!(
            result,
            Err(ModDiscoveryError::Validation(
                ModValidationError::IncompatibleEngineVersion { .. }
            ))
        ));

        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn mod_install_plan_rejects_unsafe_namespace_targets() {
        let source_root = temp_mod_dir("install_unsafe");
        let install_root = temp_mod_dir("install_root_unsafe");
        let mut manifest = manifest("example.safe");
        manifest.namespace = "example/unsafe".to_string();
        fs::create_dir_all(&source_root).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let result = plan_mod_install(&source_root, &install_root);

        assert_eq!(
            result,
            Err(ModDiscoveryError::UnsafeInstallNamespace(
                "example/unsafe".to_string()
            ))
        );

        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn mod_install_plan_reports_missing_manifest_as_io_error() {
        let source_root = temp_mod_dir("install_missing_manifest");
        let install_root = temp_mod_dir("install_root_missing_manifest");

        let result = plan_mod_install(&source_root, &install_root);

        assert!(matches!(result, Err(ModDiscoveryError::Io(_))));
    }

    #[test]
    fn install_mod_copies_directory_through_staging() {
        let source_root = temp_mod_dir("install_execute_source");
        let install_root = temp_mod_dir("install_execute_root");
        fs::create_dir_all(source_root.join("assets/nested")).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.installable")).unwrap(),
        )
        .unwrap();
        fs::write(source_root.join("assets/nested/readme.txt"), "copied").unwrap();

        let report = install_mod_for_engine(&source_root, &install_root, Some("0.1.0-m0")).unwrap();

        assert_eq!(report.manifest.namespace, "example.installable");
        assert!(report.target_root.join("manifest.json").exists());
        assert_eq!(
            fs::read_to_string(report.target_root.join("assets/nested/readme.txt")).unwrap(),
            "copied"
        );
        assert!(!install_root
            .join(".installing-example.installable")
            .exists());
        assert!(matches!(
            report.actions[2],
            ModInstallAction::MoveDirectory { .. }
        ));

        let _ = fs::remove_dir_all(install_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn install_mod_package_checks_and_installs_content_root() {
        let source_root = temp_mod_dir("install_package_source");
        let package_output_root = temp_mod_dir("install_package_output");
        let install_root = temp_mod_dir("install_package_root");
        fs::create_dir_all(source_root.join("assets")).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.package")).unwrap(),
        )
        .unwrap();
        fs::write(source_root.join("assets/readme.txt"), "from package").unwrap();
        let package =
            package_mod_project_for_engine(&source_root, &package_output_root, Some("0.1.0-m0"))
                .unwrap();

        let report =
            install_mod_package_for_engine(&package.package_root, &install_root, Some("0.1.0-m0"))
                .unwrap();

        assert_eq!(report.manifest.namespace, "example.package");
        assert_eq!(report.target_root, install_root.join("example.package"));
        assert_eq!(
            fs::read_to_string(install_root.join("example.package/assets/readme.txt")).unwrap(),
            "from package"
        );
        assert!(!install_root.join(".installing-example.package").exists());

        let _ = fs::remove_dir_all(install_root);
        let _ = fs::remove_dir_all(package_output_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn install_mod_package_rejects_existing_target_without_overwrite() {
        let source_root = temp_mod_dir("install_package_existing_source");
        let package_output_root = temp_mod_dir("install_package_existing_output");
        let install_root = temp_mod_dir("install_package_existing_root");
        let target_root = install_root.join("example.package");
        fs::create_dir_all(&source_root).unwrap();
        fs::create_dir_all(&target_root).unwrap();
        fs::write(target_root.join("keep.txt"), "existing").unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.package")).unwrap(),
        )
        .unwrap();
        let package = package_mod_project(&source_root, &package_output_root).unwrap();

        let result = install_mod_package(&package.package_root, &install_root);

        assert!(matches!(
            result,
            Err(ModDiscoveryError::InstallTargetExists(_))
        ));
        assert_eq!(
            fs::read_to_string(target_root.join("keep.txt")).unwrap(),
            "existing"
        );

        let _ = fs::remove_dir_all(install_root);
        let _ = fs::remove_dir_all(package_output_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn install_mod_package_rejects_bad_package_before_install_root_is_created() {
        let package_root = temp_mod_dir("install_package_bad");
        let install_root = temp_mod_dir("install_package_bad_root");
        fs::create_dir_all(package_root.join("content")).unwrap();
        fs::write(
            package_root.join("content/manifest.json"),
            serde_json::to_string_pretty(&manifest("example.package")).unwrap(),
        )
        .unwrap();
        fs::write(
            package_root.join("eratw-mod-package.json"),
            serde_json::to_string_pretty(&ModPackageManifest {
                schema_version: "bad".to_string(),
                namespace: "example.package".to_string(),
                version: "0.1.0".to_string(),
                manifest_path: "content/manifest.json".to_string(),
            })
            .unwrap(),
        )
        .unwrap();

        let result = install_mod_package(&package_root, &install_root);

        assert_eq!(
            result,
            Err(ModDiscoveryError::UnsupportedPackageSchema(
                "bad".to_string()
            ))
        );
        assert!(!install_root.exists());

        let _ = fs::remove_dir_all(package_root);
    }

    #[test]
    fn install_mod_package_uses_explicit_capability_authorization() {
        let source_root = temp_mod_dir("install_package_policy_source");
        let package_output_root = temp_mod_dir("install_package_policy_output");
        let install_root = temp_mod_dir("install_package_policy_root");
        fs::create_dir_all(&source_root).unwrap();
        let mut manifest = manifest("example.policy");
        manifest.capabilities = vec![ModCapability::SystemCommand];
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        let policy = ModSecurityPolicy::with_authorized_unsafe_capabilities(vec![
            ModCapability::SystemCommand,
        ]);
        let package = package_mod_project_for_engine_with_policy(
            &source_root,
            &package_output_root,
            Some("0.1.0-m0"),
            &policy,
        )
        .unwrap();

        let denied = install_mod_package(&package.package_root, &install_root);
        let allowed = install_mod_package_for_engine_with_policy(
            &package.package_root,
            &install_root,
            Some("0.1.0-m0"),
            &policy,
        )
        .unwrap();

        assert!(matches!(
            denied,
            Err(ModDiscoveryError::Validation(
                ModValidationError::UnsafeCapability { .. }
            ))
        ));
        assert_eq!(allowed.manifest.namespace, "example.policy");
        assert!(install_root.join("example.policy/manifest.json").exists());

        let _ = fs::remove_dir_all(install_root);
        let _ = fs::remove_dir_all(package_output_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn install_mod_rejects_existing_target_without_overwrite() {
        let source_root = temp_mod_dir("install_existing_source");
        let install_root = temp_mod_dir("install_existing_root");
        let target_root = install_root.join("example.installable");
        fs::create_dir_all(&source_root).unwrap();
        fs::create_dir_all(&target_root).unwrap();
        fs::write(target_root.join("keep.txt"), "existing").unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.installable")).unwrap(),
        )
        .unwrap();

        let result = install_mod(&source_root, &install_root);

        assert!(matches!(
            result,
            Err(ModDiscoveryError::InstallTargetExists(_))
        ));
        assert_eq!(
            fs::read_to_string(target_root.join("keep.txt")).unwrap(),
            "existing"
        );

        let _ = fs::remove_dir_all(install_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn install_mod_cleans_staging_when_copy_fails() {
        let source_root = temp_mod_dir("install_fail_source");
        let install_root = temp_mod_dir("install_fail_root");
        let staging_root = install_root.join(".installing-example.installable");
        let target_root = install_root.join("example.installable");
        let plan = ModInstallPlan {
            source_root: source_root.clone(),
            install_root: install_root.clone(),
            target_root,
            staging_root: staging_root.clone(),
            manifest_path: source_root.join("manifest.json"),
            manifest: manifest("example.installable"),
            actions: Vec::new(),
        };

        let result = execute_mod_install_plan(plan);

        assert!(matches!(result, Err(ModDiscoveryError::Io(_))));
        assert!(!staging_root.exists());

        let _ = fs::remove_dir_all(install_root);
    }

    #[test]
    fn mod_uninstall_plan_targets_namespace_directory() {
        let install_root = temp_mod_dir("uninstall_plan");

        let plan = plan_mod_uninstall(&install_root, "example.installable").unwrap();

        assert_eq!(plan.install_root, install_root);
        assert_eq!(
            plan.target_root,
            plan.install_root.join("example.installable")
        );
        assert_eq!(
            plan.staging_root,
            plan.install_root.join(".uninstalling-example.installable")
        );
        assert_eq!(
            plan.actions,
            vec![
                ModInstallAction::MoveDirectory {
                    from: plan.target_root.clone(),
                    to: plan.staging_root.clone(),
                },
                ModInstallAction::DeleteDirectory {
                    path: plan.staging_root.clone(),
                },
            ]
        );
    }

    #[test]
    fn uninstall_mod_moves_to_staging_then_deletes() {
        let install_root = temp_mod_dir("uninstall_execute");
        let target_root = install_root.join("example.installable");
        fs::create_dir_all(target_root.join("assets")).unwrap();
        fs::write(target_root.join("assets/readme.txt"), "remove").unwrap();

        let report = uninstall_mod(&install_root, "example.installable").unwrap();

        assert_eq!(report.namespace, "example.installable");
        assert_eq!(report.target_root, target_root);
        assert!(!report.target_root.exists());
        assert!(!install_root
            .join(".uninstalling-example.installable")
            .exists());
        assert!(matches!(
            report.actions[0],
            ModInstallAction::MoveDirectory { .. }
        ));
        assert!(matches!(
            report.actions[1],
            ModInstallAction::DeleteDirectory { .. }
        ));

        let _ = fs::remove_dir_all(install_root);
    }

    #[test]
    fn uninstall_mod_rejects_missing_target() {
        let install_root = temp_mod_dir("uninstall_missing");

        let result = uninstall_mod(&install_root, "example.missing");

        assert!(matches!(
            result,
            Err(ModDiscoveryError::InstallTargetMissing(_))
        ));
    }

    #[test]
    fn uninstall_mod_rejects_non_directory_target() {
        let install_root = temp_mod_dir("uninstall_file");
        fs::create_dir_all(&install_root).unwrap();
        fs::write(install_root.join("example.file"), b"not a directory").unwrap();

        let result = uninstall_mod(&install_root, "example.file");

        assert!(matches!(
            result,
            Err(ModDiscoveryError::InstallTargetNotDirectory(_))
        ));
        assert!(install_root.join("example.file").is_file());

        let _ = fs::remove_dir_all(install_root);
    }

    #[test]
    fn uninstall_mod_rejects_unsafe_namespace() {
        let install_root = temp_mod_dir("uninstall_unsafe");

        let result = uninstall_mod(&install_root, "../outside");

        assert_eq!(
            result,
            Err(ModDiscoveryError::UnsafeInstallNamespace(
                "../outside".to_string()
            ))
        );
    }

    #[test]
    fn read_manifest_file_validates_manifest_json() {
        let dir = temp_mod_dir("read_manifest");
        let path = dir.join("manifest.json");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            &path,
            serde_json::to_string_pretty(&manifest("example.file")).unwrap(),
        )
        .unwrap();

        let manifest = read_manifest_file(&path).unwrap();

        assert_eq!(manifest.namespace, "example.file");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn discover_mods_reports_good_and_bad_manifest_dirs() {
        let root = temp_mod_dir("discover");
        let good_dir = root.join("good");
        let bad_json_dir = root.join("bad-json");
        let unsafe_dir = root.join("unsafe");
        let installing_dir = root.join(".installing-example.pending");
        let uninstalling_dir = root.join(".uninstalling-example.removing");
        fs::create_dir_all(&good_dir).unwrap();
        fs::create_dir_all(&bad_json_dir).unwrap();
        fs::create_dir_all(&unsafe_dir).unwrap();
        fs::create_dir_all(&installing_dir).unwrap();
        fs::create_dir_all(&uninstalling_dir).unwrap();
        fs::write(
            good_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.good")).unwrap(),
        )
        .unwrap();
        fs::write(bad_json_dir.join("manifest.json"), b"{broken").unwrap();
        let mut unsafe_manifest = manifest("example.unsafe");
        unsafe_manifest.capabilities = vec![ModCapability::SystemCommand];
        fs::write(
            unsafe_dir.join("manifest.json"),
            serde_json::to_string_pretty(&unsafe_manifest).unwrap(),
        )
        .unwrap();
        fs::write(
            installing_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.pending")).unwrap(),
        )
        .unwrap();
        fs::write(
            uninstalling_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.removing")).unwrap(),
        )
        .unwrap();
        fs::write(root.join("loose-file.txt"), b"ignored").unwrap();

        let report = discover_mods(&root);

        assert_eq!(
            report
                .discovered
                .iter()
                .map(|entry| entry.manifest.namespace.as_str())
                .collect::<Vec<_>>(),
            vec!["example.good"]
        );
        assert_eq!(report.errors.len(), 2);
        assert!(matches!(report.errors[0].error, ModDiscoveryError::Json(_)));
        assert!(matches!(
            report.errors[1].error,
            ModDiscoveryError::Validation(ModValidationError::UnsafeCapability { .. })
        ));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn discover_mods_for_engine_reports_incompatible_manifests() {
        let root = temp_mod_dir("discover_engine");
        let mod_dir = root.join("addon");
        fs::create_dir_all(&mod_dir).unwrap();
        fs::write(
            mod_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.addon")).unwrap(),
        )
        .unwrap();

        let report = discover_mods_for_engine(&root, Some("9.9.9"));

        assert!(report.discovered.is_empty());
        assert!(matches!(
            report.errors[0].error,
            ModDiscoveryError::Validation(ModValidationError::IncompatibleEngineVersion { .. })
        ));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn discover_mods_reports_unreadable_root_without_panicking() {
        let root = temp_mod_dir("missing_root");

        let report = discover_mods(&root);

        assert!(report.discovered.is_empty());
        assert_eq!(report.errors.len(), 1);
        assert!(matches!(report.errors[0].error, ModDiscoveryError::Io(_)));
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

    fn enablement(namespace: &str, enabled: bool) -> ModEnablement {
        ModEnablement {
            namespace: namespace.to_string(),
            enabled,
        }
    }

    fn namespaces(plan: &ModLoadPlan) -> Vec<&str> {
        plan.manifests
            .iter()
            .map(|manifest| manifest.namespace.as_str())
            .collect()
    }

    fn disabled_namespaces(plan: &ModEnablementPlan) -> Vec<&str> {
        plan.disabled
            .iter()
            .map(|disabled| disabled.manifest.namespace.as_str())
            .collect()
    }

    fn temp_mod_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("eratw_next_mod_{label}_{nonce}"))
    }
}
