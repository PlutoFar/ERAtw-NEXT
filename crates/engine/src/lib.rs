//! ERAtw-NEXT 核心引擎。
//!
//! 业务逻辑集中在此 crate，桌面壳只做 command 桥接。

mod content;
mod game;
mod map;
mod save;
mod status;

pub use content::{
    load_content_package, CharacterIndexEntry, ContentPackageIndex, ContentPackageManifest,
    LoadedContentPackage, LocationIndexEntry, PackageCounts, PackageIdentity, ResourceIndexEntry,
    RuntimeLocation, CONTENT_PACKAGE_INDEX_SCHEMA_VERSION,
};
pub use game::{
    apply_command, new_game, replay_commands, CommandResult, EventRecord, GameClock, GameCommand,
    GameContext, GameSession, GameState, PlayerState, ScheduledEvent, GAME_STATE_SCHEMA_VERSION,
};
pub use map::{map_overview, Area, Grid, LegendEntry, MapModel, MapNode, Occupant};
pub use save::{
    load_save_file, save_game_file, SaveDependency, SaveEnvelope, SaveReport,
    SAVE_ENVELOPE_SCHEMA_VERSION,
};
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
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details,
        }
    }

    pub fn unavailable(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            code,
            message,
            serde_json::Value::Object(serde_json::Map::new()),
        )
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
