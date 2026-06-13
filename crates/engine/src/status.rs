//! 系统状态：项目身份、引擎信息、构建信息、路径占位、能力、里程碑。

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: &str = "system-status/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatus {
    pub schema_version: String,
    pub app: AppInfo,
    pub engine: EngineInfo,
    pub build: BuildInfo,
    pub paths: Vec<PathPlaceholder>,
    pub capabilities: Vec<Capability>,
    pub current_milestone: String,
    pub milestones: Vec<Milestone>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub stage: String,
    pub tagline: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildInfo {
    pub profile: String,
    pub git_describe: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathPlaceholder {
    pub id: String,
    pub label: String,
    pub value: String,
    pub kind: String,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub id: String,
    pub label: String,
    pub status: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Milestone {
    pub id: String,
    pub title: String,
    pub status: String,
    pub summary: String,
}

fn s(value: &str) -> String {
    value.to_string()
}

fn build_profile() -> String {
    if cfg!(debug_assertions) {
        s("debug")
    } else {
        s("release")
    }
}

/// 返回稳定的系统状态。
pub fn system_status() -> SystemStatus {
    SystemStatus {
        schema_version: s(SCHEMA_VERSION),
        app: AppInfo {
            name: s("ERAtw-NEXT"),
            stage: s("M4"),
            tagline: s("ERAtw 现代化引擎与桌面应用，不是旧运行时打包。"),
        },
        engine: EngineInfo {
            name: s("eratw_next_engine"),
            version: s(env!("CARGO_PKG_VERSION")),
        },
        build: BuildInfo {
            profile: build_profile(),
            git_describe: option_env!("ERATW_GIT_DESCRIBE").map(s),
            timestamp: option_env!("ERATW_BUILD_TIMESTAMP").map(s),
        },
        paths: vec![
            PathPlaceholder {
                id: s("content_source"),
                label: s("内容源"),
                value: s("D:\\AICODE\\eratw-content"),
                kind: s("read_only"),
                note: s("外部只读源，永不复制进引擎仓库。"),
            },
            PathPlaceholder {
                id: s("playable_reference"),
                label: s("可游玩对照"),
                value: s("D:\\AICODE\\eratw"),
                kind: s("reference"),
                note: s("仅供人工参考，引擎不读取。"),
            },
            PathPlaceholder {
                id: s("modern"),
                label: s("ERAtw-modern"),
                value: s("D:\\AICODE\\ERAtw-modern"),
                kind: s("excluded"),
                note: s("无关项目，不作为输入或迁移来源。"),
            },
            PathPlaceholder {
                id: s("native_foundation"),
                label: s("ERAtw-native-foundation"),
                value: s("D:\\AICODE\\ERAtw-native-foundation"),
                kind: s("excluded"),
                note: s("无关项目，不作为输入或迁移来源。"),
            },
        ],
        capabilities: vec![
            Capability {
                id: s("system_status"),
                label: s("系统状态查询"),
                status: s("available"),
                description: s("system_get_status 已可用并被 schema 校验。"),
            },
            Capability {
                id: s("map_overview"),
                label: s("地图总览（双模式）"),
                status: s("available"),
                description: s("map_get_overview 提供字符画 / SVG 共享的地图模型。"),
            },
            Capability {
                id: s("content_audit"),
                label: s("只读内容审计"),
                status: s("available"),
                description: s("M1 只读审计工具已完成。"),
            },
            Capability {
                id: s("content_migration"),
                label: s("内容转换草案"),
                status: s("available"),
                description: s("M2 可生成仓库外 draft 内容包。"),
            },
            Capability {
                id: s("content_package"),
                label: s("内容包加载"),
                status: s("available"),
                description: s("M3 可加载、校验并索引仓库外内容包。"),
            },
            Capability {
                id: s("game_state"),
                label: s("玩法状态机与存档"),
                status: s("available"),
                description: s("M4 reducer、事件队列、replay 与版本化存档已可用。"),
            },
            Capability {
                id: s("erb_runtime"),
                label: s("ERB 子集解释器"),
                status: s("disabled"),
                description: s("默认禁用，不执行任何外部 ERB 或脚本。"),
            },
        ],
        current_milestone: s("M4"),
        milestones: vec![
            Milestone {
                id: s("M0"),
                title: s("现代工程骨架"),
                status: s("done"),
                summary: s("Rust + Tauri + React/MUI 工程基线。"),
            },
            Milestone {
                id: s("M1"),
                title: s("只读内容审计"),
                status: s("done"),
                summary: s("安全扫描 eratw-content，输出规模/编码/资源引用报告。"),
            },
            Milestone {
                id: s("M2"),
                title: s("内容契约与转换草案"),
                status: s("done"),
                summary: s("定义新内容 schema 并生成可校验的草案内容包。"),
            },
            Milestone {
                id: s("M3"),
                title: s("最小内容包加载"),
                status: s("done"),
                summary: s("加载、校验并展示独立内容包索引。"),
            },
            Milestone {
                id: s("M4"),
                title: s("玩法状态机与存档基础"),
                status: s("done"),
                summary: s("确定性 reducer、时间事件队列与版本化存档。"),
            },
            Milestone {
                id: s("M5"),
                title: s("ERB 迁移双轨实验"),
                status: s("planned"),
                summary: s("验证有限 ERB 子集与新 schema 主线的边界。"),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_has_stable_identity() {
        let status = system_status();
        assert_eq!(status.schema_version, SCHEMA_VERSION);
        assert_eq!(status.engine.name, "eratw_next_engine");
        assert_eq!(status.current_milestone, "M4");
    }

    #[test]
    fn capabilities_are_present() {
        let status = system_status();
        assert!(!status.capabilities.is_empty());
        assert!(status.capabilities.iter().any(|c| c.id == "map_overview"));
    }

    #[test]
    fn milestones_contain_current() {
        let status = system_status();
        let current = status.current_milestone.clone();
        assert!(status.milestones.iter().any(|m| m.id == current));
    }

    #[test]
    fn build_profile_is_known() {
        let profile = system_status().build.profile;
        assert!(profile == "debug" || profile == "release");
    }
}
