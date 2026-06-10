import { displayText } from "../../engine/displayText";
import type {
  Character,
  Location,
  ResourceAsset,
  TextMap,
  TextMapAction,
  TextMapArea,
  WorldState,
} from "../../types";

export interface AsciiMapHotspot {
  key: string;
  action: TextMapAction;
  row: number;
  column: number;
  width: number;
  label: string;
  color: string | null;
  locationId: string | null;
}

export interface AsciiMapModel {
  gridRows: string[][];
  lines: string[];
  hotspots: AsciiMapHotspot[];
  maxColumns: number;
  rowCount: number;
}

export const seasonLabels = {
  spring: "春",
  summer: "夏",
  autumn: "秋",
  winter: "冬",
};

export const weatherLabels = {
  clear: "晴",
  cloudy: "阴",
  rain: "雨",
  snow: "雪",
};

export const terrainLabels: Record<string, string> = {
  street: "街道",
  interior: "室内",
  grass: "户外",
};

export const formatClock = (world: WorldState) =>
  `第${world.clock.day}日 ${String(world.clock.hour).padStart(2, "0")}:${String(
    world.clock.minute,
  ).padStart(2, "0")}`;

export const mapLocationIds = (textMap: TextMap | undefined) =>
  new Set(textMap?.locations.map((location) => location.location_id) ?? []);

export const charactersAtLocation = (
  world: WorldState,
  locationId: string | undefined,
) =>
  locationId
    ? world.characters.filter((character) => character.location_id === locationId)
    : [];

export const areaName = (
  textMap: TextMap | undefined,
  areaId: string | null | undefined,
) => displayText(textMap?.areas.find((area) => area.id === areaId)?.name, "未知区域");

export const locationName = (
  location: Location | undefined,
  fallback = "未知地点",
) => displayText(location?.name, fallback);

export const characterName = (
  character: Character | undefined,
  fallback = "未知人物",
) => displayText(character?.display_name, fallback);

export const terrainName = (terrain: string | undefined) =>
  terrain ? terrainLabels[terrain] ?? displayText(terrain) : "未知地形";

export const locationSymbol = (location: Location) => {
  const fallback =
    location.legacy_place_id === null || location.legacy_place_id === undefined
      ? "??"
      : String(location.legacy_place_id).slice(-2).padStart(2, "0");
  return displayText(location.ascii_symbol) || fallback;
};

export const findPortrait = (
  resources: ResourceAsset[],
  characterId: string | undefined,
): ResourceAsset | undefined =>
  characterId
    ? resources.find(
        (resource) =>
          resource.media_type === "image" &&
          resource.usage.includes("portrait") &&
          resource.character_bindings.includes(characterId),
      )
    : undefined;

export const canRenderImagePath = (sourcePath: string | undefined) =>
  !!sourcePath &&
  (sourcePath.startsWith("/") ||
    sourcePath.startsWith("http://") ||
    sourcePath.startsWith("https://") ||
    sourcePath.startsWith("data:image/"));

export const visibleLocationsForTextMap = (
  world: WorldState,
  textMap: TextMap | undefined,
) => {
  const locationIds = mapLocationIds(textMap);
  return world.locations.filter((location) => locationIds.has(location.id));
};

const charLength = (value: string) => Array.from(value).length;

export const normalizeMapRunText = (text: string) => {
  const expectedLength = charLength(text);
  const normalized = displayText(text);
  const normalizedChars = Array.from(normalized);
  if (normalizedChars.length === expectedLength) {
    return normalized;
  }
  if (normalizedChars.length > expectedLength) {
    return normalizedChars.slice(0, expectedLength).join("");
  }
  return `${normalized}${" ".repeat(expectedLength - normalizedChars.length)}`;
};

export const buildAsciiMapModel = (area: TextMapArea | undefined): AsciiMapModel => {
  if (!area) {
    return {
      gridRows: [["N", "O", " ", "T", "E", "X", "T", " ", "M", "A", "P", " ", "D", "A", "T", "A"]],
      lines: ["NO TEXT MAP DATA"],
      hotspots: [],
      maxColumns: 16,
      rowCount: 1,
    };
  }

  const hotspots: AsciiMapHotspot[] = [];
  const lines = area.rows.map((row, rowIndex) => {
    let column = 0;
    return row.runs
      .map((run, runIndex) => {
        const text = normalizeMapRunText(run.text);
        const width = charLength(text);
        if (run.action) {
          const locationId =
            run.action.type === "move_to_location" ? run.action.location_id : null;
          hotspots.push({
            key: `${area.id}:${rowIndex}:${runIndex}:${run.action.type}:${run.action.value}`,
            action: run.action,
            row: rowIndex,
            column,
            width: Math.max(1, width),
            label: displayText(run.action.title ?? run.action.label),
            color: run.color,
            locationId,
          });
        }
        column += width;
        return text;
      })
      .join("");
  });

  const maxColumns = Math.max(1, ...lines.map(charLength));

  return {
    gridRows: lines.map((line) => Array.from(line)),
    lines,
    hotspots,
    maxColumns,
    rowCount: lines.length,
  };
};
