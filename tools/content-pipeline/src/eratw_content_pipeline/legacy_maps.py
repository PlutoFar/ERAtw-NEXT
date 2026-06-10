from __future__ import annotations

import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

ZERO_WIDTH = "\u200b"

FUNCTION_RE = re.compile(r"^@(?P<name>[A-Z0-9_]+)")
ROW_RE = re.compile(r"^\s*AA:(?P<row>\d+)\s*=\s*(?P<text>.*)$")
COLOR_RE = re.compile(r"^\s*MAPROW_COLORS:(?P<row>\d+)\s*=\s*(?P<colors>.*)$")
PLACE_RE = re.compile(r"P(?P<id>\d{3})(?P<name>[\w一-龯ぁ-んァ-ヴー々〆〤]+)")

NAMED_COLORS = {
    "florad": "#5a965a",
    "floran": "#1e321e",
    "floras": "#3c643c",
    "water": "#3d5b79",
    "tides": "#017979",
    "ground": "#513d29",
    "paving": "#797979",
}


@dataclass(frozen=True)
class LegacyMapExtraction:
    text_map: dict[str, Any]
    locations: list[dict[str, Any]]


def extract_legacy_maps(source: Path, map_id: int) -> LegacyMapExtraction:
    source = source.resolve()
    colored_path = _find_required(source, f"COLOREDMAP_{map_id:02}.ERB")
    movement_path = _find_optional(source, f"MAP_{map_id:02}.ERB")
    comm_path = _find_optional(source, f"MAP_COMM_{map_id:02}.ERB")

    place_names: dict[int, str] = {}
    if comm_path:
        place_names.update(_extract_place_names(_read_text(comm_path)))
    if movement_path:
        place_names.update(_extract_place_names(_read_text(movement_path)))

    blocks = _extract_function_blocks(_read_text(colored_path))
    map_key = _legacy_map_key(blocks, map_id)
    areas = []
    location_ids: dict[int, str] = {}

    for block_name, block_lines in blocks:
        area_id, area_name, kind = _area_identity(block_name, map_id, map_key)
        if area_id is None:
            continue

        rows = _extract_rows(block_lines)
        color_rows = _extract_color_rows(block_lines)
        rendered_rows = []
        for row_index in sorted(rows):
            rendered_row, row_locations = _row_to_runs(
                rows[row_index],
                color_rows.get(row_index, []),
                map_id,
                map_key,
                place_names,
            )
            rendered_rows.append({"runs": rendered_row})
            for legacy_place_id, location_id in row_locations.items():
                location_ids.setdefault(legacy_place_id, location_id)

        areas.append(
            {
                "id": area_id,
                "name": area_name,
                "kind": kind,
                "rows": rendered_rows,
            }
        )

    if not areas:
        raise ValueError(f"no supported colored map blocks found for map {map_id}")

    default_area_id = next(
        (area["id"] for area in areas if area["kind"] == "base"), areas[0]["id"]
    )
    locations = [
        {
            "id": location_id,
            "name": place_names.get(legacy_place_id, f"P{legacy_place_id}"),
            "ascii_symbol": "□",
            "terrain": "legacy",
            "legacy_place_id": legacy_place_id,
            "map_id": f"legacy.{map_key}",
            "map_area_id": _area_for_location(legacy_place_id, areas),
            "move_minutes": 5,
        }
        for legacy_place_id, location_id in sorted(location_ids.items())
    ]

    text_map = {
        "id": f"legacy.{map_key}",
        "name": _map_display_name(map_id, map_key),
        "default_area_id": default_area_id,
        "areas": areas,
        "locations": [
            {
                "location_id": location["id"],
                "legacy_place_id": location["legacy_place_id"],
                "area_id": location["map_area_id"],
            }
            for location in locations
        ],
    }
    return LegacyMapExtraction(text_map=text_map, locations=locations)


