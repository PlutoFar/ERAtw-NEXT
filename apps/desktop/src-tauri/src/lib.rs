use eratw_content::{
    preflight_content_package_install, preflight_content_package_install_with_registry,
    ContentInstallPreflightReport, ContentPackage,
};
use eratw_engine::{
    resource::{
        audit_resource_publication_with_options, cache_resource_loads_with_options,
        clean_resource_cache_with_options, inspect_resource_files_with_options,
        plan_resource_loads_with_options, preflight_resource_loads_with_options,
        ResourceCacheCleanReport, ResourceCacheReport, ResourcePlanningOptions,
        ResourcePreflightReport, ResourcePublishReport, ResourceResolutionReport,
    },
    save::{
        preflight_save_against_registry, read_save, recover_save_from_latest_backup,
        write_save_atomic, SaveEnvelope, SaveModDependency, SaveReadReport,
        SaveRecoveryReport as EngineSaveRecoveryReport, SaveValidationReport,
    },
    EngineCommand, WorldState,
};
use eratw_mod_runtime::{
    discover_mods_for_engine_with_policy, install_mod_for_engine_with_policy,
    install_mod_package_for_engine_with_policy, parse_mod_capability,
    plan_enabled_mods_for_engine_with_policy, plan_mod_install_for_engine_with_policy,
    plan_mod_uninstall, preflight_mod_package_install_for_engine_with_policy, uninstall_mod,
    DisabledMod, DiscoveredMod, ModCapability, ModDiscoveryError, ModDiscoveryIssue,
    ModDiscoveryReport, ModEnablement, ModEnablementPlan, ModInstallAction, ModInstallPlan,
    ModInstallPreflightIssue, ModInstallPreflightIssueSeverity, ModInstallPreflightReport,
    ModInstallReport, ModLoadError, ModManifest, ModRegistry, ModSecurityPolicy, ModUninstallPlan,
    ModUninstallReport, ModValidationError,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

#[derive(Debug, Serialize)]
struct SaveSlotReport {
    path: String,
    backup_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct SaveRecoveryReport {
    path: String,
    recovered_from: String,
    failed_primary_backup_path: Option<String>,
    save: SaveEnvelope,
}

#[derive(Debug, Deserialize)]
struct ContentPackageInstallRequest {
    package: ContentPackage,
    registry: Option<ModRegistry>,
}

#[derive(Debug, Deserialize)]
struct SavePreflightRequest {
    #[serde(alias = "slotId")]
    slot_id: String,
    #[serde(alias = "modRoot")]
    mod_root: String,
    enablement: Vec<ModEnablement>,
    #[serde(alias = "engineVersion")]
    engine_version: Option<String>,
    #[serde(default, alias = "authorizedUnsafeCapabilities")]
    authorized_unsafe_capabilities: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct SavePreflightReport {
    slot_id: String,
    path: String,
    ready: bool,
    registry: ModRegistry,
    discovery: ModDiscoveryReportDto,
    validation: SaveValidationReportDto,
    save: SaveEnvelope,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct SaveValidationReportDto {
    missing_required_mods: Vec<SaveModDependency>,
    incompatible_schema: Option<u32>,
    engine_version_mismatch: bool,
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
    #[serde(default, alias = "authorizedUnsafeCapabilities")]
    authorized_unsafe_capabilities: Vec<String>,
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

#[derive(Debug, PartialEq, Eq, Serialize)]
struct ModInstallPreflightReportDto {
    source_root: String,
    content_root: Option<String>,
    install_root: String,
    target_root: Option<String>,
    staging_root: Option<String>,
    manifest: Option<ModManifest>,
    ready: bool,
    issues: Vec<ModInstallPreflightIssueReport>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct ModInstallPreflightIssueReport {
    severity: String,
    path: String,
    kind: String,
    message: String,
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
    #[serde(default, alias = "authorizedUnsafeCapabilities")]
    authorized_unsafe_capabilities: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ModEnablementSettingsRequest {
    #[serde(alias = "installRoot")]
    install_root: String,
    enablement: Vec<ModEnablement>,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct ModEnablementSettings {
    #[serde(default)]
    install_roots: BTreeMap<String, Vec<ModEnablement>>,
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
    request: ContentPackageInstallRequest,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> Result<WorldState, String> {
    let mut world = state.lock().expect("engine state lock poisoned");
    let installed = if let Some(registry) = &request.registry {
        request
            .package
            .install_into_world_with_registry(world.clone(), registry)
    } else {
        request.package.install_into_world(world.clone())
    }
    .map_err(|error| error.to_string())?;
    *world = installed;
    Ok(world.clone())
}

#[tauri::command]
fn engine_preflight_content_package_install(
    request: ContentPackageInstallRequest,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> ContentInstallPreflightReport {
    let world = state.lock().expect("engine state lock poisoned");
    if let Some(registry) = &request.registry {
        preflight_content_package_install_with_registry(&request.package, &world, registry)
    } else {
        preflight_content_package_install(&request.package, &world)
    }
}

#[tauri::command]
fn engine_plan_resources(
    root: String,
    low_spec: Option<bool>,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> ResourceResolutionReport {
    let world = state.lock().expect("engine state lock poisoned");
    plan_resource_loads_with_options(
        &world.resources,
        root,
        ResourcePlanningOptions {
            low_spec: low_spec.unwrap_or_default(),
        },
    )
}

#[tauri::command]
fn engine_inspect_resources(
    root: String,
    low_spec: Option<bool>,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> ResourceResolutionReport {
    let world = state.lock().expect("engine state lock poisoned");
    inspect_resource_files_with_options(
        &world.resources,
        root,
        ResourcePlanningOptions {
            low_spec: low_spec.unwrap_or_default(),
        },
    )
}

#[tauri::command]
fn engine_preflight_resources(
    root: String,
    low_spec: Option<bool>,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> ResourcePreflightReport {
    let world = state.lock().expect("engine state lock poisoned");
    preflight_resource_loads_with_options(
        &world.resources,
        root,
        ResourcePlanningOptions {
            low_spec: low_spec.unwrap_or_default(),
        },
    )
}

#[tauri::command]
fn engine_audit_resource_publication(
    root: String,
    low_spec: Option<bool>,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> ResourcePublishReport {
    let world = state.lock().expect("engine state lock poisoned");
    audit_resource_publication_with_options(
        &world.resources,
        root,
        ResourcePlanningOptions {
            low_spec: low_spec.unwrap_or_default(),
        },
    )
}

#[tauri::command]
fn engine_cache_resources(
    root: String,
    low_spec: Option<bool>,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> ResourceCacheReport {
    let world = state.lock().expect("engine state lock poisoned");
    cache_resource_loads_with_options(
        &world.resources,
        root,
        ResourcePlanningOptions {
            low_spec: low_spec.unwrap_or_default(),
        },
    )
}

#[tauri::command]
fn engine_clean_resource_cache(
    root: String,
    low_spec: Option<bool>,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> ResourceCacheCleanReport {
    let world = state.lock().expect("engine state lock poisoned");
    clean_resource_cache_with_options(
        &world.resources,
        root,
        ResourcePlanningOptions {
            low_spec: low_spec.unwrap_or_default(),
        },
    )
}

#[tauri::command]
fn engine_discover_mods(
    root: String,
    engine_version: Option<String>,
    authorized_unsafe_capabilities: Option<Vec<String>>,
) -> Result<ModDiscoveryReportDto, ModDiscoveryIssueReport> {
    let authorized_unsafe_capabilities = authorized_unsafe_capabilities.unwrap_or_default();
    let policy = security_policy(&authorized_unsafe_capabilities)?;
    Ok(discover_mods_for_engine_with_policy(root, engine_version.as_deref(), &policy).into())
}

#[tauri::command]
fn engine_plan_mod_install(
    request: ModInstallRequest,
) -> Result<ModInstallPlanReport, ModDiscoveryIssueReport> {
    let policy = security_policy(&request.authorized_unsafe_capabilities)?;
    plan_mod_install_for_engine_with_policy(
        request.source_root,
        request.install_root,
        request.engine_version.as_deref(),
        &policy,
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
    let policy = security_policy(&request.authorized_unsafe_capabilities)?;
    install_mod_for_engine_with_policy(
        request.source_root,
        request.install_root,
        request.engine_version.as_deref(),
        &policy,
    )
    .map(Into::into)
    .map_err(|error| ModDiscoveryIssueReport {
        path: String::new(),
        kind: mod_discovery_error_kind(&error).to_string(),
        message: error.to_string(),
    })
}

#[tauri::command]
fn engine_preflight_mod_package_install(
    request: ModInstallRequest,
) -> Result<ModInstallPreflightReportDto, ModDiscoveryIssueReport> {
    let policy = security_policy(&request.authorized_unsafe_capabilities)?;
    Ok(preflight_mod_package_install_for_engine_with_policy(
        request.source_root,
        request.install_root,
        request.engine_version.as_deref(),
        &policy,
    )
    .into())
}

#[tauri::command]
fn engine_install_mod_package(
    request: ModInstallRequest,
) -> Result<ModInstallReportDto, ModDiscoveryIssueReport> {
    let policy = security_policy(&request.authorized_unsafe_capabilities)?;
    install_mod_package_for_engine_with_policy(
        request.source_root,
        request.install_root,
        request.engine_version.as_deref(),
        &policy,
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
    let policy = security_policy_for_load(&request.authorized_unsafe_capabilities)?;
    plan_enabled_mods_for_engine_with_policy(
        request.manifests,
        request.enablement,
        request.engine_version.as_deref(),
        &policy,
    )
    .map(Into::into)
    .map_err(Into::into)
}

#[tauri::command]
fn engine_load_mod_enablement(
    app: tauri::AppHandle,
    install_root: String,
) -> Result<Vec<ModEnablement>, String> {
    let install_root = normalized_mod_enablement_install_root(&install_root)?;
    let settings = read_mod_enablement_settings_path(&mod_enablement_settings_path(&app)?)?;
    Ok(settings
        .install_roots
        .get(&install_root)
        .cloned()
        .unwrap_or_default())
}

#[tauri::command]
fn engine_save_mod_enablement(
    app: tauri::AppHandle,
    request: ModEnablementSettingsRequest,
) -> Result<Vec<ModEnablement>, String> {
    let install_root = normalized_mod_enablement_install_root(&request.install_root)?;
    let path = mod_enablement_settings_path(&app)?;
    let mut settings = read_mod_enablement_settings_path(&path)?;
    settings
        .install_roots
        .insert(install_root, request.enablement.clone());
    write_mod_enablement_settings_path(&path, &settings)?;
    Ok(request.enablement)
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

#[tauri::command]
fn engine_recover_slot(
    app: tauri::AppHandle,
    slot_id: String,
    recovered_at_unix_ms: u64,
    state: tauri::State<'_, Mutex<WorldState>>,
) -> Result<SaveRecoveryReport, String> {
    let save_path = save_path_for_slot(&app, &slot_id)?;
    let report = recover_save_from_latest_backup(&save_path, &[], recovered_at_unix_ms)
        .map_err(|error| error.to_string())?;
    let mut world = state.lock().expect("engine state lock poisoned");
    *world = report.save.world.clone();
    Ok(report.into())
}

#[tauri::command]
fn engine_preflight_load_slot(
    app: tauri::AppHandle,
    request: SavePreflightRequest,
) -> Result<SavePreflightReport, ModLoadErrorReport> {
    let save_path = save_path_for_slot(&app, &request.slot_id).map_err(save_preflight_error)?;
    preflight_load_slot_path(save_path, request)
}

fn preflight_load_slot_path(
    save_path: PathBuf,
    request: SavePreflightRequest,
) -> Result<SavePreflightReport, ModLoadErrorReport> {
    let policy = security_policy_for_load(&request.authorized_unsafe_capabilities)?;
    let discovery = discover_mods_for_engine_with_policy(
        &request.mod_root,
        request.engine_version.as_deref(),
        &policy,
    );
    let enablement = plan_enabled_mods_for_engine_with_policy(
        discovery
            .discovered
            .iter()
            .map(|discovered| discovered.manifest.clone())
            .collect(),
        request.enablement,
        request.engine_version.as_deref(),
        &policy,
    )?;
    let registry = eratw_mod_runtime::mod_registry_from_enablement_plan(&enablement);
    let enabled_mods = save_dependencies_from_registry(&registry);
    let read_report = preflight_save_against_registry(&save_path, &enabled_mods)
        .map_err(|error| save_preflight_error(error.to_string()))?;

    Ok(SavePreflightReport::from_parts(
        request.slot_id,
        save_path,
        registry,
        discovery,
        read_report,
    ))
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

fn mod_enablement_settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("settings")
        .join("mod_enablement.json"))
}

fn normalized_mod_enablement_install_root(install_root: &str) -> Result<String, String> {
    let install_root = install_root.trim();
    if install_root.is_empty() {
        Err("mod install root is required".to_string())
    } else {
        Ok(install_root.to_string())
    }
}

fn read_mod_enablement_settings_path(path: &Path) -> Result<ModEnablementSettings, String> {
    let encoded = match fs::read(path) {
        Ok(encoded) => encoded,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ModEnablementSettings::default());
        }
        Err(error) => return Err(error.to_string()),
    };

    serde_json::from_slice(&encoded).map_err(|error| error.to_string())
}

fn write_mod_enablement_settings_path(
    path: &Path,
    settings: &ModEnablementSettings,
) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "mod enablement settings path has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let encoded = serde_json::to_vec_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(path, encoded).map_err(|error| error.to_string())
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

fn security_policy(
    authorized_unsafe_capabilities: &[String],
) -> Result<ModSecurityPolicy, ModDiscoveryIssueReport> {
    parse_security_policy(authorized_unsafe_capabilities)
        .ok_or_else(|| unknown_capability_issue(authorized_unsafe_capabilities))
}

fn security_policy_for_load(
    authorized_unsafe_capabilities: &[String],
) -> Result<ModSecurityPolicy, ModLoadErrorReport> {
    parse_security_policy(authorized_unsafe_capabilities).ok_or_else(|| ModLoadErrorReport {
        kind: "unknown_capability".to_string(),
        message: format!(
            "unknown mod capability authorization: {}",
            authorized_unsafe_capabilities
                .iter()
                .find(|capability| parse_mod_capability(capability).is_none())
                .cloned()
                .unwrap_or_default()
        ),
    })
}

fn save_preflight_error(message: String) -> ModLoadErrorReport {
    ModLoadErrorReport {
        kind: "save".to_string(),
        message,
    }
}

fn save_dependencies_from_registry(registry: &ModRegistry) -> Vec<SaveModDependency> {
    registry
        .enabled
        .iter()
        .map(|entry| SaveModDependency {
            namespace: entry.namespace.clone(),
            version: entry.version.clone(),
            required: true,
        })
        .collect()
}

fn parse_security_policy(authorized_unsafe_capabilities: &[String]) -> Option<ModSecurityPolicy> {
    let capabilities = authorized_unsafe_capabilities
        .iter()
        .map(|capability| parse_mod_capability(capability))
        .collect::<Option<Vec<ModCapability>>>()?;
    Some(ModSecurityPolicy::with_authorized_unsafe_capabilities(
        capabilities,
    ))
}

fn unknown_capability_issue(authorized_unsafe_capabilities: &[String]) -> ModDiscoveryIssueReport {
    let capability = authorized_unsafe_capabilities
        .iter()
        .find(|capability| parse_mod_capability(capability).is_none())
        .cloned()
        .unwrap_or_default();
    ModDiscoveryIssueReport {
        path: String::new(),
        kind: "unknown_capability".to_string(),
        message: format!("unknown mod capability authorization: {capability}"),
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

impl SavePreflightReport {
    fn from_parts(
        slot_id: String,
        path: PathBuf,
        registry: ModRegistry,
        discovery: ModDiscoveryReport,
        read_report: SaveReadReport,
    ) -> Self {
        let validation: SaveValidationReportDto = read_report.validation.into();
        let ready =
            validation.incompatible_schema.is_none() && validation.missing_required_mods.is_empty();

        Self {
            slot_id,
            path: path.to_string_lossy().to_string(),
            ready,
            registry,
            discovery: discovery.into(),
            validation,
            save: read_report.save,
        }
    }
}

impl From<SaveValidationReport> for SaveValidationReportDto {
    fn from(report: SaveValidationReport) -> Self {
        Self {
            missing_required_mods: report.missing_required_mods,
            incompatible_schema: report.incompatible_schema,
            engine_version_mismatch: report.engine_version_mismatch,
        }
    }
}

impl From<EngineSaveRecoveryReport> for SaveRecoveryReport {
    fn from(report: EngineSaveRecoveryReport) -> Self {
        Self {
            path: report.path.to_string_lossy().to_string(),
            recovered_from: report.recovered_from.to_string_lossy().to_string(),
            failed_primary_backup_path: report
                .failed_primary_backup_path
                .map(|path| path.to_string_lossy().to_string()),
            save: report.save,
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

impl From<ModInstallPreflightReport> for ModInstallPreflightReportDto {
    fn from(report: ModInstallPreflightReport) -> Self {
        let ready = report.is_ready();
        Self {
            source_root: report.source_root.to_string_lossy().to_string(),
            content_root: report
                .content_root
                .map(|path| path.to_string_lossy().to_string()),
            install_root: report.install_root.to_string_lossy().to_string(),
            target_root: report
                .target_root
                .map(|path| path.to_string_lossy().to_string()),
            staging_root: report
                .staging_root
                .map(|path| path.to_string_lossy().to_string()),
            manifest: report.manifest,
            ready,
            issues: report.issues.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ModInstallPreflightIssue> for ModInstallPreflightIssueReport {
    fn from(issue: ModInstallPreflightIssue) -> Self {
        Self {
            severity: match issue.severity {
                ModInstallPreflightIssueSeverity::Error => "error".to_string(),
                ModInstallPreflightIssueSeverity::Warning => "warning".to_string(),
            },
            path: issue.path.to_string_lossy().to_string(),
            kind: mod_discovery_error_kind(&issue.error).to_string(),
            message: issue.error.to_string(),
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
        ModDiscoveryError::UnsupportedPackageSchema(_) => "unsupported_package_schema",
        ModDiscoveryError::PackageManifestMismatch { .. } => "package_manifest_mismatch",
        ModDiscoveryError::ResourcePublicationFailed { .. } => "resource_publication_failed",
        ModDiscoveryError::ResourcePublicationWarning { .. } => "resource_publication_warning",
        ModDiscoveryError::InstallTargetExists(_) => "install_target_exists",
        ModDiscoveryError::InstallRootNotDirectory(_) => "install_root_not_directory",
        ModDiscoveryError::InstallStagingExists(_) => "install_staging_exists",
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
            None,
        )
        .unwrap();

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

        let report = engine_discover_mods(root.to_string_lossy().to_string(), None, None).unwrap();

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
            authorized_unsafe_capabilities: Vec::new(),
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
            authorized_unsafe_capabilities: Vec::new(),
        })
        .unwrap_err();

        assert_eq!(report.kind, "unsafe_install_namespace");

        let _ = fs::remove_dir_all(source_root);
    }

    #[test]
    fn engine_plan_mod_install_uses_explicit_capability_authorization() {
        let source_root = temp_mod_root("install_command_policy_source");
        let install_root = temp_mod_root("install_command_policy_target");
        let mut manifest = manifest("example.policy");
        manifest.capabilities = vec![eratw_mod_runtime::ModCapability::NetworkAccess];
        fs::create_dir_all(&source_root).unwrap();
        fs::write(
            source_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let denied = engine_plan_mod_install(ModInstallRequest {
            source_root: source_root.to_string_lossy().to_string(),
            install_root: install_root.to_string_lossy().to_string(),
            engine_version: Some("0.1.0-m0".to_string()),
            authorized_unsafe_capabilities: Vec::new(),
        })
        .unwrap_err();
        let allowed = engine_plan_mod_install(ModInstallRequest {
            source_root: source_root.to_string_lossy().to_string(),
            install_root: install_root.to_string_lossy().to_string(),
            engine_version: Some("0.1.0-m0".to_string()),
            authorized_unsafe_capabilities: vec!["network_access".to_string()],
        })
        .unwrap();
        let unknown = engine_plan_mod_install(ModInstallRequest {
            source_root: source_root.to_string_lossy().to_string(),
            install_root: install_root.to_string_lossy().to_string(),
            engine_version: Some("0.1.0-m0".to_string()),
            authorized_unsafe_capabilities: vec!["unknown".to_string()],
        })
        .unwrap_err();

        assert_eq!(denied.kind, "unsafe_capability");
        assert_eq!(allowed.manifest.namespace, "example.policy");
        assert_eq!(unknown.kind, "unknown_capability");

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
            authorized_unsafe_capabilities: Vec::new(),
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
    fn engine_preflight_mod_package_install_returns_frontend_report() {
        let package_root = temp_mod_root("preflight_package_command_source");
        let content_root = package_root.join("content");
        let install_root = temp_mod_root("preflight_package_command_target");
        fs::create_dir_all(&content_root).unwrap();
        fs::write(
            content_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.package")).unwrap(),
        )
        .unwrap();
        fs::write(
            package_root.join("eratw-mod-package.json"),
            serde_json::to_string_pretty(&eratw_mod_runtime::ModPackageManifest {
                schema_version: "eratw-mod-package/v0".to_string(),
                namespace: "example.package".to_string(),
                version: "0.1.0".to_string(),
                manifest_path: "content/manifest.json".to_string(),
            })
            .unwrap(),
        )
        .unwrap();

        let report = engine_preflight_mod_package_install(ModInstallRequest {
            source_root: package_root.to_string_lossy().to_string(),
            install_root: install_root.to_string_lossy().to_string(),
            engine_version: Some("0.1.0-m0".to_string()),
            authorized_unsafe_capabilities: Vec::new(),
        })
        .unwrap();

        assert!(report.ready);
        assert_eq!(report.manifest.unwrap().namespace, "example.package");
        assert!(report.target_root.unwrap().ends_with("example.package"));
        assert_eq!(report.issues, Vec::<ModInstallPreflightIssueReport>::new());
        assert!(!install_root.exists());

        let _ = fs::remove_dir_all(package_root);
    }

    #[test]
    fn engine_preflight_mod_package_install_reports_existing_target() {
        let package_root = temp_mod_root("preflight_package_existing_source");
        let content_root = package_root.join("content");
        let install_root = temp_mod_root("preflight_package_existing_target");
        fs::create_dir_all(&content_root).unwrap();
        fs::create_dir_all(install_root.join("example.package")).unwrap();
        fs::write(
            content_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.package")).unwrap(),
        )
        .unwrap();
        fs::write(
            package_root.join("eratw-mod-package.json"),
            serde_json::to_string_pretty(&eratw_mod_runtime::ModPackageManifest {
                schema_version: "eratw-mod-package/v0".to_string(),
                namespace: "example.package".to_string(),
                version: "0.1.0".to_string(),
                manifest_path: "content/manifest.json".to_string(),
            })
            .unwrap(),
        )
        .unwrap();

        let report = engine_preflight_mod_package_install(ModInstallRequest {
            source_root: package_root.to_string_lossy().to_string(),
            install_root: install_root.to_string_lossy().to_string(),
            engine_version: None,
            authorized_unsafe_capabilities: Vec::new(),
        })
        .unwrap();

        assert!(!report.ready);
        assert_eq!(report.issues[0].severity, "error");
        assert_eq!(report.issues[0].kind, "install_target_exists");

        let _ = fs::remove_dir_all(install_root);
        let _ = fs::remove_dir_all(package_root);
    }

    #[test]
    fn engine_install_mod_package_executes_checked_package() {
        let package_root = temp_mod_root("install_package_command_source");
        let content_root = package_root.join("content");
        let install_root = temp_mod_root("install_package_command_target");
        fs::create_dir_all(content_root.join("assets")).unwrap();
        fs::write(
            content_root.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.package")).unwrap(),
        )
        .unwrap();
        fs::write(content_root.join("assets/readme.txt"), "installed").unwrap();
        fs::write(
            package_root.join("eratw-mod-package.json"),
            serde_json::to_string_pretty(&eratw_mod_runtime::ModPackageManifest {
                schema_version: "eratw-mod-package/v0".to_string(),
                namespace: "example.package".to_string(),
                version: "0.1.0".to_string(),
                manifest_path: "content/manifest.json".to_string(),
            })
            .unwrap(),
        )
        .unwrap();

        let report = engine_install_mod_package(ModInstallRequest {
            source_root: package_root.to_string_lossy().to_string(),
            install_root: install_root.to_string_lossy().to_string(),
            engine_version: Some("0.1.0-m0".to_string()),
            authorized_unsafe_capabilities: Vec::new(),
        })
        .unwrap();

        assert_eq!(report.manifest.namespace, "example.package");
        assert!(report.target_root.ends_with("example.package"));
        assert_eq!(report.actions[2].kind, "move_directory");
        assert!(install_root
            .join("example.package/assets/readme.txt")
            .exists());

        let _ = fs::remove_dir_all(install_root);
        let _ = fs::remove_dir_all(package_root);
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
            authorized_unsafe_capabilities: Vec::new(),
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
            authorized_unsafe_capabilities: Vec::new(),
        })
        .unwrap_err();

        assert_eq!(report.kind, "missing_dependency");
    }

    #[test]
    fn engine_plan_enabled_mods_uses_explicit_capability_authorization() {
        let mut manifest = manifest("example.policy");
        manifest.capabilities = vec![eratw_mod_runtime::ModCapability::SystemCommand];

        let denied = engine_plan_enabled_mods(ModEnablementRequest {
            manifests: vec![manifest.clone()],
            enablement: Vec::new(),
            engine_version: Some("0.1.0-m0".to_string()),
            authorized_unsafe_capabilities: Vec::new(),
        })
        .unwrap_err();
        let allowed = engine_plan_enabled_mods(ModEnablementRequest {
            manifests: vec![manifest],
            enablement: Vec::new(),
            engine_version: Some("0.1.0-m0".to_string()),
            authorized_unsafe_capabilities: vec!["system_command".to_string()],
        })
        .unwrap();

        assert_eq!(denied.kind, "unsafe_capability");
        assert_eq!(allowed.enabled[0].namespace, "example.policy");
    }

    #[test]
    fn mod_enablement_settings_round_trips_install_roots() {
        let root = temp_mod_root("mod_enablement_settings");
        let path = root.join("settings/mod_enablement.json");

        let missing = read_mod_enablement_settings_path(&path).unwrap();
        assert!(missing.install_roots.is_empty());

        let mut settings = ModEnablementSettings::default();
        settings.install_roots.insert(
            "mods/installed".to_string(),
            vec![ModEnablement {
                namespace: "example.minimal_character".to_string(),
                enabled: false,
            }],
        );

        write_mod_enablement_settings_path(&path, &settings).unwrap();
        let loaded = read_mod_enablement_settings_path(&path).unwrap();

        assert_eq!(
            loaded.install_roots["mods/installed"],
            vec![ModEnablement {
                namespace: "example.minimal_character".to_string(),
                enabled: false,
            }]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn preflight_load_slot_checks_save_dependencies_against_enabled_registry() {
        let save_root = temp_mod_root("save_preflight_slot");
        let mod_root = temp_mod_root("save_preflight_mods");
        let mod_dir = mod_root.join("example.installable");
        let save_path = save_root.join("slot_1.json");
        let mut world = WorldState::bootstrap_demo();
        world
            .installed_content_packages
            .push(eratw_engine::InstalledContentPackage {
                namespace: "example".to_string(),
                package_id: "example.installable".to_string(),
                version: "0.1.0".to_string(),
                dependencies: Vec::new(),
                conflicts: Vec::new(),
            });
        let save = SaveEnvelope::new("slot_1", world, 123);
        fs::create_dir_all(&mod_dir).unwrap();
        fs::write(
            mod_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest("example.installable")).unwrap(),
        )
        .unwrap();
        write_save_atomic(&save_path, &save, 123).unwrap();

        let ready = preflight_load_slot_path(
            save_path.clone(),
            SavePreflightRequest {
                slot_id: "slot_1".to_string(),
                mod_root: mod_root.to_string_lossy().to_string(),
                enablement: Vec::new(),
                engine_version: Some("0.1.0-m0".to_string()),
                authorized_unsafe_capabilities: Vec::new(),
            },
        )
        .unwrap();
        let blocked = preflight_load_slot_path(
            save_path,
            SavePreflightRequest {
                slot_id: "slot_1".to_string(),
                mod_root: mod_root.to_string_lossy().to_string(),
                enablement: vec![ModEnablement {
                    namespace: "example.installable".to_string(),
                    enabled: false,
                }],
                engine_version: Some("0.1.0-m0".to_string()),
                authorized_unsafe_capabilities: Vec::new(),
            },
        )
        .unwrap();

        assert!(ready.ready);
        assert_eq!(ready.registry.enabled.len(), 1);
        assert!(ready.validation.missing_required_mods.is_empty());
        assert!(!blocked.ready);
        assert_eq!(
            blocked.validation.missing_required_mods,
            vec![SaveModDependency {
                namespace: "example.installable".to_string(),
                version: "0.1.0".to_string(),
                required: true,
            }]
        );

        let _ = fs::remove_dir_all(save_root);
        let _ = fs::remove_dir_all(mod_root);
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
            resources: Vec::new(),
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
            engine_preflight_content_package_install,
            engine_plan_resources,
            engine_inspect_resources,
            engine_preflight_resources,
            engine_audit_resource_publication,
            engine_cache_resources,
            engine_clean_resource_cache,
            engine_discover_mods,
            engine_plan_mod_install,
            engine_install_mod,
            engine_preflight_mod_package_install,
            engine_install_mod_package,
            engine_plan_mod_uninstall,
            engine_uninstall_mod,
            engine_plan_enabled_mods,
            engine_load_mod_enablement,
            engine_save_mod_enablement,
            engine_save_preview,
            engine_save_slot,
            engine_recover_slot,
            engine_preflight_load_slot,
            engine_load_slot
        ])
        .run(tauri::generate_context!())
        .expect("failed to run ERAtw-NEXT desktop app");
}
