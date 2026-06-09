use eratw_engine::{EngineCommand, WorldState};
use std::sync::Mutex;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(WorldState::bootstrap_demo()))
        .invoke_handler(tauri::generate_handler![engine_snapshot, engine_dispatch])
        .run(tauri::generate_context!())
        .expect("failed to run ERAtw-NEXT desktop app");
}
