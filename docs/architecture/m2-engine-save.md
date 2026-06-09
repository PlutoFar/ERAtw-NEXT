# M2 Engine Save Foundation

## 范围

本阶段先建立存档领域边界，不承诺完整 UI 存取档体验。

## 当前结构

- `SaveEnvelope`：版本化存档外壳，包含 schema、engine 版本、保存时间、slot、Mod 依赖和 `WorldState`。
- `SaveModDependency`：记录存档需要的 Mod 命名空间、版本和是否必需。
- `WorldState.installed_content_packages`：记录已成功安装的运行内容包 namespace、package_id 和版本。
- `SaveValidationReport`：报告缺失必需 Mod、schema 不兼容和 engine 版本不一致。
- `preflight_save_against_registry`：读取存档但不替换当前世界，用外部启用 Mod registry 严格检查存档依赖。
- `SaveBackupPlan`：生成覆盖、迁移、恢复前的备份目标路径。
- `write_save_atomic`：写入同目录临时文件，覆盖前复制备份，再替换主存档。
- `recover_save_from_latest_backup`：当主存档损坏时，选择同目录最新 `.bak` 恢复主存档，并先备份损坏的主存档。
- `read_save`：读取 JSON 存档，执行 schema migration，再做基础校验。
- Tauri `engine_save_slot` / `engine_recover_slot` / `engine_load_slot`：按 slot id 写入、从最新备份恢复、读取应用数据目录下的 `saves/{slot}.json`。
- `WorldState.command_log` 随 `SaveEnvelope` 序列化，用于后续确定性回放和故障复现。
- `SaveEnvelope::new` 会从 `WorldState.installed_content_packages` 派生 `mod_dependencies`；当前使用 `package_id` 作为存档依赖 namespace。

## 规则

- 当前 schema 为 `1`。
- 高版本存档直接拒绝读取。
- 空 slot、空地点、空角色视为损坏存档。
- 旧 schema 通过 `migrate_to_current` 进入当前结构；真实迁移步骤后续逐版补齐。
- 前端不得直接改写 `WorldState`，存档预览也通过 Tauri command 生成。
- slot id 仅允许 ASCII 字母、数字、`-`、`_`，避免路径穿越。
- 兼容读档路径仍会把存档内嵌的已安装内容包记录视作当前可用依赖；读档预检路径使用 `eratw_mod_runtime` 生成的外部启用 registry 严格检查缺失或版本不匹配的必需 Mod。
- 恢复入口不直接信任备份文件；复制最新备份回主路径后仍走 `read_save` migration 和基础校验。

## 后续

- 增加备份轮转和损坏存档恢复 UI。
- 将桌面读档 UI 切到预检优先的确认流程。
- 将 deterministic replay seed 接入当前事件调度器和 command log。
