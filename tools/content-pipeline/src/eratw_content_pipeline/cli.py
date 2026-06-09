from __future__ import annotations

import argparse
import json
from pathlib import Path

from .legacy_audit import AuditOptions, audit_legacy_source, write_audit_outputs


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="eratw-content-pipeline",
        description="Offline ERAtw-NEXT content audit and packaging tools.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    audit = subparsers.add_parser(
        "audit-legacy",
        help="Scan a read-only legacy ERAtw source tree and emit M1 audit reports.",
    )
    audit.add_argument("--source", required=True, type=Path)
    audit.add_argument("--out", required=True, type=Path)
    audit.add_argument(
        "--sample-text-bytes",
        default=8192,
        type=int,
        help="Maximum bytes sampled from each ERB/CSV/text file for encoding and language checks.",
    )
    audit.add_argument(
        "--max-issues",
        default=200,
        type=int,
        help="Maximum issue rows retained in the summary report.",
    )

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if args.command == "audit-legacy":
        options = AuditOptions(
            source=args.source,
            out=args.out,
            sample_text_bytes=args.sample_text_bytes,
            max_issues=args.max_issues,
        )
        report = audit_legacy_source(options)
        written = write_audit_outputs(report, options.out)
        print(json.dumps({"ok": True, "written": [str(path) for path in written]}, ensure_ascii=False))
        return 0

    parser.error(f"unknown command: {args.command}")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
