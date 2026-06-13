# ERAtw-NEXT 现代化总路线

## 定位

ERAtw-NEXT 是 ERAtw 的现代化引擎与桌面应用项目，不是旧 Emuera 运行时的再打包。长期方向是以新 schema、新 engine、新 UI 和独立内容包为主线，保留有限 ERB 子集解释器作为迁移期工具，不承诺全量 Emuera/ERB 兼容。

源内容 `D:\AICODE\eratw-content` 永远作为外部只读输入。转换产物未来进入独立内容包仓库。`ERAtw-NEXT` 只保存 engine、工具、schema、UI、测试、文档和必要示例。

## 基本原则

- 新 schema 为主，有限 ERB 子集解释器为辅。
- 每个阶段都有可验证退出标准。
- 先建立工程可信度，再迁移内容。
- 内容本体、旧 exe/dll、存档、缓存、构建输出不进入引擎仓库。
- 不使用 `ERAtw-modern`、`ERAtw-native-foundation`，也不参考旧 `ERAtw-NEXT` 提交。
- 禁止执行外部 ERB/脚本，内容目录默认只读。

## M0：现代工程骨架

目标：建立 Rust + Tauri + React/TypeScript + MUI 的可启动工程基线。

涉及子系统：

- Engine：`eratw_next_engine`，只提供系统状态。
- Desktop：Tauri 2 桌面壳。
- UI：MUI 首屏，展示项目身份、迁移状态、路径占位。
- Contract：`schemas/system-status.schema.json`。
- CI：Windows 完整验证，Ubuntu 非桌面测试。
- Docs：README、M0 spec、本路线。

退出标准：

- 应用能启动并显示 `system_get_status` 返回的状态。
- schema 被测试实际校验。
- Rust/TS 测试、typecheck、build、diff check 有明确验证结果。
- 本地 agent 指令与本地配置被忽略，不进入远端。

## M1：只读内容审计

目标：建立安全的内容审计器，扫描 `eratw-content`，输出规模、编码、资源引用、CSV/ERB 分类报告。

涉及子系统：

- Content Audit：只读扫描、allowlist、路径穿越防护、符号链接策略。
- Reports：生成 JSON/Markdown 报告。
- UI：M1 不接 UI，审计仅通过独立 CLI 运行。
- Security：不执行 ERB，不执行 Python/批处理脚本，不访问网络。

退出标准：

- 可统计 CSV、ERB、resources、font、sound、docs。
- 可识别明显排除项：exe/dll、sav、log、cache、archive。
- 可输出缺失资源引用、编码异常、疑似乱码、文件规模热点。
- 审计结果可复现，不修改内容源。

## M2：内容契约与转换草案

目标：定义新内容 schema，并为角色、地点、资源、基础文本建立最小转换草案。

涉及子系统：

- Schemas：character、location、resource、dialogue、package manifest。
- Converter：只生成草案 JSON，不导入运行时。
- Validation：schema 校验、引用校验、资源路径校验。
- Docs：记录原字段到新字段的映射和不可自动迁移项。

退出标准：

- 至少一批角色、资源和 dialogue source 能生成 draft package；地点建立 schema 与人工草案路径，不从正文猜测。
- draft package 可通过 schema 校验。
- 未映射字段明确列出，不静默丢失。
- 转换输出写到临时目录或独立内容包仓库，不进入 engine 仓库。

## M3：最小内容包加载

目标：engine 能加载小型新 schema 内容包，并由桌面 UI 展示索引。

涉及子系统：

- Engine：内容包加载、基础索引、引用校验。
- Desktop：使用原生目录选择器或绝对路径读取只读 package。
- UI：角色、地点、资源索引视图。
- Contract：content package API schema。

退出标准：

- 能加载人工维护或转换生成的小型内容包。
- 加载失败返回结构化错误。
- UI 能展示角色/地点/资源，不进入玩法循环。
- 内容包与 engine 仓库仍分离。

实现结果：

- M2 全量草案包 4194 个对象可被 M3 loader 完整索引。
- 新增 `content-package-index/v1`、结构化错误和跨引用验证。
- 草案包保持不可玩；仓库外 accepted 自有最小包可进入 M4。

