# M3 Resource Loading Foundation

## 当前结构

- `ResourceAsset`：记录 resource_id、source_path、media_type、许可、作者、用途、角色绑定、标签和可选 sha256。
- `eratw_engine::resource::plan_resource_loads`：只做路径规划，不访问文件系统，用于 UI/编辑器预览加载计划。
- `eratw_engine::resource::inspect_resource_files`：在给定内容根目录下检查文件存在性和 sha256，返回结构化资源状态。
- `eratw_engine::resource::preflight_resource_loads`：复用文件检查结果生成 `ready` 和阻断 issue，用于安装前、发布前和运行前资源完整性门禁。
- `ResourceResolutionStatus`：当前覆盖 `planned`、`ready`、`missing`、`unsafe_path`、`hash_mismatch` 和 `io_error`。
- `ResourceFallback`：按资源类型提供 `placeholder_image`、`silent_audio`、`default_font` 或 `missing_resource`。
- Tauri `engine_plan_resources` / `engine_inspect_resources` / `engine_preflight_resources`：前端通过 engine command 获取资源计划、检查报告和安装/运行前预检报告。

## 安全规则

- `source_path` 必须是内容包根目录下的相对路径。
- 拒绝绝对路径、Windows 盘符路径、根路径和 `..` 逃逸。
- 剧情和事件只引用 `resourceId`；文件路径只存在于 ResourceAsset 元数据。
- 文件缺失、hash 不匹配或 IO 错误不应让 UI 崩溃，前端按 fallback 降级展示。
- 资源预检把缺失文件、不安全路径、hash 不匹配和 IO 错误视为 blocking issue；报告同时保留完整 resolution entries，方便 UI 展示降级方案。

## 后续

- 接入真实资源缓存、缩略图生成和低配模式。
- 为缺失资源恢复 UI、资源许可检查和发布前资源完整性报告提供入口。
- 将资源根目录绑定到 Mod/package registry，而不是由调用方手工传入。