def write_legacy_map_outputs(extraction: LegacyMapExtraction, out: Path) -> list[Path]:
    out.mkdir(parents=True, exist_ok=True)
    written = [out / "text-map.json", out / "locations.json"]
    written[0].write_text(
        json.dumps(extraction.text_map, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    written[1].write_text(
        json.dumps(extraction.locations, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    return written


def _find_required(source: Path, file_name: str) -> Path:
    found = _find_optional(source, file_name)
    if not found:
        raise FileNotFoundError(f"missing legacy map file: {file_name}")
    return found


def _find_optional(source: Path, file_name: str) -> Path | None:
    matches = sorted(source.rglob(file_name))
    return matches[0] if matches else None


def _read_text(path: Path) -> str:
    data = path.read_bytes()
    for encoding in ("utf-8-sig", "utf-8", "cp932"):
        try:
            return data.decode(encoding)
        except UnicodeDecodeError:
            continue
    return data.decode("utf-8", errors="replace")


def _extract_function_blocks(text: str) -> list[tuple[str, list[str]]]:
    blocks: list[tuple[str, list[str]]] = []
    current_name: str | None = None
    current_lines: list[str] = []

    for line in text.splitlines():
        match = FUNCTION_RE.match(line)
        if match:
            if current_name is not None:
                blocks.append((current_name, current_lines))
            current_name = match.group("name")
            current_lines = []
        elif current_name is not None:
            current_lines.append(line)

    if current_name is not None:
        blocks.append((current_name, current_lines))
    return blocks


def _legacy_map_key(blocks: list[tuple[str, list[str]]], map_id: int) -> str:
    for name, _ in blocks:
        match = re.match(r"COLOREDMAP_(?P<key>[A-Z0-9]+)_\d+", name)
        if match:
            return match.group("key").lower()
    return f"map{map_id:02}"


def _area_identity(
    block_name: str, map_id: int, map_key: str
) -> tuple[str | None, str, str]:
    base_match = re.match(r"COLOREDMAP_[A-Z0-9]+_(?P<area>\d+)$", block_name)
    if base_match:
        area_number = int(base_match.group("area"))
        return f"{map_key}-{area_number}", f"{map_key.upper()} {area_number}", "base"

    outing_match = re.match(rf"COLOREDODEKAKEMAP_{map_id}$", block_name)
    if outing_match:
        return f"{map_key}-outing", f"{map_key.upper()} 外出", "outing"

    return None, "", "base"


def _extract_rows(lines: list[str]) -> dict[int, str]:
    rows: dict[int, str] = {}
    for line in lines:
        match = ROW_RE.match(line)
        if match:
            rows[int(match.group("row"))] = match.group("text").replace(ZERO_WIDTH, "")
    return rows


def _extract_color_rows(lines: list[str]) -> dict[int, list[str]]:
    rows: dict[int, list[str]] = {}
    for line in lines:
        match = COLOR_RE.match(line)
        if match:
            rows[int(match.group("row"))] = [
                token.strip() for token in match.group("colors").split(",")
            ]
    return rows


def _row_to_runs(
    text: str,
    color_tokens: list[str],
    map_id: int,
    map_key: str,
    place_names: dict[int, str],
) -> tuple[list[dict[str, Any]], dict[int, str]]:
    runs: list[dict[str, Any]] = []
    locations: dict[int, str] = {}
    chars = list(text)
    index = 0

    def append_text(value: str, token: str | None) -> None:
        if not value:
            return
        color = _color_from_token(token)
        if runs and runs[-1].get("action") is None and runs[-1].get("color_token") == token:
            runs[-1]["text"] += value
            return
        runs.append(
            {
                "text": value,
                "color": color,
                "color_token": token,
                "action": None,
            }
        )

    while index < len(chars):
        current = chars[index]
        next_char = chars[index + 1] if index + 1 < len(chars) else ""
        token = _token_at(color_tokens, index)
        if current.isascii() and current.isdigit() and next_char.isascii() and next_char.isdigit():
            label = f"{current}{next_char}"
            legacy_place_id = map_id * 100 + int(label)
            location_id = f"legacy.{map_key}.{legacy_place_id}"
            locations[legacy_place_id] = location_id
            runs.append(
                {
                    "text": label,
                    "color": _color_from_token(token) or "#7fd7ff",
                    "color_token": token or "legacy_button",
                    "action": {
                        "type": "move_to_location",
                        "label": label,
                        "value": str(legacy_place_id),
                        "location_id": location_id,
                        "title": place_names.get(legacy_place_id, f"P{legacy_place_id}"),
                    },
                }
            )
            index += 2
        else:
            append_text(current, token)
            index += 1

    return runs, locations


def _token_at(color_tokens: list[str], index: int) -> str | None:
    if index >= len(color_tokens):
        return None
    token = color_tokens[index].strip()
    return token or None


def _color_from_token(token: str | None) -> str | None:
    if token is None or token == "def":
        return None
    if token in NAMED_COLORS:
        return NAMED_COLORS[token]
    if token == "0":
        return "#000000"
    if re.fullmatch(r"0x[0-9a-fA-F]{6}", token):
        return f"#{token[2:].lower()}"
    return None


def _extract_place_names(text: str) -> dict[int, str]:
    names: dict[int, str] = {}
    for match in PLACE_RE.finditer(text):
        legacy_id = int(match.group("id"))
        names.setdefault(legacy_id, match.group("name"))
    return names


def _map_display_name(map_id: int, map_key: str) -> str:
    if map_id == 2 or map_key == "sato":
        return "人里"
    return map_key.upper()


def _area_for_location(legacy_place_id: int, areas: list[dict[str, Any]]) -> str | None:
    label = f"{legacy_place_id % 100:02}"
    for area in areas:
        for row in area["rows"]:
            for run in row["runs"]:
                action = run.get("action")
                if action and action.get("value") == str(legacy_place_id):
                    return area["id"]
                if action is None and run.get("text") == label:
                    return area["id"]
    return areas[0]["id"] if areas else None
