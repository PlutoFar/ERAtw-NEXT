from __future__ import annotations

import json
from pathlib import Path

from eratw_content_pipeline.cli import main
from eratw_content_pipeline.legacy_audit import AuditOptions, audit_legacy_source


def write_bytes(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def test_audit_legacy_source_classifies_files(tmp_path: Path) -> None:
    source = tmp_path / "legacy"
    write_bytes(source / "ERB" / "demo.ERB", "@EVENT\nPRINTFORML 你好 image.webp\n".encode("utf-8"))
    write_bytes(source / "CSV" / "Chara.csv", "id,name\n1,示例\n".encode("utf-8"))
    write_bytes(source / "resources" / "image.webp", b"fake-webp")
    write_bytes(source / "sound" / "theme.mp3", b"fake-mp3")
    write_bytes(source / "sav" / "old.sav", b"legacy-save")

    report = audit_legacy_source(AuditOptions(source=source, out=tmp_path / "out"))

    assert report.summary["erb"] == 1
    assert report.summary["csv"] == 1
    assert report.summary["image"] == 1
    assert report.summary["audio"] == 1
    assert report.summary["legacy_runtime"] == 1
    assert report.resource_reference_summary["image.webp"] == 1
    assert any(asset.resource_id == "legacy.resources.image" for asset in report.assets)
    assert any(issue.code == "excluded_runtime_artifact" for issue in report.issues)


def test_audit_legacy_cli_writes_reports(tmp_path: Path) -> None:
    source = tmp_path / "legacy"
    out = tmp_path / "reports"
    write_bytes(source / "ERB" / "demo.ERB", "$LABEL\nPRINTFORML こんにちは\n".encode("utf-8"))
    write_bytes(source / "font" / "main.ttf", b"fake-font")

    exit_code = main(["audit-legacy", "--source", str(source), "--out", str(out)])

    assert exit_code == 0
    report = json.loads((out / "legacy-audit-report.json").read_text(encoding="utf-8"))
    manifest = json.loads((out / "asset-manifest.draft.json").read_text(encoding="utf-8"))
    assert report["schema_version"] == "legacy-audit/v0"
    assert report["files"][0]["path"] == "ERB/demo.ERB"
    assert manifest["schemaVersion"] == "asset-manifest/v0"
    assert (out / "legacy-file-inventory.csv").exists()
    assert (out / "summary.md").exists()
