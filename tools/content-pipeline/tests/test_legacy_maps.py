from __future__ import annotations

import json
from pathlib import Path

from eratw_content_pipeline.cli import main
from eratw_content_pipeline.legacy_maps import extract_legacy_maps


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def test_extract_legacy_maps_parses_colored_rows_and_actions(tmp_path: Path) -> None:
    source = tmp_path / "legacy"
    write_text(
        source / "ERB" / "COLOREDMAPS" / "COLOREDMAP_02.ERB",
        "\n".join(
            [
                "@COLOREDMAP_SATO_1(AA,MAPROW_COLORS)",
                "\tAA:00 = □\u200b0\u200b1\u200b森",
                "\tMAPROW_COLORS:00 = 0x797979,florad,0x292929,def",
                "@COLOREDODEKAKEMAP_2(AA,MAPROW_COLORS)",
                "\tAA:00 = ┼02┼",
                "\tMAPROW_COLORS:00 = def,0x017979,0x013D3D,def",
            ]
        ),
    )
    write_text(
        source / "ERB" / "MOVEMENTS" / "物件関連" / "02人間の里" / "MAP_COMM_02.ERB",
        "\n".join(
            [
                "SELECTCASE ARG",
                "\tCASE P201人里的門",
                "\tCASE P202广场",
                "ENDSELECT",
            ]
        ),
    )

    extraction = extract_legacy_maps(source, 2)

    assert extraction.text_map["id"] == "legacy.sato"
    assert extraction.text_map["default_area_id"] == "sato-1"
    assert [area["kind"] for area in extraction.text_map["areas"]] == [
        "base",
        "outing",
    ]
    first_row_runs = extraction.text_map["areas"][0]["rows"][0]["runs"]
    assert "".join(run["text"] for run in first_row_runs) == "□01森"
    action_run = next(run for run in first_row_runs if run["action"])
    assert action_run["text"] == "01"
    assert action_run["color"] == "#5a965a"
    assert action_run["action"]["location_id"] == "legacy.sato.201"
    assert action_run["action"]["title"] == "人里的門"
    assert extraction.locations[0]["id"] == "legacy.sato.201"
    assert extraction.locations[0]["name"] == "人里的門"


def test_extract_legacy_maps_cli_writes_json(tmp_path: Path) -> None:
    source = tmp_path / "legacy"
    out = tmp_path / "out"
    write_text(
        source / "ERB" / "COLOREDMAPS" / "COLOREDMAP_02.ERB",
        "\n".join(
            [
                "@COLOREDMAP_SATO_1(AA,MAPROW_COLORS)",
                "\tAA:00 = 01",
                "\tMAPROW_COLORS:00 = 0x797979,0x515151",
            ]
        ),
    )

    exit_code = main(
        ["extract-legacy-maps", "--source", str(source), "--map-id", "2", "--out", str(out)]
    )

    assert exit_code == 0
    text_map = json.loads((out / "text-map.json").read_text(encoding="utf-8"))
    locations = json.loads((out / "locations.json").read_text(encoding="utf-8"))
    assert text_map["id"] == "legacy.sato"
    assert locations[0]["legacy_place_id"] == 201
