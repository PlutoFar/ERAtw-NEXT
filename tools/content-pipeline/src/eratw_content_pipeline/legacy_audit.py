from __future__ import annotations

import csv
import hashlib
import json
import re
from collections import Counter
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable


ERB_SUFFIXES = {".erb"}
ERH_SUFFIXES = {".erh", ".erd"}
CSV_SUFFIXES = {".csv"}
IMAGE_SUFFIXES = {".png", ".jpg", ".jpeg", ".webp", ".bmp", ".gif"}
AUDIO_SUFFIXES = {".ogg", ".mp3", ".wav", ".flac", ".m4a", ".mid", ".midi"}
FONT_SUFFIXES = {".ttf", ".otf", ".ttc", ".woff", ".woff2"}
TEXT_SUFFIXES = {".txt", ".md", ".json", ".cfg", ".config", ".dat", ".log", ".xml", ".khp"}
ARCHIVE_SUFFIXES = {".zip", ".7z", ".rar"}
DOCUMENT_SUFFIXES = {".docx", ".pdf", ".xls", ".xlsx"}
TOOL_SCRIPT_SUFFIXES = {".py", ".bat", ".ps1"}

LEGACY_RUNTIME_SUFFIXES = {".exe", ".dll", ".sav"}

MOJIBAKE_PATTERNS = ("�", "ã", "Ã", "繧", "縺", "蜿", "譁", "荳")
RESOURCE_REF_RE = re.compile(
    r"(?P<name>[\w\-.一-龥ぁ-んァ-ヶー]+?\.(?:png|jpe?g|webp|bmp|gif|ogg|mp3|wav|ttf|otf))",
    re.IGNORECASE,
)
ERB_FUNCTION_RE = re.compile(r"^\s*@(?P<name>[A-Za-z0-9_\-\u3040-\u30ff\u3400-\u9fff]+)", re.MULTILINE)
ERB_LABEL_RE = re.compile(r"^\s*\$(?P<name>[A-Za-z0-9_\-\u3040-\u30ff\u3400-\u9fff]+)", re.MULTILINE)
CHARA_CSV_RE = re.compile(r"^Chara(?P<number>\d+)(?:\s+(?P<name>.+))?\.csv$", re.IGNORECASE)
CHARA_DATA_ERB_RE = re.compile(r"^Chara_data_(?P<number>\d+)_(?P<name>.+)\.erb$", re.IGNORECASE)
PERSONAL_DIALOGUE_DIR_RE = re.compile(r"^(?P<number>\d{3})\s+(?P<latin>.+?)\s+\[(?P<name>.+)]$")


@dataclass(frozen=True)
class AuditOptions:
    source: Path
    out: Path
    sample_text_bytes: int = 8192
    max_issues: int = 200


@dataclass
class FileRecord:
    path: str
    category: str
    size_bytes: int
    sha256: str | None = None
    encoding: str | None = None
    language_hint: str | None = None
    line_count: int | None = None
    issue_flags: list[str] = field(default_factory=list)
    resource_refs: list[str] = field(default_factory=list)
    erb_functions: list[str] = field(default_factory=list)
    erb_labels: list[str] = field(default_factory=list)


@dataclass
class AssetManifestItem:
    resource_id: str
    source_path: str
    media_type: str
    size_bytes: int
    sha256: str
    license: str = "unknown"
    author: str = "unknown"
    usage: list[str] = field(default_factory=list)
    character_bindings: list[str] = field(default_factory=list)
    tags: list[str] = field(default_factory=list)


@dataclass
class CharacterInventoryItem:
    legacy_id: int
    name: str
    call_name: str | None
    csv_path: str
    data_erb_paths: list[str] = field(default_factory=list)
    dialogue_paths: list[str] = field(default_factory=list)
    language_hint: str | None = None
    issue_flags: list[str] = field(default_factory=list)


