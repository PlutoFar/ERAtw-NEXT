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
- Mod manifest validation with dependency, conflict, unsafe capability, and load-order checks

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

## Content Audit

```powershell
python -m pip install -e "tools/content-pipeline[test]"
python -m eratw_content_pipeline.cli audit-legacy --source D:\AICODE\ERAtw --out reports\legacy-audit
```
