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
CSV_SUFFIXES = {".csv"}
IMAGE_SUFFIXES = {".png", ".jpg", ".jpeg", ".webp", ".bmp", ".gif"}
AUDIO_SUFFIXES = {".ogg", ".mp3", ".wav", ".flac", ".m4a"}
FONT_SUFFIXES = {".ttf", ".otf", ".ttc", ".woff", ".woff2"}
TEXT_SUFFIXES = {".txt", ".md", ".json", ".cfg", ".config", ".dat", ".log"}

LEGACY_RUNTIME_SUFFIXES = {".exe", ".dll", ".sav"}

MOJIBAKE_PATTERNS = ("�", "ã", "Ã", "繧", "縺", "蜿", "譁", "荳")
RESOURCE_REF_RE = re.compile(
    r"(?P<name>[\w\-.一-龥ぁ-んァ-ヶー]+?\.(?:png|jpe?g|webp|bmp|gif|ogg|mp3|wav|ttf|otf))",
    re.IGNORECASE,
)
ERB_FUNCTION_RE = re.compile(r"^\s*@(?P<name>[A-Za-z0-9_\-\u3040-\u30ff\u3400-\u9fff]+)", re.MULTILINE)
ERB_LABEL_RE = re.compile(r"^\s*\$(?P<name>[A-Za-z0-9_\-\u3040-\u30ff\u3400-\u9fff]+)", re.MULTILINE)


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

        if category in {"erb", "csv", "text"}:
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
    summary_counter["total_files"] = len(records)
    summary_counter["total_size_bytes"] = sum(record.size_bytes for record in records)

    return LegacyAuditReport(
        schema_version="legacy-audit/v0",
        generated_at=datetime.now(timezone.utc).isoformat(),
        source_root=str(source),
        summary=dict(sorted(summary_counter.items())),
        files=records,
        assets=assets,
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
            "## Resource References",
            "",
            f"- Unique referenced resource names: {len(report.resource_reference_summary)}",
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
    if suffix in TEXT_SUFFIXES:
        return "text"
    return "other"


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
