# ERAtw-NEXT

ERAtw 现代化引擎与桌面应用（modernization engine & desktop app）。**不是**旧 Emuera/ERB 运行时的再打包。

长期方向是新 schema、新 engine、新 UI 和独立内容包；旧内容只作为外部只读参考。

## 当前阶段：M0 工程骨架 + 双模式地图

- 技术栈：Rust workspace + Tauri 2 桌面壳 + React/TypeScript + MUI + Vite。
- 引擎 `eratw_next_engine` 提供 `system_get_status` 与 `map_get_overview`（纯查询，不读盘）。
- 首屏（status）展示项目身份、当前里程碑、能力列表与路径占位。
- 地图（map）支持**运行时随时切换两种显示方式**：
  - **字符画地图**：等宽字符网格 + 连线 + 节点字形 + 标签浮层 + 占用着色 + 缩放 + 键盘方向导航。
  - **SVG 地图**：节点圆形 + 连线 + MUI 浮动标签 + 缩放。
  - 两种模式共享同一份地图数据与选中/区域状态，切换不丢状态。
- JSON Schema 契约：`schemas/system-status.schema.json`、`schemas/map-model.schema.json`，由前端 Ajv 测试实际校验。

> 地图数据为引擎内置的**自有示例数据**（幻想乡题材、自有布局），**不复制** `eratw-content` 的地图。真实数据后续由内容包提供。

## 安装与验证

```powershell
# Node.js 20.19+ / Rust stable
npm install
npm run typecheck
npm test
npm run build
npm run dev:web      # 浏览器预览（使用 mockData）
```

Rust / Tauri 验证（需本地 Rust 工具链；桌面构建在 Windows 上进行）：

```powershell
cargo fmt --check
cargo test --workspace
npm run dev          # Tauri 开发（需 Rust 工具链）
```

> 在浏览器中运行前端时（非 Tauri 环境），引擎客户端自动回退到镜像数据 `src/engine/mockData.ts`，
> 因此 `npm run dev`/`npm run build` 出的前端可独立预览地图与首屏。

## 目录结构

```text
ERAtw-NEXT/
  Cargo.toml                 # Rust workspace
  package.json               # npm workspaces
  schemas/                   # JSON Schema 契约（单一来源）
  crates/engine/             # eratw_next_engine：系统状态 + 地图模型
  apps/desktop/              # @eratw-next/desktop
    src/                     # React + MUI 前端
      engine/                # 客户端 + 镜像数据
      system/                # 系统状态首屏
      map/                   # 双模式地图（geometry / ascii / svg / view）
      settings/              # 设置 context（地图显示方式）
    src-tauri/               # Tauri 壳与 command 桥接
  .github/workflows/ci.yml   # Windows 完整 / Ubuntu 非桌面
  docs/                      # 路线与设计文档
```

## 内容边界

- `D:\AICODE\eratw-content`：外部**只读**内容源，永不复制进本仓库。
- `D:\AICODE\eratw`：可游玩对照版本，仅供人工参考，引擎不读取。
- `D:\AICODE\ERAtw-modern`、`D:\AICODE\ERAtw-native-foundation`：**无关项目**，不作为输入、依赖或迁移来源。

## 安全边界

默认禁止执行外部 ERB / 脚本、禁止下载未知脚本、禁止自动访问未知 URL。
内容审计与 allowlist/路径穿越防护在 M1 细化。

## 路线

详见 [docs/roadmap/modernization-roadmap.md](docs/roadmap/modernization-roadmap.md)。
M0 仅建立可信工程基线（本阶段额外提前接入了地图功能）；M1 起进入只读内容审计与内容迁移。