## M4：玩法状态机与存档基础

目标：建立可测试的核心玩法状态机、时间推进和存档 envelope。

涉及子系统：

- Engine：world state、time、event queue、command reducer。
- Save：版本化存档 envelope、迁移占位、依赖记录。
- UI：基础状态面板和命令调度入口。
- Tests：确定性 replay 测试。

退出标准：

- 同一命令序列能重放出同一状态。
- 存档能保存/加载最小 world state。
- 错误存档、版本不匹配、缺依赖有明确报告。
- 不依赖 ERB 执行。

实现结果：

- `game-state/v1` 和纯 command reducer 已实现。
- 时间推进、地点移动、体力、flags、稳定事件队列已实现。
- `save-envelope/v1` 记录依赖、initial/current state、command log 和 SHA-256。
- 存档加载执行版本、依赖、hash 和确定性 replay 校验。

## M5：ERB 迁移双轨实验

目标：验证新 schema 主线与有限 ERB 子集解释器的边界。

策略：

- 新 schema 承载长期内容。
- 有限 ERB 子集解释器只服务迁移期高频语法。
- 不承诺全 Emuera 兼容。
- 每个支持的 ERB 语法都必须有样例、测试和退出策略。

涉及子系统：

- ERB Audit：统计高频语法、函数、变量、宏。
- ERB Subset Runtime：只支持经过选择的表达式、条件、文本输出和调用形式。
- Converter：能把部分 ERB 文本/分支转换到 dialogue schema。
- Risk Log：记录不能支持或不值得支持的语法。

退出标准：

- 明确支持语法清单和拒绝语法清单。
- 有少量真实高频 ERB 片段可被解析或转换。
- 子集解释器不能执行任意外部脚本或危险操作。
- 能判断继续扩展、停止扩展或转向纯转换。

## M6：资源与表现层

目标：建立资源 manifest、懒加载和 UI 表现层规则。

涉及子系统：

- Resource Manifest：资源 ID、路径、hash、license/author 占位、用途标签。
- Loader：安全路径解析、hash 校验、缺失资源 fallback。
- UI：头像/立绘/音频占位展示。
- Tooling：资源缺失和重复报告。

退出标准：

- 内容包能引用资源 ID 而不是直接路径。
- 缺失、hash 不匹配、不安全路径都有结构化错误。
- UI 能展示资源 fallback，不崩溃。

## M7：编辑器与迁移工作台

目标：提供本地工具辅助内容迁移、人工修正和报告复核。

涉及子系统：

- Editor：内容包浏览、字段编辑、引用修复。
- Migration Workbench：审计报告、转换草案、差异视图。
- Validation：保存前校验、批量问题列表。
- UX：面向维护者，不做公开发布承诺。

退出标准：

- 能打开内容包草案并修正基础字段。
- 能从报告跳转到问题项。
- 保存操作不会修改 `eratw-content` 源目录。

## M8：打包、发布与内容包生态

目标：形成可分发桌面应用和独立内容包流程。

涉及子系统：

- Build：Windows 桌面打包。
- Release：版本号、changelog、artifact 检查。
- Content Packages：独立仓库、版本、依赖、冲突声明。
- Mod：有限扩展点、权限声明、安全默认拒绝。

退出标准：

- 可生成可安装或可运行的 Windows 桌面产物。
- 发布产物不包含源码外内容、缓存、旧运行时或本地配置。
- 内容包能独立版本化并声明 engine 兼容范围。
- Mod 权限默认拒绝，可信工作流显式授权。

## 长期风险

- ERB 全兼容成本不可控，必须持续避免变成旧运行时复刻。
- 内容版权、作者、许可元数据可能不完整，需要人工治理。
- 资源量大，必须尽早建立 manifest、hash、fallback 和按需加载。
- 桌面依赖和 WebView 行为会受 Windows 环境影响，CI 与本机验证都要保留。
- 自动转换不能静默丢内容，未迁移字段必须报告。

## 当前下一步

M0-M4 已完成工程实现与真实数据验收。下一阶段为 M5：验证有限 ERB 子集解释器与纯转换路线的边界；M2 全量草案仍不可玩，必须先完成人工复核与内容治理。
