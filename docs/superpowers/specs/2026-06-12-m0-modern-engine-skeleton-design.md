# ERAtw-NEXT M0 现代引擎骨架设计

## 结论

M0 从当前空仓库重新建立现代化工程骨架，不恢复、不参考旧 `ERAtw-NEXT` 提交。技术路线为稳定 Rust + Tauri 2 + React/TypeScript + npm，目标是得到一个可启动、可测试、可持续扩展的桌面应用基线。

`D:\AICODE\eratw-content` 是后续外部只读内容源，`D:\AICODE\eratw` 是可游玩对照版本。`D:\AICODE\ERAtw-modern` 和 `D:\AICODE\ERAtw-native-foundation` 明确无关，不能作为输入、依赖或迁移来源。

English keywords: clean skeleton, Rust engine, Tauri desktop shell, React UI, JSON Schema contract, content source boundary.

## 目标

- 建立 Rust workspace、Tauri 桌面壳、React/TypeScript 前端、MUI 应用框架。
- 提供可启动首屏：上方体现未来游戏项目身份，下方展示开发与迁移状态。
- 提供 `system_get_status` Tauri command，由 Rust engine 返回结构化系统状态。
- 建立顶层 JSON Schema 契约：`schemas/system-status.schema.json`。
- 建立基础测试、类型检查、构建和 CI。
- 固化仓库边界、内容边界、安全边界、本地配置和本地 agent 指令策略。

## 非目标

- 不恢复、不参考、不复制旧 `ERAtw-NEXT` 历史提交中的代码、配置或测试。
- 不读取、复制、提交 `eratw-content` 的正文、图片、音频、字体或派生内容。
- 不扫描真实 CSV、ERB、资源、字体或音频。
- 不实现地图、玩法循环、角色互动、存档、Mod 运行时、编辑器或 ERB 解释器。
- 不写入用户配置目录，不保存窗口状态，不产生运行期本地状态。
- 不访问未知 URL，不下载未知脚本，不执行外部 ERB 或脚本。

## 工程结构

```text
ERAtw-NEXT/
  .github/workflows/ci.yml
  .gitignore
  README.md
  Cargo.toml
  Cargo.lock
  package.json
  package-lock.json
  schemas/
    system-status.schema.json
  apps/
    desktop/
      package.json
      index.html
      src/
        App.tsx
        main.tsx
        systemStatus.ts
        App.test.tsx
        test/
      src-tauri/
        Cargo.toml
        tauri.conf.json
        build.rs
        src/
          main.rs
          lib.rs
  crates/
    engine/
      Cargo.toml
      src/lib.rs
  docs/
    roadmap/
      modernization-roadmap.md
    superpowers/specs/
```

不提交项目级 `AGENTS.md`。M0 可以创建本地 `AGENTS.local.md`，但必须加入 `.gitignore`，仅作本机开发参考，不推送远端。

## 命名

- Rust crate：`eratw_next_engine`
- npm workspace package：`@eratw-next/desktop`
- Tauri command 命名规则：`domain_action`
- M0 command：`system_get_status`
- JSON Schema：`system-status/v1`

## Rust Engine

`crates/engine` 是唯一核心 engine crate。M0 只提供纯 Rust 状态查询 API，不依赖 Tauri，不读取磁盘，不接触真实内容目录。

建议核心类型：

- `SystemStatus`
- `BuildInfo`
- `PathPlaceholder`
- `Capability`
- `Milestone`
- `SystemStatusError`

`SystemStatus` 至少包含：

- `schemaVersion`
- `app`
- `engine`
- `build`
- `paths`
- `capabilities`
- `currentMilestone`
- `milestones`

`capabilities` 使用对象数组，形态为 `{ id, label, status, description }`。`milestones` 由 engine status 返回，UI 只负责渲染，不在前端硬编码路线。

## Tauri 边界

`apps/desktop/src-tauri` 只负责桌面壳、窗口配置和 command 桥接。业务逻辑放在 `crates/engine`。

M0 command：

```text
system_get_status() -> Result<SystemStatus, SystemStatusErrorDto>
```

错误对象使用稳定结构：

```json
{
  "code": "SYSTEM_STATUS_UNAVAILABLE",
  "message": "System status is unavailable.",
  "details": {}
}
```

M0 不需要真实错误分支很复杂，但 UI 和测试必须能覆盖错误状态。

## JSON Schema 契约

顶层 `schemas/system-status.schema.json` 是 `system_get_status` 返回值的单一契约来源。M0 先不做 Rust/TypeScript 自动生成，但必须把 schema 作为测试输入。

TypeScript 测试使用 Ajv 校验 status fixture 或 mock output。后续当 API 增多时，再评估从 JSON Schema 生成 Rust/TS 类型，或引入专门的类型生成链路。

## React/MUI 首屏

