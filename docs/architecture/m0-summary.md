# M0 阶段总结

## 完成项

- 初始化独立仓库与 `.gitignore`。
- 建立 `apps/desktop`、`crates/engine`、`crates/content`、`crates/mod_runtime`、`packages/ui`、`tools/content-pipeline`、`docs`、`examples`。
- 建立 Tauri 2 + React + TypeScript + Vite 桌面工程。
- 实现 Rust `eratw_engine` mock：`WorldState`、时间、季节、天气、地点、角色、Dialogue、命令分发。
- 实现 Tauri command API：`engine_snapshot`、`engine_dispatch`。
- 实现传统 UI 与现代 UI 原型，两者共享同一 engine 状态。
- 增加前端测试、Rust 单元测试、CI、架构决策记录、内容边界文档、最小 Mod 示例。

## 验证

- `npm run typecheck`：通过。
- `npm run lint`：通过。
- `npm test`：通过。
- `npm run build:web -w @eratw-next/desktop`：通过。
- `npm audit --omit=dev`：0 vulnerabilities。

## 环境限制

当前机器未安装 Rust toolchain，`cargo test --workspace` 和 `rustc --version` 无法执行。CI 已配置 `dtolnay/rust-toolchain@stable`，推送后由远端验证 Rust workspace。

## 已知风险

- PixiJS 当前进入主包，Vite 构建提示首包超过 500 kB；M3 前应改为现代 UI 懒加载或手动 chunk。
- M0 engine 仍是 mock，尚未实现存档、Mod 沙箱、内容包加载和确定性回放框架。
- 文档生成器暂未作为依赖安装，避免在 M0 引入旧 Vite 链路；后续可评估 VitePress 新版本或 Docusaurus。

## 下一阶段输入

M1 需要接入旧 ERAtw 只读路径配置，建立内容审计 CLI，输出 ERB、CSV、resources、sound、font 清单，以及乱码、语言、许可和资源引用报告。
