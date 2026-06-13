//! 桌面壳 command 桥接。
//!
//! 这里只把引擎结果暴露为 Tauri command，不写业务逻辑。
//! 命名规则：`domain_action`。

use eratw_next_engine as engine;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Default)]
struct DesktopRuntime {
    package: Mutex<Option<engine::LoadedContentPackage>>,
    game: Mutex<Option<engine::GameSession>>,
}

impl DesktopRuntime {
    fn load_package(
        &self,
        path: String,
    ) -> Result<engine::ContentPackageIndex, engine::EngineError> {
        let package = engine::load_content_package(PathBuf::from(path))?;
        let index = package.index.clone();
        *self.lock_package()? = Some(package);
        *self.lock_game()? = None;
        Ok(index)
    }

    fn loaded_package(&self) -> Result<Option<engine::ContentPackageIndex>, engine::EngineError> {
        Ok(self
            .lock_package()?
            .as_ref()
            .map(|package| package.index.clone()))
    }

    fn new_game(&self) -> Result<engine::GameState, engine::EngineError> {
        let package = self
            .lock_package()?
            .clone()
            .ok_or_else(|| runtime_error("CONTENT_NOT_LOADED", "Load a content package first."))?;
        let session = engine::GameSession::from_package(&package)?;
        let state = session.state.clone();
        *self.lock_game()? = Some(session);
        Ok(state)
    }

    fn game_state(&self) -> Result<Option<engine::GameState>, engine::EngineError> {
        Ok(self
            .lock_game()?
            .as_ref()
            .map(|session| session.state.clone()))
    }

    fn apply_command(
        &self,
        command: engine::GameCommand,
    ) -> Result<engine::CommandResult, engine::EngineError> {
        self.lock_game()?
            .as_mut()
            .ok_or_else(|| runtime_error("GAME_NOT_STARTED", "Start or load a game first."))?
            .apply(command)
    }

    fn write_save(&self, path: String) -> Result<engine::SaveReport, engine::EngineError> {
        let game = self.lock_game()?;
        let session = game
            .as_ref()
            .ok_or_else(|| runtime_error("GAME_NOT_STARTED", "Start or load a game first."))?;
        engine::save_game_file(PathBuf::from(path), session)
    }

    fn load_save(&self, path: String) -> Result<engine::GameState, engine::EngineError> {
        let package = self
            .lock_package()?
            .clone()
            .ok_or_else(|| runtime_error("CONTENT_NOT_LOADED", "Load a content package first."))?;
        let session = engine::load_save_file(PathBuf::from(path), &package)?;
        let state = session.state.clone();
        *self.lock_game()? = Some(session);
        Ok(state)
    }

    fn lock_package(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, Option<engine::LoadedContentPackage>>, engine::EngineError>
    {
        self.package
            .lock()
            .map_err(|_| runtime_error("RUNTIME_LOCK_POISONED", "Content runtime lock failed."))
    }

    fn lock_game(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, Option<engine::GameSession>>, engine::EngineError> {
        self.game
            .lock()
            .map_err(|_| runtime_error("RUNTIME_LOCK_POISONED", "Game runtime lock failed."))
    }
}

fn runtime_error(code: &str, message: &str) -> engine::EngineError {
    engine::EngineError::unavailable(code, message)
}

/// `system_get_status() -> Result<SystemStatus, EngineError>`
#[tauri::command]
fn system_get_status() -> Result<engine::SystemStatus, engine::EngineError> {
    engine::system_get_status()
}

/// `map_get_overview() -> Result<MapModel, EngineError>`
#[tauri::command]
fn map_get_overview() -> Result<engine::MapModel, engine::EngineError> {
    engine::map_get_overview()
}

#[tauri::command]
fn content_load_package(
    path: String,
    runtime: tauri::State<'_, DesktopRuntime>,
) -> Result<engine::ContentPackageIndex, engine::EngineError> {
    runtime.load_package(path)
}

#[tauri::command]
fn content_get_loaded(
    runtime: tauri::State<'_, DesktopRuntime>,
) -> Result<Option<engine::ContentPackageIndex>, engine::EngineError> {
    runtime.loaded_package()
}

#[tauri::command]
fn game_new(
    runtime: tauri::State<'_, DesktopRuntime>,
) -> Result<engine::GameState, engine::EngineError> {
    runtime.new_game()
}

#[tauri::command]
fn game_get_state(
    runtime: tauri::State<'_, DesktopRuntime>,
) -> Result<Option<engine::GameState>, engine::EngineError> {
    runtime.game_state()
}

#[tauri::command]
fn game_apply_command(
    command: engine::GameCommand,
    runtime: tauri::State<'_, DesktopRuntime>,
) -> Result<engine::CommandResult, engine::EngineError> {
    runtime.apply_command(command)
}

#[tauri::command]
fn save_write(
    path: String,
    runtime: tauri::State<'_, DesktopRuntime>,
) -> Result<engine::SaveReport, engine::EngineError> {
    runtime.write_save(path)
}

#[tauri::command]
fn save_load(
    path: String,
    runtime: tauri::State<'_, DesktopRuntime>,
) -> Result<engine::GameState, engine::EngineError> {
    runtime.load_save(path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(DesktopRuntime::default())
        .invoke_handler(tauri::generate_handler![
            system_get_status,
            map_get_overview,
            content_load_package,
            content_get_loaded,
            game_new,
            game_get_state,
            game_apply_command,
            save_write,
            save_load
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    // command 函数本身不写业务逻辑，只验证桥接返回结构稳定。
    #[test]
    fn system_command_bridges_engine() {
        let status = system_get_status().expect("ok");
        assert_eq!(status.schema_version, "system-status/v1");
    }

    #[test]
    fn map_command_bridges_engine() {
        let model = map_get_overview().expect("ok");
        assert_eq!(model.schema_version, "map-model/v1");
        assert!(!model.nodes.is_empty());
    }

    #[test]
    fn runtime_reports_missing_content_and_game() {
        let runtime = DesktopRuntime::default();
        assert_eq!(runtime.new_game().unwrap_err().code, "CONTENT_NOT_LOADED");
        assert_eq!(
            runtime
                .apply_command(engine::GameCommand::Wait { minutes: 10 })
                .unwrap_err()
                .code,
            "GAME_NOT_STARTED"
        );
    }
}