@dataclass
class DialogueInventoryItem:
    path: str
    kind: str
    owner_legacy_id: int | None
    owner_name: str | None
    size_bytes: int
    line_count: int | None
    encoding: str | None
    language_hint: str | None
    function_count: int
    label_count: int
    resource_ref_count: int
    issue_flags: list[str] = field(default_factory=list)


@dataclass
class ResourceReferenceItem:
    reference: str
    count: int
    matched_asset_paths: list[str] = field(default_factory=list)
    status: str = "missing"


@dataclass
class DialogueCoverageItem:
    legacy_id: int
    name: str
    dialogue_file_count: int
    dialogue_line_count: int
    issue_file_count: int
    missing_resource_ref_count: int
    language_hints: list[str] = field(default_factory=list)


@dataclass
class AuditIssue:
    severity: str
    code: str
    path: str
    message: str


@dataclass
class LegacyAuditReport:
    schema_version: str
    generated_at: str
    source_root: str
    summary: dict[str, int]
    files: list[FileRecord]
    assets: list[AssetManifestItem]
    characters: list[CharacterInventoryItem]
    dialogues: list[DialogueInventoryItem]
    resource_references: list[ResourceReferenceItem]
    dialogue_coverage: list[DialogueCoverageItem]
    issues: list[AuditIssue]
    resource_reference_summary: dict[str, int]


def audit_legacy_source(options: AuditOptions) -> LegacyAuditReport:
    source = options.source.resolve()
    if not source.exists() or not source.is_dir():
        raise FileNotFoundError(f"legacy source directory not found: {source}")

    records: list[FileRecord] = []
    assets: list[AssetManifestItem] = []
    issues: list[AuditIssue] = []
    referenced_resources: Counter[str] = Counter()

    for path in iter_files(source):
        relative_path = normalize_relative(path.relative_to(source))
        category = categorize_file(path)
        digest = sha256_file(path)
        stat = path.stat()
        record = FileRecord(
            path=relative_path,
            category=category,
            size_bytes=stat.st_size,
            sha256=digest,
        )
        setattr(record, "_source_path", str(path))

        if category in {"erb", "legacy_header", "csv", "text"}:
            inspect_text_file(path, record, options.sample_text_bytes)
            referenced_resources.update(record.resource_refs)
            issues.extend(text_issues(record))

        if category in {"image", "audio", "font"}:
            assets.append(
                AssetManifestItem(
                    resource_id=resource_id_for(relative_path),
                    source_path=relative_path,
                    media_type=category,
                    size_bytes=stat.st_size,
                    sha256=digest,
                    tags=derive_asset_tags(path),
                )
            )

        if category == "legacy_runtime":
            record.issue_flags.append("excluded_runtime_artifact")
            issues.append(
                AuditIssue(
                    severity="info",
                    code="excluded_runtime_artifact",
                    path=relative_path,
                    message="旧运行时、DLL 或存档只能作为排除项记录，不得进入新运行时。",
                )
            )

        records.append(record)

    summary_counter = Counter(record.category for record in records)
    characters = build_character_inventory(records)
    dialogues = build_dialogue_inventory(records)
    resource_references = build_resource_reference_report(assets, referenced_resources)
    dialogue_coverage = build_dialogue_coverage_report(characters, dialogues, records, resource_references)
    summary_counter["total_files"] = len(records)
    summary_counter["total_size_bytes"] = sum(record.size_bytes for record in records)
    summary_counter["characters"] = len(characters)
    summary_counter["dialogue_files"] = len(dialogues)
    summary_counter["resource_refs"] = len(resource_references)
    summary_counter["resource_refs_matched"] = sum(
        1 for reference in resource_references if reference.status == "matched"
    )
    summary_counter["resource_refs_missing"] = sum(
        1 for reference in resource_references if reference.status == "missing"
    )
    summary_counter["characters_with_dialogue"] = sum(
        1 for coverage in dialogue_coverage if coverage.dialogue_file_count > 0
    )

    return LegacyAuditReport(
        schema_version="legacy-audit/v0",
        generated_at=datetime.now(timezone.utc).isoformat(),
        source_root=str(source),
        summary=dict(sorted(summary_counter.items())),
        files=records,
        assets=assets,
        characters=characters,
        dialogues=dialogues,
        resource_references=resource_references,
        dialogue_coverage=dialogue_coverage,
        issues=issues[: options.max_issues],
        resource_reference_summary=dict(sorted(referenced_resources.items())),
    )


