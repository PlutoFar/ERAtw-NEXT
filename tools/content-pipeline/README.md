# Content Pipeline

Python tooling stays out of runtime. This package is for offline content audit, reference extraction, validation, and packaging.

## Install

```powershell
python -m pip install -e "tools/content-pipeline[test]"
```

## Audit Legacy ERAtw

```powershell
python -m eratw_content_pipeline.cli audit-legacy --source D:\AICODE\ERAtw --out reports\legacy-audit
```

Generated files:

- `legacy-audit-report.json`
- `legacy-file-inventory.csv`
- `asset-manifest.draft.json`
- `summary.md`

The audit treats old ERB and CSV as read-only reference material. It records legacy runtime binaries and saves as excluded artifacts.

## Planned Commands

- `validate-pack`: validate versioned JSON/YAML authoring content.
- `pack`: compile validated content into a release package.
