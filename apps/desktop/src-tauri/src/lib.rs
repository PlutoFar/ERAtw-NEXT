use eratw_content::ContentPackage;
use eratw_engine::{
    resource::{inspect_resource_files, plan_resource_loads, ResourceResolutionReport},
    save::{read_save, write_save_atomic, SaveEnvelope},
    EngineCommand, WorldState,
};
use eratw_mod_runtime::{
    discover_mods_for_engine, install_mod_for_engine, plan_enabled_mods_for_engine,
    plan_mod_install_for_engine, plan_mod_uninstall, uninstall_mod, DisabledMod, DiscoveredMod,
    ModDiscoveryError, ModDiscoveryIssue, ModDiscoveryReport, ModEnablement, ModEnablementPlan,
    ModInstallAction, ModInstallPlan, ModInstallReport, ModLoadError, ModManifest,
    ModUninstallPlan, ModUninstallReport, ModValidationError,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

#[derive(Debug, Serialize)]
struct SaveSlotReport {
    path: String,
    backup_path: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct DiscoveredModReport {
    root_path: String,
    manifest_path: String,
    manifest: ModManifest,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct ModDiscoveryIssueReport {
    path: String,
    kind: String,
    message: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct ModDiscoveryReportDto {
    root_path: String,
    discovered: Vec<DiscoveredModReport>,
    errors: Vec<ModDiscoveryIssueReport>,
}

#[derive(Debug, Deserialize)]
struct ModInstallRequest {
    #[serde(alias = "sourceRoot")]
    source_root: String,
    #[serde(alias = "installRoot")]
    install_root: String,
    #[serde(alias = "engineVersion")]
    engine_version: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct ModInstallPlanReport {
    source_root: String,
    install_root: String,
    target_root: String,
    staging_root: String,
    manifest_path: String,
    manifest: ModManifest,
    actions: Vec<ModInstallActionReport>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct ModInstallReportDto {
    target_root: String,
    manifest: ModManifest,
    actions: Vec<ModInstallActionReport>,
}

#[derive(Debug, Deserialize)]
struct ModUninstallRequest {
    #[serde(alias = "installRoot")]
    install_root: String,
    namespace: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct ModUninstallPlanReport {
    install_root: String,
    target_root: String,
    staging_root: String,
    namespace: String,
    actions: Vec<ModInstallActionReport>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct ModUninstallReportDto {
    namespace: String,
    target_root: String,
    actions: Vec<ModInstallActionReport>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct ModInstallActionReport {
    kind: String,
    from: Option<String>,
    path: Option<String>,
    to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ModEnablementRequest {
    manifests: Vec<ModManifest>,
    enablement: Vec<ModEnablement>,
    #[serde(alias = "engineVersion")]
    engine_version: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct DisabledModReport {
    manifest: ModManifest,
    reason: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct ModEnablementPlanReport {
    enabled: Vec<ModManifest>,
    disabled: Vec<DisabledModReport>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct ModLoadErrorReport {
    kind: String,
    message: String,
}

#[tauri::command]
fn engine_snapshot(state: tauri::State<'_, Mutex<WorldState>>) -> WorldState {
    state.lock().expect("engine state lock poisoned").clone()
}

#[tauri::command]
fn engine_dispatch(
    command: EngineCommand,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> Result<WorldState, String> {
    let mut world = state.lock().expect("engine state lock poisoned");
    world
        .apply_command(command)
        .map_err(|error| error.to_string())?;
    Ok(world.clone())
}

#[tauri::command]
fn engine_install_content_package(
    package: ContentPackage,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> Result<WorldState, String> {
    let mut world = state.lock().expect("engine state lock poisoned");
    let installed = package
        .install_into_world(world.clone())
        .map_err(|error| error.to_string())?;
    *world = installed;
    Ok(world.clone())
}

#[tauri::command]
fn engine_plan_resources(
    root: String,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> ResourceResolutionReport {
    let world = state.lock().expect("engine state lock poisoned");
    plan_resource_loads(&world.resources, root)
}

#[tauri::command]
fn engine_inspect_resources(
    root: String,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> ResourceResolutionReport {
    let world = state.lock().expect("engine state lock poisoned");
    inspect_resource_files(&world.resources, root)
}

#[tauri::command]
fn engine_discover_mods(root: String, engine_version: Option<String>) -> ModDiscoveryReportDto {
    discover_mods_for_engine(root, engine_version.as_deref()).into()
}

#[tauri::command]
fn engine_plan_mod_install(
    request: ModInstallRequest,
) -> Result<ModInstallPlanReport, ModDiscoveryIssueReport> {
    plan_mod_install_for_engine(
        request.source_root,
        request.install_root,
        request.engine_version.as_deref(),
    )
    .map(Into::into)
    .map_err(|error| ModDiscoveryIssueReport {
        path: String::new(),
        kind: mod_discovery_error_kind(&error).to_string(),
        message: error.to_string(),
    })
}

#[tauri::command]
fn engine_install_mod(
    request: ModInstallRequest,
) -> Result<ModInstallReportDto, ModDiscoveryIssueReport> {
    install_mod_for_engine(
        request.source_root,
        request.install_root,
        request.engine_version.as_deref(),
    )
    .map(Into::into)
    .map_err(|error| ModDiscoveryIssueReport {
        path: String::new(),
        kind: mod_discovery_error_kind(&error).to_string(),
        message: error.to_string(),
    })
}

#[tauri::command]
fn engine_plan_mod_uninstall(
    request: ModUninstallRequest,
) -> Result<ModUninstallPlanReport, ModDiscoveryIssueReport> {
    plan_mod_uninstall(request.install_root, request.namespace)
        .map(Into::into)
        .map_err(|error| ModDiscoveryIssueReport {
            path: String::new(),
            kind: mod_discovery_error_kind(&error).to_string(),
            message: error.to_string(),
        })
}

#[tauri::command]
fn engine_uninstall_mod(
    request: ModUninstallRequest,
) -> Result<ModUninstallReportDto, ModDiscoveryIssueReport> {
    uninstall_mod(request.install_root, request.namespace)
        .map(Into::into)
        .map_err(|error| ModDiscoveryIssueReport {
            path: String::new(),
            kind: mod_discovery_error_kind(&error).to_string(),
            message: error.to_string(),
        })
}

#[tauri::command]
fn engine_plan_enabled_mods(
    request: ModEnablementRequest,
) -> Result<ModEnablementPlanReport, ModLoadErrorReport> {
    plan_enabled_mods_for_engine(
        request.manifests,
        request.enablement,
        request.engine_version.as_deref(),
    )
    .map(Into::into)
    .map_err(Into::into)
}

#[tauri::command]
fn engine_save_preview(
    slot_id: String,
    saved_at_unix_ms: u64,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> SaveEnvelope {
    let world = state.lock().expect("engine state lock poisoned").clone();
    SaveEnvelope::new(slot_id, world, saved_at_unix_ms)
}

#[tauri::command]
fn engine_save_slot(
    app: tauri::AppHandle,
    slot_id: String,
    saved_at_unix_ms: u64,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> Result<SaveSlotReport, String> {
    let save_path = save_path_for_slot(&app, &slot_id)?;
    let world = state.lock().expect("engine state lock poisoned").clone();
    let save = SaveEnvelope::new(slot_id, world, saved_at_unix_ms);
    let report = write_save_atomic(&save_path, &save, saved_at_unix_ms)
        .map_err(|error| error.to_string())?;

    Ok(SaveSlotReport {
        path: report.path.to_string_lossy().to_string(),
        backup_path: report
            .backup_path
            .map(|path| path.to_string_lossy().to_string()),
    })
}

#[tauri::command]
fn engine_load_slot(
    app: tauri::AppHandle,
    slot_id: String,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> Result<WorldState, String> {
    let save_path = save_path_for_slot(&app, &slot_id)?;
    let save = read_save(save_path, &[]).map_err(|error| error.to_string())?;
    let mut world = state.lock().expect("engine state lock poisoned");
    *world = save.world;
    Ok(world.clone())
}

fn save_path_for_slot(app: &tauri::AppHandle, slot_id: &str) -> Result<PathBuf, String> {
    let sanitized = sanitize_slot_id(slot_id)?;
    let save_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("saves");
    Ok(save_dir.join(format!("{sanitized}.json")))
}

fn sanitize_slot_id(slot_id: &str) -> Result<String, String> {
    let slot_id = slot_id.trim();
    if slot_id.is_empty() {
        return Err("save slot id is required".to_string());
    }

    if slot_id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '_')
    {
        Ok(slot_id.to_string())
    } else {
        Err("save slot id may only contain ascii letters, numbers, '-' and '_'".to_string())
    }
}

impl From<ModDiscoveryReport> for ModDiscoveryReportDto {
    fn from(report: ModDiscoveryReport) -> Self {
        Self {
            root_path: report.root_path.to_string_lossy().to_string(),
            discovered: report.discovered.into_iter().map(Into::into).collect(),
            errors: report.errors.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<DiscoveredMod> for DiscoveredModReport {
    fn from(discovered: DiscoveredMod) -> Self {
        Self {
            root_path: discovered.root_path.to_string_lossy().to_string(),
            manifest_path: discovered.manifest_path.to_string_lossy().to_string(),
            manifest: discovered.manifest,
        }
    }
}

impl From<ModDiscoveryIssue> for ModDiscoveryIssueReport {
    fn from(issue: ModDiscoveryIssue) -> Self {
        Self {
            path: issue.path.to_string_lossy().to_string(),
            kind: mod_discovery_error_kind(&issue.error).to_string(),
            message: issue.error.to_string(),
        }
    }
}

impl From<ModInstallPlan> for ModInstallPlanReport {
    fn from(plan: ModInstallPlan) -> Self {
        Self {
            source_root: plan.source_root.to_string_lossy().to_string(),
            install_root: plan.install_root.to_string_lossy().to_string(),
            target_root: plan.target_root.to_string_lossy().to_string(),
            staging_root: plan.staging_root.to_string_lossy().to_string(),
            manifest_path: plan.manifest_path.to_string_lossy().to_string(),
            manifest: plan.manifest,
            actions: plan.actions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ModInstallReport> for ModInstallReportDto {
    fn from(report: ModInstallReport) -> Self {
        Self {
            target_root: report.target_root.to_string_lossy().to_string(),
            manifest: report.manifest,
            actions: report.actions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ModUninstallPlan> for ModUninstallPlanReport {
    fn from(plan: ModUninstallPlan) -> Self {
        Self {
            install_root: plan.install_root.to_string_lossy().to_string(),
            target_root: plan.target_root.to_string_lossy().to_string(),
            staging_root: plan.staging_root.to_string_lossy().to_string(),
            namespace: plan.namespace,
            actions: plan.actions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ModUninstallReport> for ModUninstallReportDto {
    fn from(report: ModUninstallReport) -> Self {
        Self {
            namespace: report.namespace,
            target_root: report.target_root.to_string_lossy().to_string(),
            actions: report.actions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ModInstallAction> for ModInstallActionReport {
    fn from(action: ModInstallAction) -> Self {
        match action {
            ModInstallAction::CreateDirectory { path } => Self {
                kind: "create_directory".to_string(),
                from: None,
                path: Some(path.to_string_lossy().to_string()),
                to: None,
            },
            ModInstallAction::CopyDirectory { from, to } => Self {
                kind: "copy_directory".to_string(),
                from: Some(from.to_string_lossy().to_string()),
                path: None,
                to: Some(to.to_string_lossy().to_string()),
            },
            ModInstallAction::MoveDirectory { from, to } => Self {
                kind: "move_directory".to_string(),
                from: Some(from.to_string_lossy().to_string()),
                path: None,
                to: Some(to.to_string_lossy().to_string()),
            },
            ModInstallAction::DeleteDirectory { path } => Self {
                kind: "delete_directory".to_string(),
                from: None,
                path: Some(path.to_string_lossy().to_string()),
                to: None,
            },
        }
    }
}

impl From<ModEnablementPlan> for ModEnablementPlanReport {
    fn from(plan: ModEnablementPlan) -> Self {
        Self {
            enabled: plan.enabled.manifests,
            disabled: plan.disabled.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<DisabledMod> for DisabledModReport {
    fn from(disabled: DisabledMod) -> Self {
        Self {
            manifest: disabled.manifest,
            reason: "user_disabled".to_string(),
        }
    }
}

impl From<ModLoadError> for ModLoadErrorReport {
    fn from(error: ModLoadError) -> Self {
        Self {
            kind: mod_load_error_kind(&error).to_string(),
            message: error.to_string(),
        }
    }
}

fn mod_load_error_kind(error: &ModLoadError) -> &'static str {
    match error {
        ModLoadError::Validation(error) => mod_validation_error_kind(error),
        ModLoadError::DuplicateEnablement(_) => "duplicate_enablement",
        ModLoadError::UnknownEnablement(_) => "unknown_enablement",
        ModLoadError::DuplicateNamespace(_) => "duplicate_namespace",
        ModLoadError::MissingDependency { .. } => "missing_dependency",
        ModLoadError::DependencyVersionMismatch { .. } => "dependency_version_mismatch",
        ModLoadError::Conflict { .. } => "conflict",
        ModLoadError::DependencyCycle(_) => "dependency_cycle",
    }
}

fn mod_discovery_error_kind(error: &ModDiscoveryError) -> &'static str {
    match error {
        ModDiscoveryError::Io(_) => "io",
        ModDiscoveryError::Json(_) => "json",
        ModDiscoveryError::UnsafeInstallNamespace(_) => "unsafe_install_namespace",
        ModDiscoveryError::UnsafePackageVersion(_) => "unsafe_package_version",
        ModDiscoveryError::TemplateTargetNotEmpty(_) => "template_target_not_empty",
        ModDiscoveryError::InstallTargetExists(_) => "install_target_exists",
        ModDiscoveryError::InstallTargetMissing(_) => "install_target_missing",
        ModDiscoveryError::InstallTargetNotDirectory(_) => "install_target_not_directory",
        ModDiscoveryError::Validation(error) => mod_validation_error_kind(error),
    }
}

fn mod_validation_error_kind(error: &ModValidationError) -> &'static str {
    match error {
        ModValidationError::MissingNamespace => "missing_namespace",
        ModValidationError::MissingName(_) => "missing_name",
        ModValidationError::MissingVersion(_) => "missing_version",
        ModValidationError::MissingEngineVersion(_) => "missing_engine_version",
        ModValidationError::IncompatibleEngineVersion { .. } => "incompatible_engine_version",
        ModValidationError::DuplicateDependency { .. } => "duplicate_dependency",
        ModValidationError::DuplicateConflict { .. } => "duplicate_conflict",
        ModValidationError::UnsafeCapability { .. } => "unsafe_capability",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn engine_discover_mods_returns_frontend_report() {
        let root = temp_mod_root("discover_mods_command");
        let mod_root = root.join("minimal-character");
        fs::create_dir_all(&mod_root).unwrap();
        fs::write(
            mod_root.join("manifest.json"),
            r#"{
  "namespace": "example.minimal_character",
  "name": "最小角色 Mod",
  "version": "0.1.0",
  "engine_version": "0.1.0-m0",
  "load_order": 0,
  "dependencies": [],
  "conflicts": [],
  "capabilities": ["content"]
}"#,
        )
        .unwrap();

        let report = engine_discover_mods(
            root.to_string_lossy().to_string(),
            Some("0.1.0-m0".to_string()),
        );

        assert_eq!(report.discovered.len(), 1);
        assert_eq!(report.errors, Vec::<ModDiscoveryIssueReport>::new());
        assert_eq!(
            report.discovered[0].manifest.namespace,
            "example.minimal_character"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn engine_discover_mods_reports_frontend_error_kind() {
        let root = temp_mod_root("discover_mods_error");
        let mod_root = root.join("bad");
        fs::create_dir_all(&mod_root).unwrap();
        fs::write(mod_root.join("manifest.json"), "{broken").unwrap();

        let report = engine_discover_mods(root.to_string_lossy().to_string(), None);

        assert_eq!(report.discovered.len(), 0);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0].kind, "json");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn engine_plan_mod_install_returns_frontend_plan() {
        let source_root = temp_mod_root("install_command_source");
        let install_root = temp_mod_root("install_command_target");
        fs::create_dir_all(&source_root).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.installable")).unwrap(),
        )
        .unwrap();

        let report = engine_plan_mod_install(ModInstallRequest {
            source_root: source_root.to_string_lossy().to_string(),
            install_root: install_root.to_string_lossy().to_string(),
            engine_version: Some("0.1.0-m0".to_string()),
        })
        .unwrap();

        assert_eq!(report.manifest.namespace, "example.installable");
        assert!(report.target_root.ends_with("example.installable"));
        assert!(report
            .staging_root
            .ends_with(".installing-example.installable"));
        assert_eq!(report.actions[0].kind, "create_directory");
        assert_eq!(report.actions[1].kind, "copy_directory");
        assert_eq!(report.actions[2].kind, "move_directory");

        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn engine_plan_mod_install_reports_frontend_error_kind() {
        let source_root = temp_mod_root("install_command_error");
        let install_root = temp_mod_root("install_command_target_error");
        let mut manifest = manifest("example.safe");
        manifest.namespace = "example/unsafe".to_string();
        fs::create_dir_all(&source_root).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let report = engine_plan_mod_install(ModInstallRequest {
            source_root: source_root.to_string_lossy().to_string(),
            install_root: install_root.to_string_lossy().to_string(),
            engine_version: None,
        })
        .unwrap_err();

        assert_eq!(report.kind, "unsafe_install_namespace");

        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn engine_install_mod_executes_and_returns_frontend_report() {
        let source_root = temp_mod_root("install_execute_command_source");
        let install_root = temp_mod_root("install_execute_command_target");
        fs::create_dir_all(source_root.join("assets")).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.installable")).unwrap(),
        )
        .unwrap();
        fs::write(source_root.join("assets/readme.txt"), "installed").unwrap();

        let report = engine_install_mod(ModInstallRequest {
            source_root: source_root.to_string_lossy().to_string(),
            install_root: install_root.to_string_lossy().to_string(),
            engine_version: Some("0.1.0-m0".to_string()),
        })
        .unwrap();

        assert_eq!(report.manifest.namespace, "example.installable");
        assert!(report.target_root.ends_with("example.installable"));
        assert_eq!(report.actions[2].kind, "move_directory");
        assert!(install_root
            .join("example.installable/assets/readme.txt")
            .exists());

        let _ = fs::remove_dir_all(install_root);
        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn engine_plan_mod_uninstall_returns_frontend_plan() {
        let install_root = temp_mod_root("uninstall_plan_command");

        let report = engine_plan_mod_uninstall(ModUninstallRequest {
            install_root: install_root.to_string_lossy().to_string(),
            namespace: "example.installable".to_string(),
        })
        .unwrap();

        assert_eq!(report.namespace, "example.installable");
        assert!(report.target_root.ends_with("example.installable"));
        assert!(report
            .staging_root
            .ends_with(".uninstalling-example.installable"));
        assert_eq!(report.actions[0].kind, "move_directory");
        assert_eq!(report.actions[1].kind, "delete_directory");
    }

    #[test]
    fn engine_uninstall_mod_executes_and_returns_frontend_report() {
        let install_root = temp_mod_root("uninstall_execute_command");
        let target_root = install_root.join("example.installable");
        fs::create_dir_all(target_root.join("assets")).unwrap();
        fs::write(target_root.join("assets/readme.txt"), "remove").unwrap();

        let report = engine_uninstall_mod(ModUninstallRequest {
            install_root: install_root.to_string_lossy().to_string(),
            namespace: "example.installable".to_string(),
        })
        .unwrap();

        assert_eq!(report.namespace, "example.installable");
        assert!(report.target_root.ends_with("example.installable"));
        assert_eq!(report.actions[0].kind, "move_directory");
        assert_eq!(report.actions[1].kind, "delete_directory");
        assert!(!install_root.join("example.installable").exists());
        assert!(!install_root
            .join(".uninstalling-example.installable")
            .exists());

        let _ = fs::remove_dir_all(install_root);
    }

    #[test]
    fn engine_plan_enabled_mods_returns_frontend_plan() {
        let base = manifest("core.base");
        let mut addon = manifest("example.addon");
        addon.dependencies = vec![dependency("core.base", None)];
        let optional = manifest("example.optional");

        let report = engine_plan_enabled_mods(ModEnablementRequest {
            manifests: vec![optional, addon, base],
            enablement: vec![ModEnablement {
                namespace: "example.optional".to_string(),
                enabled: false,
            }],
            engine_version: Some("0.1.0-m0".to_string()),
        })
        .unwrap();

        assert_eq!(
            report
                .enabled
                .iter()
                .map(|manifest| manifest.namespace.as_str())
                .collect::<Vec<_>>(),
            vec!["core.base", "example.addon"]
        );
        assert_eq!(report.disabled.len(), 1);
        assert_eq!(
            report.disabled[0].manifest.namespace,
            "example.optional".to_string()
        );
        assert_eq!(report.disabled[0].reason, "user_disabled");
    }

    #[test]
    fn engine_plan_enabled_mods_reports_frontend_error_kind() {
        let base = manifest("core.base");
        let mut addon = manifest("example.addon");
        addon.dependencies = vec![dependency("core.base", None)];

        let report = engine_plan_enabled_mods(ModEnablementRequest {
            manifests: vec![base, addon],
            enablement: vec![ModEnablement {
                namespace: "core.base".to_string(),
                enabled: false,
            }],
            engine_version: None,
        })
        .unwrap_err();

        assert_eq!(report.kind, "missing_dependency");
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
            capabilities: vec![eratw_mod_runtime::ModCapability::Content],
        }
    }

    fn dependency(namespace: &str, version: Option<&str>) -> eratw_mod_runtime::ModDependency {
        eratw_mod_runtime::ModDependency {
            namespace: namespace.to_string(),
            version: version.map(ToString::to_string),
            required: true,
        }
    }

    fn temp_mod_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("eratw_next_desktop_{label}_{nonce}"))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(WorldState::bootstrap_demo()))
        .invoke_handler(tauri::generate_handler![
            engine_snapshot,
            engine_dispatch,
            engine_install_content_package,
            engine_plan_resources,
            engine_inspect_resources,
            engine_discover_mods,
            engine_plan_mod_install,
            engine_install_mod,
            engine_plan_mod_uninstall,
            engine_uninstall_mod,
            engine_plan_enabled_mods,
            engine_save_preview,
            engine_save_slot,
            engine_load_slot
        ])
        .run(tauri::generate_context!())
        .expect("failed to run ERAtw-NEXT desktop app");
}
