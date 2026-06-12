# ERAtw-NEXT M0 现代引擎骨架设计

## 结论

本阶段从当前空仓库重新建立现代化工程骨架。历史提交只作为参考资料，不直接恢复代码；`D:\AICODE\eratw-content` 是后续内容源，`D:\AICODE\eratw` 是可游玩对照版本；`D:\AICODE\ERAtw-modern` 和 `D:\AICODE\ERAtw-native-foundation` 明确不纳入本项目输入。

English summary: M0 rebuilds a clean modern skeleton with Rust, Tauri, and React. Legacy content is referenced only by path metadata for now.

## 目标

- 建立可启动、可测试、可持续扩展的 Rust + Tauri + React/TypeScript 工程基线。
- 首屏显示项目状态、引擎状态、版本信息和内容源配置占位。
- 为后续内容审计、CSV/ERB 迁移、资源管理、存档和 Mod 系统保留清晰边界。
- 建立基础 CI、验证命令、忽略规则和仓库说明。

## 非目标

- 不恢复旧 `ERAtw-NEXT` 提交中的实现代码。
- 不读取、复制或提交 `eratw-content` 的正文、图片、音频、字体等内容本体。
- 不接入真实 CSV、ERB、资源解析。
- 不实现地图、玩法循环、角色互动、存档、Mod 运行时或 ERB 兼容层。
- 不使用 `ERAtw-modern` 或 `ERAtw-native-foundation` 作为依赖或迁移来源。

## 工程结构

```text
ERAtw-NEXT/
  .github/workflows/ci.yml
  .gitignore
  AGENTS.md
  README.md
  Cargo.toml
  package.json
  apps/
    desktop/
      package.json
      src/
      src-tauri/
  crates/
    engine/
  docs/
    architecture/
    superpowers/specs/
```

`crates/engine` 是唯一核心引擎 crate。M0 只包含稳定的状态查询接口，例如项目版本、引擎状态、配置路径占位和能力标记。它不依赖 Tauri，也不读取本地内容目录。

`apps/desktop/src-tauri` 负责桌面壳和命令桥接。Tauri command 只暴露 engine status，不承载业务逻辑。

`apps/desktop/src` 负责 React 首屏。UI 只消费 Tauri command 返回的结构化状态，保留后续本地配置、内容审计、运行模式切换的入口位置。

## 数据流

1. 桌面应用启动 React 首屏。
2. React 调用 Tauri command 请求 engine status。
3. Tauri command 调用 `crates/engine` 的纯 Rust API。
4. UI 渲染项目状态、版本、内容源占位和下一阶段提示。

M0 不扫描磁盘内容。内容源路径只作为默认配置文本或空状态展示，避免把 `eratw-content` 误当作子模块、依赖或打包资源。

## 错误处理

- Tauri command 返回结构化错误，不向 UI 抛未格式化异常。
- UI 为状态加载、失败和空配置分别提供明确视图。
- 所有文件路径在 M0 阶段只显示占位，不做存在性断言。

## 验证

M0 完成后必须至少通过：

- `cargo fmt --check`
- `cargo test --workspace`
- `npm test`
- `npm run typecheck`
- `npm run build`
- `git diff --check`

若本机缺少 Tauri、Rust、Node 或系统依赖，记录实际失败原因，不伪报通过。

## 仓库与发布纪律

- 提交源码、配置、文档和测试。
- 不提交 `node_modules/`、`target/`、`dist/`、build 输出、缓存、日志、存档、旧 exe/dll、压缩包。
- 累积到可验证阶段后提交并推送。
- 网络失败时优先检查系统代理、TUN、Git、npm 和 Cargo 代理配置。

## 后续阶段预留

- M1：只读内容审计器，统计 `eratw-content` 中 CSV、ERB、资源、字体、音频和文档。
- M2：内容 schema 与资源 manifest 草案。
- M3：最小内容包加载与桌面展示。
- M4：玩法状态机、存档和事件调度。
- M5：选择性 ERB 迁移或兼容实验。