首屏采用混合形态：

- 顶部：项目名、阶段标识、不可点击或禁用的未来启动入口，避免伪装成已可玩。
- 中部：engine status、当前 milestone、能力列表、路径占位。
- 底部：验证命令、内容边界提示、后续路线摘要。

UI 框架使用 MUI，从 M0 开始建立主题、布局、状态组件和错误组件。样式不引入 Tailwind。普通局部样式只用于 MUI 无法覆盖的少量布局。

默认窗口尺寸为 1600x900，最小尺寸为 1200x760。M0 不保存窗口尺寸和位置。

## 内容源与路径

M0 只展示路径占位，不做存在性断言：

- 内容源：`D:\AICODE\eratw-content`
- 可游玩对照：`D:\AICODE\eratw`
- 无关项目：`D:\AICODE\ERAtw-modern`、`D:\AICODE\ERAtw-native-foundation`

`eratw-content` 永远作为外部只读源存在，不作为 submodule，不复制进引擎仓库。后续转换产物进入独立内容包仓库，不进入 `ERAtw-NEXT` 引擎仓库。

## 本地配置

M0 提供配置示例，不写本地状态：

- 入库：`config.example.toml`
- 忽略：`config.local.toml`
- 后续运行时配置目录：留到 M1/M2 之后接入

`config.example.toml` 只表达字段和默认路径，不要求路径存在。

## 测试

M0 至少包含：

- Rust 单元测试：验证 engine status 的稳定字段、capabilities、milestones。
- Tauri command 逻辑测试：验证 command 桥接返回结构，业务逻辑不写在 command 里。
- React 测试：使用 Vitest + React Testing Library 覆盖 loading、success、error 三类状态。
- Schema 测试：TypeScript + Ajv 校验 status fixture 或 mock output 符合 `system-status/v1`。
- Typecheck/build：验证 TypeScript 和 Vite 构建。

M0 不引入 Playwright 截图测试。等 UI 有真实交互和视觉稳定性要求后再加入。

## CI

GitHub Actions 分两类：

- Windows：完整验证，包括 Rust、npm、React 测试、typecheck、build，以及可行范围内的 Tauri 构建检查。
- Ubuntu：只跑 Rust/TS 非桌面测试，避免过早引入 Linux Tauri 系统依赖复杂度。

CI 权限使用默认最小权限。不做发布、不上传 release artifact。

## 验证命令

M0 完成后至少执行：

```powershell
cargo fmt --check
cargo test --workspace
npm test
npm run typecheck
npm run build
git diff --check
git status --short
```

若本机缺少 Rust、Node、Tauri 或系统依赖，必须记录实际失败原因，不能把未执行或失败的验证写成通过。

## README 验收

M0 README 必须说明：

- 项目定位：ERAtw 现代化引擎，不是旧运行时打包。
- 当前阶段：M0 工程骨架。
- 安装与验证命令。
- 目录结构。
- 内容边界：`eratw-content` 外部只读，不提交内容本体。
- 无关目录：`modern/native` 排除。
- 总路线摘要，并链接 `docs/roadmap/modernization-roadmap.md`。

README 中文优先，关键术语附英文，不要求完整英译。

## Git 与仓库纪律

必须提交：

- 源码
- 测试
- schema
- README
- 配置示例
- CI
- 设计/路线文档
- lockfile

不得提交：

- `node_modules/`
- Rust `target/`
- Vite/Tauri build 输出
- runtime cache/log/save
- 旧 exe/dll
- 压缩包
- `eratw-content` 内容本体或转换副本
- `AGENTS.local.md`
- `config.local.toml`

M0 文档深化提交后按用户确认推送远端。后续实现阶段累积到可验证节点后再提交和推送。

## 安全边界

M0 阶段：

- 禁止执行外部 ERB。
- 禁止执行来自内容目录的脚本。
- 禁止下载未知脚本。
- 禁止自动访问未知 URL。
- 网络失败优先考虑系统代理、TUN、Git/npm/Cargo 代理配置。

M1 内容审计器再细化路径 allowlist、符号链接处理、路径穿越防护和只读扫描策略。

## 与总路线的关系

M0 只建立可信工程基线。M1-M8 的方向写入 `docs/roadmap/modernization-roadmap.md`。M0 不提前实现 M1 的内容扫描，也不把长期路线压进首阶段代码。

## 退出标准

M0 可结束的条件：

- 桌面应用能启动并显示系统状态首屏。
- `system_get_status` 能从 Rust engine 返回结构化状态。
- `schemas/system-status.schema.json` 存在，并被 TS 测试实际校验。
- README、`.gitignore`、`config.example.toml`、CI、测试、lockfile 完整。
- `AGENTS.local.md` 如存在，已被 `.gitignore` 忽略且未暂存。
- 验证命令按实际环境执行并记录结果。
