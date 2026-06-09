# M2 Engine Save Foundation

## 范围

本阶段先建立存档领域边界，不承诺完整 UI 存取档体验。

## 当前结构

- `SaveEnvelope`：版本化存档外壳，包含 schema、engine 版本、保存时间、slot、Mod 依赖和 `WorldState`。
- `SaveModDependency`：记录存档需要的 Mod 命名空间、版本和是否必需。
- `WorldState.installed_content_packages`：记录已成功安装的运行内容包 namespace、package_id 和版本。
- `SaveValidationReport`：报告缺失必需 Mod、schema 不兼容和 engine 版本不一致。
- `SaveBackupPlan`：生成覆盖、迁移、恢复前的备份目标路径。
- `write_save_atomic`：写入同目录临时文件，覆盖前复制备份，再替换主存档。
- `read_save`：读取 JSON 存档，执行 schema migration，再做基础校验。
- Tauri `engine_save_slot` / `engine_load_slot`：按 slot id 写入应用数据目录下的 `saves/{slot}.json`。
- `WorldState.command_log` 随 `SaveEnvelope` 序列化，用于后续确定性回放和故障复现。
- `SaveEnvelope::new` 会从 `WorldState.installed_content_packages` 派生 `mod_dependencies`；当前使用 `package_id` 作为存档依赖 namespace。

## 规则

- 当前 schema 为 `1`。
- 高版本存档直接拒绝读取。
- 空 slot、空地点、空角色视为损坏存档。
- 旧 schema 通过 `migrate_to_current` 进入当前结构；真实迁移步骤后续逐版补齐。
- 前端不得直接改写 `WorldState`，存档预览也通过 Tauri command 生成。
- slot id 仅允许 ASCII 字母、数字、`-`、`_`，避免路径穿越。
- 在完整 Mod registry 接入前，存档读取会把存档内嵌的已安装内容包记录视作当前可用依赖；外部 Mod 启停检查后续由 `eratw_mod_runtime` 提供。

## 后续

- 增加备份轮转、损坏存档恢复 UI 和恢复入口。
- 将 Mod runtime 的 manifest 校验接入存档依赖检查。
- 将 deterministic replay seed 接入当前事件调度器和 command log。
