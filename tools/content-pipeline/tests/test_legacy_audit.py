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
    write_bytes(source / "ERB" / "DIM.ERH", "#DIM WORLD_STATE\n".encode("utf-8"))
    write_bytes(source / "CSV" / "Chara" / "Chara1 霊夢.csv", "番号,1,\n名前,博麗 霊夢,\n呼び名,霊夢,\n".encode("utf-8"))
    write_bytes(source / "CSV" / "Train.csv", "id,name\n1,示例\n".encode("utf-8"))
    write_bytes(source / "ERB" / "キャラデータ" / "Chara_data_1_霊夢.ERB", "@DATA\n".encode("utf-8"))
    write_bytes(
        source / "ERB" / "口上・メッセージ関連" / "個人口上" / "001 Reimu [霊夢]" / "talk.ERB",
        "@TALK\n$HELLO\nPRINTFORML 你好\n".encode("utf-8"),
    )
    write_bytes(source / "resources" / "image.webp", b"fake-webp")
    write_bytes(source / "sound" / "theme.mp3", b"fake-mp3")
    write_bytes(source / "sound" / "theme.mid", b"fake-midi")
    write_bytes(source / "docs" / "guide.pdf", b"fake-pdf")
    write_bytes(source / "patches" / "old.zip", b"fake-zip")
    write_bytes(source / "mkResourceXml.py", b"print('tool')\n")
    write_bytes(source / "sav" / "old.sav", b"legacy-save")

    report = audit_legacy_source(AuditOptions(source=source, out=tmp_path / "out"))

    assert report.summary["erb"] == 3
    assert report.summary["csv"] == 2
    assert report.summary["legacy_header"] == 1
    assert report.summary["image"] == 1
    assert report.summary["audio"] == 2
    assert report.summary["document"] == 1
    assert report.summary["archive"] == 1
    assert report.summary["tool_script"] == 1
    assert report.summary["legacy_runtime"] == 1
    assert report.summary["characters"] == 1
    assert report.summary["characters_with_dialogue"] == 1
    assert report.summary["dialogue_files"] == 1
    assert report.summary["resource_refs"] == 1
    assert report.summary["resource_refs_matched"] == 1
    assert report.summary["resource_refs_missing"] == 0
    assert report.resource_reference_summary["image.webp"] == 1
    assert report.resource_references[0].status == "matched"
    assert report.resource_references[0].matched_asset_paths == ["resources/image.webp"]
    assert report.characters[0].name == "博麗 霊夢"
    assert report.characters[0].call_name == "霊夢"
    assert report.characters[0].data_erb_paths == ["ERB/キャラデータ/Chara_data_1_霊夢.ERB"]
    assert report.characters[0].dialogue_paths == ["ERB/口上・メッセージ関連/個人口上/001 Reimu [霊夢]/talk.ERB"]
    assert report.dialogues[0].owner_legacy_id == 1
    assert report.dialogues[0].kind == "personal_dialogue"
    assert report.dialogue_coverage[0].legacy_id == 1
    assert report.dialogue_coverage[0].dialogue_file_count == 1
    assert report.dialogue_coverage[0].dialogue_line_count == 3
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
    characters = json.loads((out / "character-inventory.json").read_text(encoding="utf-8"))
    dialogues = json.loads((out / "dialogue-inventory.json").read_text(encoding="utf-8"))
    resource_refs = json.loads((out / "resource-reference-report.json").read_text(encoding="utf-8"))
    coverage = json.loads((out / "dialogue-coverage-report.json").read_text(encoding="utf-8"))
    assert report["schema_version"] == "legacy-audit/v0"
    assert report["files"][0]["path"] == "ERB/demo.ERB"
    assert manifest["schemaVersion"] == "asset-manifest/v0"
    assert characters["schemaVersion"] == "character-inventory/v0"
    assert dialogues["schemaVersion"] == "dialogue-inventory/v0"
    assert resource_refs["schemaVersion"] == "resource-reference-report/v0"
    assert coverage["schemaVersion"] == "dialogue-coverage-report/v0"
    assert (out / "character-inventory.csv").exists()
    assert (out / "dialogue-inventory.csv").exists()
    assert (out / "resource-reference-report.csv").exists()
    assert (out / "dialogue-coverage-report.csv").exists()
    assert (out / "legacy-file-inventory.csv").exists()
    assert (out / "summary.md").exists()
