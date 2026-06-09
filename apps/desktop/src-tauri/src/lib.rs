use eratw_content::ContentPackage;
use eratw_engine::{
    resource::{inspect_resource_files, plan_resource_loads, ResourceResolutionReport},
    save::{read_save, write_save_atomic, SaveEnvelope},
    EngineCommand, WorldState,
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
            engine_save_preview,
            engine_save_slot,
            engine_load_slot
        ])
        .run(tauri::generate_context!())
        .expect("failed to run ERAtw-NEXT desktop app");
}
