# M6 Mod Runtime

## 当前结构

- `ModManifest`：声明 namespace、名称、版本、兼容 engine 版本、加载权重、依赖、冲突和能力。
- `ModDependency`：声明依赖 namespace、可选版本约束和是否必需。
- `ModCapability`：当前支持 `content`、`theme`、`rules_extension`，以及默认拒绝的高危能力。
- `validate_manifest`：拒绝空 namespace/name/version/engine_version、重复依赖、重复冲突和默认高危能力。
- `plan_load_order`：在启用 Mod 集合上执行依赖检查、版本检查、冲突检查、循环依赖检查，并输出稳定加载顺序。
- `read_manifest_file`：读取单个 `manifest.json`，完成 JSON 解析和 manifest 安全校验。
- `discover_mods` / `discover_mods_for_engine`：扫描 Mod 根目录的一级子目录，分别返回成功发现的 manifest 和每个失败 manifest 的结构化错误。
- Tauri `engine_discover_mods`：桌面层把发现报告转换成前端稳定 DTO，包含可展示的错误类型和消息。
- 运行时内容包安装成功后会写入 `WorldState.installed_content_packages`，包含 package_id、version、dependencies 和 conflicts；存档外壳据此生成 `mod_dependencies`，为后续 Mod registry 接管启停检查预留稳定入口。

## 安全默认值

- 默认拒绝 `local_file_access`、`network_access` 和 `system_command`。
- 缺失必需依赖时不加载；缺失可选依赖时允许继续。
- 依赖版本不匹配、声明冲突或循环依赖都会返回结构化错误。
- 加载顺序先保证依赖在被依赖者之前，再在当前可加载集合内按 `load_order` 和 namespace 排序。
- Mod 目录发现不会因为单个损坏 manifest 阻塞其他 Mod；坏 manifest 进入 discovery errors，好的 manifest 继续进入后续加载计划。
- 前端只接收 discovery DTO，不依赖 Rust 内部错误枚举，避免后续 runtime 错误模型调整直接破坏 UI。
- 内容包安装阶段已经执行必需依赖、可选依赖、版本匹配和双向冲突检查；完整 Mod registry 接入后会统一迁移到 `eratw_mod_runtime` 的启停计划。

## Manifest 示例

```json
{
  "namespace": "example.minimal_character",
  "name": "最小角色 Mod",
  "version": "0.1.0",
  "engine_version": "0.1.0-m0",
  "load_order": 0,
  "dependencies": [
    {
      "namespace": "core.base",
      "version": "0.1.0",
      "required": true
    }
  ],
  "conflicts": [],
  "capabilities": ["content"]
}
```

## 后续

- 将 `eratw_mod_runtime` 接入内容包安装和存档依赖检查。
- 将存档依赖从当前内容包记录升级为完整 Mod manifest registry。
- 增加禁用/启用状态、错误恢复和安装目录写入流程。
- 为高危能力加入显式授权模型，而不是在默认校验中直接放行。
