//! ERAtw-NEXT 核心引擎（M0）。
//!
//! 仅提供系统状态与地图模型的纯查询 API：
//! 不依赖 Tauri、不读取磁盘、不接触真实内容目录。
//! 业务逻辑集中在此 crate，桌面壳只做 command 桥接。

mod map;
mod status;

pub use map::{map_overview, Area, Grid, LegendEntry, MapModel, MapNode, Occupant};
pub use status::{
    system_status, AppInfo, BuildInfo, Capability, EngineInfo, Milestone, PathPlaceholder,
    SystemStatus,
};

use serde::{Deserialize, Serialize};

/// 引擎对外错误的稳定结构，可直接序列化给前端。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineError {
    pub code: String,
    pub message: String,
    pub details: serde_json::Value,
}

impl EngineError {
    pub fn unavailable(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for EngineError {}

/// `system_get_status` 命令的业务实现。
pub fn system_get_status() -> Result<SystemStatus, EngineError> {
    Ok(system_status())
}

/// `map_get_overview` 命令的业务实现。
pub fn map_get_overview() -> Result<MapModel, EngineError> {
    Ok(map_overview())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_command_returns_status() {
        let status = system_get_status().expect("status should be available");
        assert_eq!(status.schema_version, "system-status/v1");
    }

    #[test]
    fn map_command_returns_model() {
        let model = map_get_overview().expect("map should be available");
        assert_eq!(model.schema_version, "map-model/v1");
    }

    #[test]
    fn engine_error_has_stable_shape() {
        let err = EngineError::unavailable("SYSTEM_STATUS_UNAVAILABLE", "unavailable");
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "SYSTEM_STATUS_UNAVAILABLE");
        assert!(json["details"].is_object());
    }
}
