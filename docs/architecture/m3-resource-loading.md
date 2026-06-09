# M3 Resource Loading Foundation

## 当前结构

- `ResourceAsset`：记录 resource_id、source_path、media_type、许可、作者、用途、角色绑定、标签和可选 sha256。
- `eratw_engine::resource::plan_resource_loads`：只做路径规划，不访问文件系统，用于 UI/编辑器预览加载计划。
- `eratw_engine::resource::inspect_resource_files`：在给定内容根目录下检查文件存在性和 sha256，返回结构化资源状态。
- `eratw_engine::resource::preflight_resource_loads`：复用文件检查结果生成 `ready` 和阻断 issue，用于安装前、发布前和运行前资源完整性门禁。
- `ResourceResolutionStatus`：当前覆盖 `planned`、`ready`、`missing`、`unsafe_path`、`hash_mismatch` 和 `io_error`。
- `ResourceFallback`：按资源类型提供 `placeholder_image`、`silent_audio`、`default_font` 或 `missing_resource`。
- `ResourcePlanningOptions.low_spec`：为低配模式生成同一份可预检计划；图片标记为 `thumbnail_only`，音频和 `other` 标记为 `deferred`，字体保持 `eager`。
- `ResourceResolution`：除了源文件路径和状态，还输出稳定 `cache_key`、规划中的资源缓存路径、低配图片缩略图路径和 `load_strategy`，供 UI/加载器提前决策。
- `eratw_engine::resource::cache_resource_loads`：先执行文件检查，只把 `ready` 资源复制到规划缓存路径，缺失、unsafe、hash mismatch 和 IO 错误资源进入 skipped/failed 缓存报告，不写入缓存。
- `ResourceCacheReport`：输出缓存执行结果、cached/skipped/failed 计数、原始 resolution 和逐项缓存状态。
- `eratw_engine::resource::clean_resource_cache`：基于当前资源计划清理 `.eratw-cache/resources` 和 `.eratw-cache/thumbnails` 中不再被引用的文件，保留当前缓存项、移除过期缓存项、跳过子目录并报告失败项。
- `ResourceCacheCleanReport`：输出 cache root、kept/removed/skipped/failed 计数、移除字节数、原始 resolution 和逐项清理状态。
- Tauri `engine_plan_resources` / `engine_inspect_resources` / `engine_preflight_resources`：前端通过 engine command 获取资源计划、检查报告和安装/运行前预检报告。
- Tauri `engine_cache_resources`：前端可触发真实文件缓存执行；browser mock 返回同形状的模拟缓存报告。
- Tauri `engine_clean_resource_cache`：前端可触发真实缓存清理；browser mock 返回同形状的模拟清理报告。

## 安全规则

- `source_path` 必须是内容包根目录下的相对路径。
- 拒绝绝对路径、Windows 盘符路径、根路径和 `..` 逃逸。
- 剧情和事件只引用 `resourceId`；文件路径只存在于 ResourceAsset 元数据。
- 文件缺失、hash 不匹配或 IO 错误不应让 UI 崩溃，前端按 fallback 降级展示。
- 资源预检把缺失文件、不安全路径、hash 不匹配和 IO 错误视为 blocking issue；报告同时保留完整 resolution entries，方便 UI 展示降级方案。
- 低配模式只改变加载策略和派生缓存/缩略图计划，不降低路径安全、存在性和 hash 检查要求。
- 缓存执行只信任 `inspect_resource_files` 后的 `ready` 资源；任何未通过检查的条目不会被复制到 `.eratw-cache`。
- 缓存写入会确认目标目录仍在内容根目录内，拒绝目录或符号链接形式的缓存目标。
- 缓存清理只扫描 engine 管理的 `.eratw-cache/resources` 和 `.eratw-cache/thumbnails` 两个目录，不递归删除未知子目录；缓存目录异常、逃逸或删除失败都会进入 failed 报告。

## 后续

- 接入后台缩略图生成任务。
- 为缺失资源恢复 UI、资源许可检查和发布前资源完整性报告提供入口。
- 将资源根目录绑定到 Mod/package registry，而不是由调用方手工传入。
