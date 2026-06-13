//! 桌面壳 command 桥接。
//!
//! 这里只把引擎结果暴露为 Tauri command，不写业务逻辑。
//! 命名规则：`domain_action`。

use eratw_next_engine as engine;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            system_get_status,
            map_get_overview
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
}
