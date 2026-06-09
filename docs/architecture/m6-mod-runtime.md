# M6 Mod Runtime

## 当前结构

- `ModManifest`：声明 namespace、名称、版本、兼容 engine 版本、加载权重、依赖、冲突和能力。
- `ModDependency`：声明依赖 namespace、可选版本约束和是否必需。
- `ModCapability`：当前支持 `content`、`theme`、`rules_extension`，以及默认拒绝的高危能力。
- `ModSecurityPolicy`：记录调用方显式授权的高危能力，默认空策略拒绝 `local_file_access`、`network_access` 和 `system_command`。
- `ModEnablement`：记录玩家/工具层对某个 namespace 的启用或禁用选择。
- `validate_manifest` / `validate_manifest_with_policy`：拒绝空 namespace/name/version/engine_version、重复依赖、重复冲突和未授权高危能力。
- `plan_load_order` / `plan_load_order_with_policy`：在启用 Mod 集合上执行依赖检查、版本检查、冲突检查、循环依赖检查，并输出稳定加载顺序。
- `plan_enabled_mods` / `plan_enabled_mods_for_engine_with_policy`：先应用启用/禁用选择，再对启用集合执行加载计划；被禁用 Mod 单独进入 disabled 列表。
- `read_manifest_file` / `read_manifest_file_with_policy`：读取单个 `manifest.json`，完成 JSON 解析和 manifest 安全校验。
- `scaffold_mod_template`：生成最小可验证 Mod 项目，包含 `manifest.json`、`README.md` 和 `content/character.json`。
- `validate_mod_project` / `validate_mod_project_for_engine`：以 Mod 项目根目录为输入，校验 `manifest.json`、engine 版本和可安装 namespace，输出作者工具可展示的验证报告。
- `package_mod_project` / `package_mod_project_for_engine`：把验证通过的 Mod 项目复制到发布包目录，输出 `eratw-mod-package.json` 包清单和 `content/` 内容根目录。
- `check_mod_package` / `check_mod_package_for_engine`：校验发布包清单、包内 manifest 路径、清单与内容 manifest 的 namespace/version 一致性和 engine 兼容性。
- `plan_mod_install` / `plan_mod_install_for_engine`：读取待安装目录 manifest，校验 engine 版本和安装目标 namespace，生成创建安装目录、复制到 staging、移动到目标目录的计划。
- `preflight_mod_install` / `preflight_mod_package_install`：在写入文件前输出安装预检报告，包含目标目录、staging 残留、安装根类型、包/manifest/engine/能力校验问题。
- `install_mod` / `install_mod_for_engine`：执行安装计划，先复制到 `.installing-{namespace}` staging，成功后移动到正式 namespace 目录；失败时清理 staging，目标已存在时拒绝覆盖。
- `install_mod_package` / `install_mod_package_for_engine`：先校验发布包，再安装包内 `content/` 目录，复用安装 staging 和拒绝覆盖语义。
- `plan_mod_uninstall` / `uninstall_mod`：按 namespace 生成卸载计划，先移动正式目录到 `.uninstalling-{namespace}` staging，再删除 staging，避免半删除状态被发现器当作可加载 Mod。
- `discover_mods` / `discover_mods_for_engine`：扫描 Mod 根目录的一级子目录，分别返回成功发现的 manifest 和每个失败 manifest 的结构化错误。
- `eratw-mod` CLI：提供作者侧 `new`、`validate`、`pack`、`check-package`、`preflight-install-package` 和 `install-package` 命令，作为 Mod SDK 的最小模板/验证/打包/发布检查/预检/安装入口；`--allow-capability <capability>` 用于显式授权受信 Mod 的高危能力。
- Tauri `engine_discover_mods` / `engine_plan_mod_install` / `engine_preflight_mod_package_install` / `engine_install_mod` / `engine_plan_mod_uninstall` / `engine_uninstall_mod` / `engine_plan_enabled_mods`：桌面层把发现报告、安装/卸载计划、预检报告、安装/卸载结果和启用计划转换成前端稳定 DTO，包含可展示的错误类型和消息；安装/启用请求可带 `authorizedUnsafeCapabilities`，未知授权返回 `unknown_capability`。
- 运行时内容包安装成功后会写入 `WorldState.installed_content_packages`，包含 package_id、version、dependencies 和 conflicts；存档外壳据此生成 `mod_dependencies`，为后续 Mod registry 接管启停检查预留稳定入口。

