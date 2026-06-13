# ERAtw-NEXT M3 最小内容包加载设计

## 结论

M3 在 `eratw_next_engine` 内建立仓库外内容包的只读加载、结构校验、引用校验和索引查询能力，并通过 Tauri command 暴露给桌面端。M3 不读取 `eratw-content`，不执行 ERB，不复制资源，不把内容包写入 engine 仓库。

实现状态：已完成。

## 输入边界

输入必须是绝对路径、本地目录、非 symlink/reparse point。加载器只访问以下固定文件：

```text
manifest.json
dictionaries/legacy-csv-dictionaries.jsonl
characters/characters.jsonl
locations/locations.jsonl
resources/resources.jsonl
dialogue/dialogue-sources.jsonl
dialogue/dialogue-scenes.jsonl
```

规则：

- 固定相对路径，不接受调用方拼接任意文件名。
- 每个文件必须是普通文件，canonical path 必须仍位于 package root。
- 单文件上限 128 MiB，单条 JSONL 上限 4 MiB。
- schema 嵌入引擎二进制，不依赖运行目录中的 schema 文件。
- package dependency 未满足、engine version 不兼容时拒绝加载。
- resource `sourcePath` 只允许安全相对路径；M3 不读取该资源正文。

## 加载流水线

1. 校验 package root。
2. 编译嵌入式 Draft 2020-12 schema。
3. 校验并解析 manifest。
4. 流式读取 JSONL，逐条执行 schema 校验。
5. 检查各对象类型内部 ID 唯一。
6. 检查 character/resource/dialogue source/dialogue scene/dictionary/location 引用，以及 scene 入口节点和节点 ID。
7. 建立角色、地点、资源索引和运行时地点连接表。
8. 判定包是否可玩。

可玩条件：

- manifest capabilities 包含 `playable.core`。
- manifest review status 为 `accepted`。
- 至少存在一个地点。

M2 全量 draft 可以加载和浏览，但因为没有 accepted review、`playable.core` 和地点，必须判定为不可玩。

## API

Tauri commands：

```text
content_load_package(path) -> ContentPackageIndex
content_get_loaded() -> ContentPackageIndex | null
```

`ContentPackageIndex` 使用 `content-package-index/v1`：

- package identity、root path、engine requirement。
- playable、capabilities、review status。
- dictionary/character/location/resource/dialogue source/dialogue scene 计数。
- 角色、地点、资源索引。
- 非阻断 warning。

加载新包会清空旧游戏会话，避免跨包状态污染。

## 结构化错误

主要错误码：

- `CONTENT_PATH_NOT_ABSOLUTE`
- `CONTENT_PATH_TRAVERSAL`
- `CONTENT_PATH_NETWORK`
- `CONTENT_ROOT_REPARSE_POINT`
- `CONTENT_FILE_MISSING`
- `CONTENT_FILE_ESCAPE`
- `CONTENT_FILE_TOO_LARGE`
- `CONTENT_JSON_INVALID`
- `CONTENT_SCHEMA_INVALID`
- `CONTENT_DUPLICATE_ID`
- `CONTENT_REFERENCE_INVALID`
- `CONTENT_DEPENDENCY_MISSING`
- `CONTENT_ENGINE_VERSION_MISMATCH`

错误统一使用 `EngineError { code, message, details }`。

## 桌面端

- 内容包页支持原生目录选择和手工绝对路径回退。
- 展示 package identity、可玩状态、复核状态、warning 和对象计数。
- 角色、地点、资源使用分页表格，每页 50 条。
- 浏览器开发模式使用自有 mock package，不访问本地文件系统。

## 验收

- M2 真实草案包加载成功：161 角色、2685 dialogue source、0 dialogue scene、28 dictionary、1320 resource。
- 草案包被正确判定为不可玩。
- 仓库外自有最小包加载成功：1 角色、2 地点，可玩判定为 true。
- schema 错误、缺失引用、缺依赖、engine version 不兼容均有测试。
- UI 可选择目录并展示角色/地点/资源索引。
