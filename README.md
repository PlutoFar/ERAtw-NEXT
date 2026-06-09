# ERAtw-NEXT

ERAtw-NEXT is a new, independent modernization project. The legacy ERAtw project is treated only as a read-only content and asset reference source.

## Current Milestone

M0 proves the new project can stand alone:

- Tauri 2 + React + TypeScript + Vite desktop shell
- Rust engine mock exposed through Tauri commands
- Traditional and modern UI mode prototype sharing the same state
- Workspace structure for engine, content, mod runtime, tools, docs, and examples
- CI, tests, architecture notes, and content boundary notes

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