def write_audit_outputs(report: LegacyAuditReport, out_dir: Path) -> list[Path]:
    out_dir.mkdir(parents=True, exist_ok=True)
    written: list[Path] = []

    report_json = out_dir / "legacy-audit-report.json"
    report_json.write_text(
        json.dumps(asdict(report), ensure_ascii=False, indent=2),
        encoding="utf-8",
    )
    written.append(report_json)

    asset_manifest = out_dir / "asset-manifest.draft.json"
    asset_manifest.write_text(
        json.dumps(
            {
                "schemaVersion": "asset-manifest/v0",
                "sourceRoot": report.source_root,
                "assets": [asdict(asset) for asset in report.assets],
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    written.append(asset_manifest)

    character_json = out_dir / "character-inventory.json"
    character_json.write_text(
        json.dumps(
            {
                "schemaVersion": "character-inventory/v0",
                "sourceRoot": report.source_root,
                "characters": [asdict(character) for character in report.characters],
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    written.append(character_json)

    character_csv = out_dir / "character-inventory.csv"
    with character_csv.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "legacy_id",
                "name",
                "call_name",
                "csv_path",
                "data_erb_paths",
                "dialogue_paths",
                "language_hint",
                "issue_flags",
            ],
        )
        writer.writeheader()
        for character in report.characters:
            writer.writerow(
                {
                    "legacy_id": character.legacy_id,
                    "name": character.name,
                    "call_name": character.call_name,
                    "csv_path": character.csv_path,
                    "data_erb_paths": ";".join(character.data_erb_paths),
                    "dialogue_paths": ";".join(character.dialogue_paths),
                    "language_hint": character.language_hint,
                    "issue_flags": ";".join(character.issue_flags),
                }
            )
    written.append(character_csv)

    dialogue_json = out_dir / "dialogue-inventory.json"
    dialogue_json.write_text(
        json.dumps(
            {
                "schemaVersion": "dialogue-inventory/v0",
                "sourceRoot": report.source_root,
                "dialogues": [asdict(dialogue) for dialogue in report.dialogues],
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    written.append(dialogue_json)

    dialogue_csv = out_dir / "dialogue-inventory.csv"
    with dialogue_csv.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "path",
                "kind",
                "owner_legacy_id",
                "owner_name",
                "size_bytes",
                "line_count",
                "encoding",
                "language_hint",
                "function_count",
                "label_count",
                "resource_ref_count",
                "issue_flags",
            ],
        )
        writer.writeheader()
        for dialogue in report.dialogues:
            writer.writerow(asdict(dialogue))
    written.append(dialogue_csv)

    resource_reference_json = out_dir / "resource-reference-report.json"
    resource_reference_json.write_text(
        json.dumps(
            {
                "schemaVersion": "resource-reference-report/v0",
                "sourceRoot": report.source_root,
                "references": [asdict(reference) for reference in report.resource_references],
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    written.append(resource_reference_json)

    resource_reference_csv = out_dir / "resource-reference-report.csv"
    with resource_reference_csv.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "reference",
                "count",
                "status",
                "matched_asset_paths",
            ],
        )
        writer.writeheader()
        for reference in report.resource_references:
            writer.writerow(
                {
                    "reference": reference.reference,
                    "count": reference.count,
                    "status": reference.status,
                    "matched_asset_paths": ";".join(reference.matched_asset_paths),
                }
    )
    written.append(resource_reference_csv)

    dialogue_coverage_json = out_dir / "dialogue-coverage-report.json"
    dialogue_coverage_json.write_text(
        json.dumps(
            {
                "schemaVersion": "dialogue-coverage-report/v0",
                "sourceRoot": report.source_root,
                "coverage": [asdict(item) for item in report.dialogue_coverage],
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    written.append(dialogue_coverage_json)

    dialogue_coverage_csv = out_dir / "dialogue-coverage-report.csv"
    with dialogue_coverage_csv.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "legacy_id",
                "name",
                "dialogue_file_count",
                "dialogue_line_count",
                "issue_file_count",
                "missing_resource_ref_count",
                "language_hints",
            ],
        )
        writer.writeheader()
        for item in report.dialogue_coverage:
            writer.writerow(
                {
                    "legacy_id": item.legacy_id,
                    "name": item.name,
                    "dialogue_file_count": item.dialogue_file_count,
                    "dialogue_line_count": item.dialogue_line_count,
                    "issue_file_count": item.issue_file_count,
                    "missing_resource_ref_count": item.missing_resource_ref_count,
                    "language_hints": ";".join(item.language_hints),
                }
            )
    written.append(dialogue_coverage_csv)

    csv_path = out_dir / "legacy-file-inventory.csv"
    with csv_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "path",
                "category",
                "size_bytes",
                "sha256",
                "encoding",
                "language_hint",
                "line_count",
                "issue_flags",
            ],
        )
        writer.writeheader()
        for record in report.files:
            writer.writerow(
                {
                    "path": record.path,
                    "category": record.category,
                    "size_bytes": record.size_bytes,
                    "sha256": record.sha256,
                    "encoding": record.encoding,
                    "language_hint": record.language_hint,
                    "line_count": record.line_count,
                    "issue_flags": ";".join(record.issue_flags),
                }
            )
    written.append(csv_path)

    summary_md = out_dir / "summary.md"
    summary_md.write_text(render_markdown_summary(report), encoding="utf-8")
    written.append(summary_md)

    return written


def render_markdown_summary(report: LegacyAuditReport) -> str:
    lines = [
        "# Legacy ERAtw Audit Summary",
        "",
        f"- Schema: `{report.schema_version}`",
        f"- Source: `{report.source_root}`",
        f"- Generated: `{report.generated_at}`",
        "",
        "## Counts",
        "",
    ]

    for key, value in report.summary.items():
        lines.append(f"- {key}: {value}")

    lines.extend(["", "## Issues", ""])
    if not report.issues:
        lines.append("- No issues detected in sampled files.")
    else:
        for issue in report.issues[:50]:
            lines.append(f"- `{issue.severity}` `{issue.code}` `{issue.path}`: {issue.message}")

    lines.extend(
        [
            "",
            "## Content Inventories",
            "",
            f"- Characters: {len(report.characters)}",
            f"- Characters with personal dialogue: {report.summary.get('characters_with_dialogue', 0)}",
            f"- Dialogue/reference ERB files: {len(report.dialogues)}",
            "- Character inventory: `character-inventory.csv`",
            "- Dialogue inventory: `dialogue-inventory.csv`",
            "- Dialogue coverage report: `dialogue-coverage-report.csv`",
            "",
            "## Resource References",
            "",
            f"- Unique referenced resource names: {len(report.resource_reference_summary)}",
            f"- Matched resource references: {report.summary.get('resource_refs_matched', 0)}",
            f"- Missing resource references: {report.summary.get('resource_refs_missing', 0)}",
            "- Resource reference report: `resource-reference-report.csv`",
            "- Draft asset manifest: `asset-manifest.draft.json`",
        ]
    )
    return "\n".join(lines) + "\n"


def iter_files(root: Path) -> Iterable[Path]:
    for path in sorted(root.rglob("*")):
        if path.is_file():
            yield path


def categorize_file(path: Path) -> str:
    suffix = path.suffix.lower()
    parts = {part.lower() for part in path.parts}

    if suffix in ERB_SUFFIXES:
        return "erb"
    if suffix in ERH_SUFFIXES:
        return "legacy_header"
    if suffix in CSV_SUFFIXES:
        return "csv"
    if suffix in IMAGE_SUFFIXES:
        return "image"
    if suffix in AUDIO_SUFFIXES:
        return "audio"
    if suffix in FONT_SUFFIXES:
        return "font"
    if suffix in LEGACY_RUNTIME_SUFFIXES or "sav" in parts:
        return "legacy_runtime"
    if suffix in ARCHIVE_SUFFIXES:
        return "archive"
    if suffix in DOCUMENT_SUFFIXES:
        return "document"
    if suffix in TOOL_SCRIPT_SUFFIXES:
        return "tool_script"
    if suffix in TEXT_SUFFIXES:
        return "text"
    return "other"


def build_character_inventory(records: list[FileRecord]) -> list[CharacterInventoryItem]:
    csv_records: dict[int, FileRecord] = {}
    data_erb_by_id: dict[int, list[str]] = {}
    dialogue_by_id: dict[int, list[str]] = {}

    for record in records:
        path = Path(record.path)
        if record.category == "csv" and len(path.parts) >= 2 and path.parts[-2].lower() == "chara":
            match = CHARA_CSV_RE.match(path.name)
            if match:
                csv_records[int(match.group("number"))] = record
            continue

        if record.category != "erb":
            continue

        data_match = CHARA_DATA_ERB_RE.match(path.name)
        if data_match:
            data_erb_by_id.setdefault(int(data_match.group("number")), []).append(record.path)

        owner = infer_personal_dialogue_owner(path)
        if owner:
            legacy_id, _owner_name = owner
            dialogue_by_id.setdefault(legacy_id, []).append(record.path)

    characters: list[CharacterInventoryItem] = []
    for legacy_id, record in sorted(csv_records.items()):
        name, call_name = parse_character_csv_identity(record)
        characters.append(
            CharacterInventoryItem(
                legacy_id=legacy_id,
                name=name or infer_name_from_chara_csv_path(record.path) or f"legacy:{legacy_id}",
                call_name=call_name,
                csv_path=record.path,
                data_erb_paths=sorted(data_erb_by_id.get(legacy_id, [])),
                dialogue_paths=sorted(dialogue_by_id.get(legacy_id, [])),
                language_hint=record.language_hint,
                issue_flags=record.issue_flags.copy(),
            )
        )

    return characters


def build_dialogue_inventory(records: list[FileRecord]) -> list[DialogueInventoryItem]:
    dialogues: list[DialogueInventoryItem] = []
    for record in records:
        if record.category != "erb" or not is_dialogue_candidate(record.path):
            continue

        owner = infer_personal_dialogue_owner(Path(record.path))
        dialogues.append(
            DialogueInventoryItem(
                path=record.path,
                kind=classify_dialogue_kind(record.path),
                owner_legacy_id=owner[0] if owner else None,
                owner_name=owner[1] if owner else None,
                size_bytes=record.size_bytes,
                line_count=record.line_count,
                encoding=record.encoding,
                language_hint=record.language_hint,
                function_count=len(record.erb_functions),
                label_count=len(record.erb_labels),
                resource_ref_count=len(record.resource_refs),
                issue_flags=record.issue_flags.copy(),
            )
        )

    return sorted(dialogues, key=lambda item: item.path)


def build_resource_reference_report(
    assets: list[AssetManifestItem],
    referenced_resources: Counter[str],
) -> list[ResourceReferenceItem]:
    assets_by_name: dict[str, list[str]] = {}
    for asset in assets:
        assets_by_name.setdefault(Path(asset.source_path).name.lower(), []).append(asset.source_path)

    references: list[ResourceReferenceItem] = []
    for reference, count in sorted(referenced_resources.items(), key=lambda item: item[0].lower()):
        matched_paths = sorted(assets_by_name.get(reference.lower(), []))
        references.append(
            ResourceReferenceItem(
                reference=reference,
                count=count,
                matched_asset_paths=matched_paths,
                status="matched" if matched_paths else "missing",
            )
        )

    return references


def build_dialogue_coverage_report(
    characters: list[CharacterInventoryItem],
    dialogues: list[DialogueInventoryItem],
    records: list[FileRecord],
    resource_references: list[ResourceReferenceItem],
) -> list[DialogueCoverageItem]:
    dialogues_by_owner: dict[int, list[DialogueInventoryItem]] = {}
    for dialogue in dialogues:
        if dialogue.owner_legacy_id is None:
            continue
        dialogues_by_owner.setdefault(dialogue.owner_legacy_id, []).append(dialogue)

    records_by_path = {record.path: record for record in records}
    missing_refs = {
        reference.reference.lower()
        for reference in resource_references
        if reference.status == "missing"
    }

    coverage: list[DialogueCoverageItem] = []
    for character in characters:
        owned_dialogues = dialogues_by_owner.get(character.legacy_id, [])
        language_hints = sorted(
            {
                dialogue.language_hint
                for dialogue in owned_dialogues
                if dialogue.language_hint is not None
            }
        )
        missing_for_character: set[str] = set()
        for dialogue in owned_dialogues:
            record = records_by_path.get(dialogue.path)
            if not record:
                continue
            missing_for_character.update(
                ref for ref in record.resource_refs if ref.lower() in missing_refs
            )

        coverage.append(
            DialogueCoverageItem(
                legacy_id=character.legacy_id,
                name=character.name,
                dialogue_file_count=len(owned_dialogues),
                dialogue_line_count=sum(dialogue.line_count or 0 for dialogue in owned_dialogues),
                issue_file_count=sum(1 for dialogue in owned_dialogues if dialogue.issue_flags),
                missing_resource_ref_count=len(missing_for_character),
                language_hints=language_hints,
            )
        )

    return sorted(
        coverage,
        key=lambda item: (-item.dialogue_file_count, item.legacy_id),
    )


def parse_character_csv_identity(record: FileRecord) -> tuple[str | None, str | None]:
    source_path = getattr(record, "_source_path", None)
    if not source_path:
        return infer_name_from_chara_csv_path(record.path), None

    raw = Path(source_path).read_bytes()
    text, _encoding, _had_error = decode_sample(raw)
    name: str | None = None
    call_name: str | None = None
    for row in csv.reader(text.splitlines()):
        if len(row) < 2:
            continue
        key = row[0].strip()
        value = row[1].strip()
        if key == "名前":
            name = value
        elif key == "呼び名":
            call_name = value
        if name and call_name:
            break

    return name or infer_name_from_chara_csv_path(record.path), call_name


def infer_name_from_chara_csv_path(path: str) -> str | None:
    match = CHARA_CSV_RE.match(Path(path).name)
    if not match:
        return None
    return match.group("name")


def is_dialogue_candidate(path: str) -> bool:
    normalized = path.replace("\\", "/")
    markers = (
        "口上・メッセージ関連",
        "個人口上",
        "EVENT_MESSAGE",
        "ANOTHER_TALK",
        "EVENT",
        "TALK",
        "TRAIN",
    )
    return any(marker in normalized for marker in markers)


def classify_dialogue_kind(path: str) -> str:
    normalized = path.replace("\\", "/")
    if "個人口上" in normalized:
        return "personal_dialogue"
    if "EVENT_MESSAGE" in normalized or "EVENT" in normalized:
        return "event_message"
    if "ANOTHER_TALK" in normalized or "TALK" in normalized:
        return "talk_system"
    if "TRAIN" in normalized:
        return "command_or_training"
    return "dialogue_reference"


def infer_personal_dialogue_owner(path: Path) -> tuple[int, str] | None:
    parts = path.parts
    for index, part in enumerate(parts):
        if part == "個人口上" and index + 1 < len(parts):
            match = PERSONAL_DIALOGUE_DIR_RE.match(parts[index + 1])
            if match:
                return int(match.group("number")), match.group("name")
    return None


def inspect_text_file(path: Path, record: FileRecord, sample_bytes: int) -> None:
    raw = path.read_bytes()
    sample = raw[: max(0, sample_bytes)]
    text, encoding, had_decode_error = decode_sample(sample)
    record.encoding = encoding
    record.language_hint = detect_language_hint(text)
    record.line_count = count_lines(raw)
    record.resource_refs = sorted(set(match.group("name") for match in RESOURCE_REF_RE.finditer(text)))

    if record.category == "erb":
        record.erb_functions = sorted(set(match.group("name") for match in ERB_FUNCTION_RE.finditer(text)))
        record.erb_labels = sorted(set(match.group("name") for match in ERB_LABEL_RE.finditer(text)))

    if had_decode_error:
        record.issue_flags.append("decode_error")
    if looks_mojibake(text):
        record.issue_flags.append("possible_mojibake")
    if record.language_hint != "zh":
        record.issue_flags.append(f"language_{record.language_hint}")


def decode_sample(raw: bytes) -> tuple[str, str, bool]:
    encodings = ("utf-8-sig", "utf-8", "cp932", "gb18030", "shift_jis")
    for encoding in encodings:
        try:
            return raw.decode(encoding), encoding, False
        except UnicodeDecodeError:
            continue

    return raw.decode("utf-8", errors="replace"), "utf-8-replace", True


def detect_language_hint(text: str) -> str:
    if not text.strip():
        return "empty"

    zh = len(re.findall(r"[\u4e00-\u9fff]", text))
    kana = len(re.findall(r"[\u3040-\u30ff]", text))
    latin = len(re.findall(r"[A-Za-z]", text))
    total = max(1, zh + kana + latin)

    if kana / total > 0.08:
        return "ja"
    if zh / total > 0.18:
        return "zh"
    if latin / total > 0.55:
        return "latin"
    return "mixed"


def count_lines(raw: bytes) -> int:
    if not raw:
        return 0
    return raw.count(b"\n") + (0 if raw.endswith(b"\n") else 1)


def looks_mojibake(text: str) -> bool:
    return any(pattern in text for pattern in MOJIBAKE_PATTERNS)


def text_issues(record: FileRecord) -> list[AuditIssue]:
    issues: list[AuditIssue] = []
    for flag in record.issue_flags:
        if flag == "decode_error":
            issues.append(
                AuditIssue(
                    severity="warning",
                    code=flag,
                    path=record.path,
                    message="文本采样无法用常见编码无损解码。",
                )
            )
        elif flag == "possible_mojibake":
            issues.append(
                AuditIssue(
                    severity="warning",
                    code=flag,
                    path=record.path,
                    message="文本采样包含疑似乱码特征。",
                )
            )
        elif flag.startswith("language_") and flag != "language_zh":
            issues.append(
                AuditIssue(
                    severity="info",
                    code=flag,
                    path=record.path,
                    message="文本采样不是中文优先内容，后续需要翻译或重写评估。",
                )
            )
    return issues


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def resource_id_for(relative_path: str) -> str:
    stem = Path(relative_path).with_suffix("").as_posix()
    normalized = re.sub(r"[^0-9A-Za-z_.\-/\u3040-\u30ff\u3400-\u9fff]+", "_", stem)
    return f"legacy.{normalized.replace('/', '.')}"


def derive_asset_tags(path: Path) -> list[str]:
    tags: list[str] = []
    stem = path.stem
    if "顔" in stem:
        tags.append("face")
    if "立ち" in stem:
        tags.append("standing")
    if "バニー" in stem:
        tags.append("costume:bunny")
    if "巨" in stem:
        tags.append("variant:large")
    return tags


def normalize_relative(path: Path) -> str:
    return path.as_posix()
