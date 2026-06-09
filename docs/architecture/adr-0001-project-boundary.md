# ADR-0001: 项目边界与 M0 技术验证

## 状态

Accepted

## 决策

ERAtw-NEXT 是独立新项目，不继承旧 ERAtw 代码、变量、ERB 执行模型、存档格式或 UI 架构。旧项目只能作为只读内容和资产参考源。

M0 使用以下技术路线验证项目可独立运行：

- 桌面壳：Tauri 2。
- 前端：React、TypeScript、Vite。
- UI 状态：Zustand。
- 数据请求边界：TanStack Query 预留。
- UI 基础组件：Radix UI。
- 现代表现层：PixiJS。
- 核心：Rust crate `eratw_engine`，通过 Tauri command API 暴露快照和命令分发。

## 结果

前端不直接修改核心状态。UI 通过 `engine_snapshot` 与 `engine_dispatch` 获取和推进 `WorldState`。浏览器测试环境使用同构 mock adapter；Tauri 环境使用 Rust engine。

## 后续

M1 开始建立旧内容审计工具，输出口上清单、角色清单、资源清单、乱码/语言/许可报告。
