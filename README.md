# ERAtw-NEXT

ERAtw-NEXT is a new, independent modernization project. The legacy ERAtw project is treated only as a read-only content and asset reference source.

## Current Milestone

M0 proves the new project can stand alone:

- Tauri 2 + React + TypeScript + Vite desktop shell
- Rust engine mock exposed through Tauri commands
- Traditional and modern UI mode prototype sharing the same state
- Workspace structure for engine, content, mod runtime, tools, docs, and examples
- CI, tests, architecture notes, and content boundary notes
- M1 legacy content audit CLI and draft asset manifest output
- M2 save envelope foundation in the Rust engine
- M2 deterministic event scheduler with command replay tests
- Content packages can add locations, characters, relationships, resources, dialogue, and scheduled events
- Content package installation records save dependencies and enforces package dependency/conflict declarations
- Resource load planning reports safe paths, missing assets, hash mismatches, and fallback modes
- Mod manifest validation with dependency, conflict, unsafe capability, and load-order checks
- Mod manifest discovery reports good and broken local Mod directories independently
- Desktop engine API exposes Mod discovery reports to the frontend contract
- Mod enablement planning keeps disabled Mods out of load order and reports dependency failures
- Mod install planning validates target namespace and emits planned filesystem actions
- Mod install execution stages copies before moving into the final namespace directory
- Mod uninstall planning and execution move installs through uninstall staging before deletion
- Mod CLI validates projects and packages example Mods into release directories

## Development

```powershell
npm install
npm test
npm run typecheck
npm run build
```

Tauri development additionally requires a local Rust toolchain.

```powershell
npm run dev
```

## Mod CLI

```powershell
cargo run -p eratw_mod_cli -- new D:\tmp\my-first-mod --namespace example.my_first_mod --name "我的第一个 Mod"
cargo run -p eratw_mod_cli -- validate examples/mods/minimal-character --engine-version 0.1.0-m0
cargo run -p eratw_mod_cli -- pack examples/mods/minimal-character D:\tmp\eratw-mod-packages --engine-version 0.1.0-m0
```

## Content Audit

```powershell
python -m pip install -e "tools/content-pipeline[test]"
python -m eratw_content_pipeline.cli audit-legacy --source D:\AICODE\ERAtw --out reports\legacy-audit
```
