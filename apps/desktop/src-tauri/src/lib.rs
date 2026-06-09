use eratw_content::ContentPackage;
use eratw_engine::{
    resource::{inspect_resource_files, plan_resource_loads, ResourceResolutionReport},
    save::{read_save, write_save_atomic, SaveEnvelope},
    EngineCommand, WorldState,
};
use eratw_mod_runtime::{
    discover_mods_for_engine, DiscoveredMod, ModDiscoveryError, ModDiscoveryIssue,
    ModDiscoveryReport, ModManifest, ModValidationError,
};
use serde::Serialize;
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

fn mod_discovery_error_kind(error: &ModDiscoveryError) -> &'static str {
    match error {
        ModDiscoveryError::Io(_) => "io",
        ModDiscoveryError::Json(_) => "json",
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
            engine_save_preview,
            engine_save_slot,
            engine_load_slot
        ])
        .run(tauri::generate_context!())
        .expect("failed to run ERAtw-NEXT desktop app");
}