## 安全默认值

- 默认拒绝 `local_file_access`、`network_access` 和 `system_command`；只有调用方通过 `ModSecurityPolicy`、CLI `--allow-capability` 或桌面 API `authorizedUnsafeCapabilities` 显式授权时才放行。
- 缺失必需依赖时不加载；缺失可选依赖时允许继续。
- 依赖版本不匹配、声明冲突或循环依赖都会返回结构化错误。
- 加载顺序先保证依赖在被依赖者之前，再在当前可加载集合内按 `load_order` 和 namespace 排序。
- Mod 目录发现不会因为单个损坏 manifest 阻塞其他 Mod；坏 manifest 进入 discovery errors，好的 manifest 继续进入后续加载计划。
- Mod 目录发现会跳过 `.installing-*` 和 `.uninstalling-*` staging 目录，避免异常残留进入加载计划。
- Mod 模板生成只写入不存在或空目录目标，不覆盖作者已有文件。
- Mod 打包会拒绝输出到源码目录内部，避免递归复制；包版本也不能包含路径分隔符、盘符分隔符、`.` 或 `..`；`.git`、`node_modules`、`target`、`dist`、`build` 和 staging 目录不会进入发布包。
- Mod 发布包检查会拒绝未知包 schema、包清单路径穿越、包清单与内容 manifest 不一致，以及不兼容当前 engine 版本的包。
- Mod 安装计划只允许把 Mod 安装到安装根目录下的 namespace 子目录；namespace 不能为空，也不能包含路径分隔符、盘符分隔符、`.` 或 `..`。
- Mod 安装预检不会写文件，会把包/manifest/engine/能力错误、已存在目标目录、非目录安装根记为 error，把可清理的 install staging 残留记为 warning。
- Mod 安装执行不覆盖已存在目标目录；复制失败时清理 staging，避免半安装目录参与后续发现。
- Mod 发布包安装会先通过发布包检查，再复制包内 `content/`；坏包不会创建安装根目录，也不会覆盖已安装目标。
- Mod 卸载执行要求目标目录存在；卸载前清理同名 uninstall staging，再通过移动到 staging 后删除完成卸载。
- 禁用 Mod 不进入加载顺序；如果启用 Mod 依赖被禁用的必需 Mod，启用计划返回缺失依赖错误。
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

## CLI 示例

```powershell
cargo run -p eratw_mod_cli -- new D:\tmp\my-first-mod --namespace example.my_first_mod --name "我的第一个 Mod"
cargo run -p eratw_mod_cli -- validate examples/mods/minimal-character --engine-version 0.1.0-m0
cargo run -p eratw_mod_cli -- pack examples/mods/minimal-character D:\tmp\eratw-mod-packages --engine-version 0.1.0-m0
cargo run -p eratw_mod_cli -- check-package D:\tmp\eratw-mod-packages\example.minimal_character-0.1.0 --engine-version 0.1.0-m0
cargo run -p eratw_mod_cli -- preflight-install-package D:\tmp\eratw-mod-packages\example.minimal_character-0.1.0 D:\tmp\eratw-installed-mods --engine-version 0.1.0-m0
cargo run -p eratw_mod_cli -- install-package D:\tmp\eratw-mod-packages\example.minimal_character-0.1.0 D:\tmp\eratw-installed-mods --engine-version 0.1.0-m0
cargo run -p eratw_mod_cli -- validate D:\tmp\trusted-mod --allow-capability network_access
```

`pack` 输出目录结构：

```text
example.minimal_character-0.1.0/
  eratw-mod-package.json
  content/
    manifest.json
    ...
```

## 后续

- 将 `eratw_mod_runtime` 接入内容包安装和存档依赖检查。
- 将存档依赖从当前内容包记录升级为完整 Mod manifest registry。
- 增加更完整的错误恢复、安装撤销、卸载回滚和安装前资源/内容预检。
