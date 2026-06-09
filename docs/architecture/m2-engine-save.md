# M2 Engine Save Foundation

## 范围

本阶段先建立存档领域边界，不承诺完整 UI 存取档体验。

## 当前结构

- `SaveEnvelope`：版本化存档外壳，包含 schema、engine 版本、保存时间、slot、Mod 依赖和 `WorldState`。
- `SaveModDependency`：记录存档需要的 Mod 命名空间、版本和是否必需。
- `SaveValidationReport`：报告缺失必需 Mod、schema 不兼容和 engine 版本不一致。
- `SaveBackupPlan`：生成覆盖、迁移、恢复前的备份目标路径。

## 规则

- 当前 schema 为 `1`。
- 高版本存档直接拒绝读取。
- 空 slot、空地点、空角色视为损坏存档。
- 旧 schema 通过 `migrate_to_current` 进入当前结构；真实迁移步骤后续逐版补齐。
- 前端不得直接改写 `WorldState`，存档预览也通过 Tauri command 生成。

## 后续

- 增加真实文件读写、原子写入、备份轮转。
- 增加损坏存档恢复入口。
- 将 Mod runtime 的 manifest 校验接入存档依赖检查。
- 加入 deterministic replay seed 和 command log。
