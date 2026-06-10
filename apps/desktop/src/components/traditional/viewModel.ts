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
import {
  humanVillagePlacementForLegacyPlaceId,
  humanVillageSemanticMap,
  legacyPlaceIdForHotspot,
} from "./humanVillageMap";

export interface AsciiMapHotspot {
  key: string;
  action: TextMapAction;
  row: number;
  column: number;
  height: number;
  width: number;
  label: string;
  color: string | null;
  locationId: string | null;
}

export interface AsciiMapCell {
  key: string;
  character: string;
  row: number;
  column: number;
  width: number;
}

export interface AsciiMapLabel {
  key: string;
  locationId: string | null;
  marker: string;
  row: number;
  column: number;
  text: string;
}

export interface SemanticMapFeature {
  key: string;
  kind:
    | "boundary"
    | "building"
    | "gate"
    | "landmark"
    | "plaza"
    | "river"
    | "road"
    | "trees"
    | "water";
  label?: string;
  row: number;
  column: number;
  width: number;
  height: number;
}

export interface SemanticMapLayout {
  columns: number;
  imagePrompt: string;
  renderer: "svg-village";
  rows: number;
  features: SemanticMapFeature[];
}

export interface AsciiMapModel {
  cells: AsciiMapCell[];
  labels: AsciiMapLabel[];
  lines: string[];
  hotspots: AsciiMapHotspot[];
  semanticLayout: SemanticMapLayout | null;
  maxColumns: number;
  rowCount: number;
}

export interface LocationLegendGroup {
  id: string;
  title: string;
  locations: Location[];
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

interface LocationLegendGroupDefinition {
  areaIds?: ReadonlySet<string>;
  id: string;
  legacyPlaceIds?: ReadonlySet<number>;
  title: string;
}

const legendGroupDefinitions: LocationLegendGroupDefinition[] = [
  {
    id: "street",
    title: "街区 / 出入口",
    legacyPlaceIds: new Set([201, 202, 203, 204, 205, 206, 207]),
  },
  {
    id: "shop",
    title: "商店 / 设施",
    legacyPlaceIds: new Set([211, 212, 215, 216, 217, 218, 223, 225, 227, 228]),
  },
  {
    id: "public",
    title: "公共 / 文化",
    legacyPlaceIds: new Set([213, 219, 220, 221, 222, 224, 226, 229]),
  },
  {
    id: "row-house",
    title: "长屋 / 住居",
    areaIds: new Set(["sato-row-house"]),
  },
  {
    id: "geidontei",
    title: "鲵吞亭",
    areaIds: new Set(["sato-geidontei"]),
  },
] satisfies LocationLegendGroupDefinition[];

export const groupLocationLegendLocations = (
  locations: Location[],
): LocationLegendGroup[] => {
  const remaining = new Set(locations.map((location) => location.id));
  const groups = legendGroupDefinitions
    .map((definition) => {
      const groupedLocations = locations.filter((location) => {
        const byLegacyId =
          definition.legacyPlaceIds !== undefined &&
          location.legacy_place_id !== null &&
          location.legacy_place_id !== undefined &&
          definition.legacyPlaceIds.has(location.legacy_place_id);
        const byArea =
          definition.areaIds !== undefined &&
          location.map_area_id !== null &&
          location.map_area_id !== undefined &&
          definition.areaIds.has(location.map_area_id);
        if (byLegacyId || byArea) {
          remaining.delete(location.id);
          return true;
        }
        return false;
      });
      return { id: definition.id, title: definition.title, locations: groupedLocations };
    })
    .filter((group) => group.locations.length > 0);

  const otherLocations = locations.filter((location) => remaining.has(location.id));
  if (otherLocations.length > 0) {
    groups.push({ id: "other", title: "其他", locations: otherLocations });
  }

  return groups;
};

const isWideCodePoint = (codePoint: number) =>
  (codePoint >= 0x1100 && codePoint <= 0x115f) ||
  codePoint === 0x2329 ||
  codePoint === 0x232a ||
  (codePoint >= 0x2e80 && codePoint <= 0xa4cf) ||
  (codePoint >= 0xac00 && codePoint <= 0xd7a3) ||
  (codePoint >= 0xf900 && codePoint <= 0xfaff) ||
  (codePoint >= 0xfe10 && codePoint <= 0xfe19) ||
  (codePoint >= 0xfe30 && codePoint <= 0xfe6f) ||
  (codePoint >= 0xff00 && codePoint <= 0xff60) ||
  (codePoint >= 0xffe0 && codePoint <= 0xffe6) ||
  (codePoint >= 0x20000 && codePoint <= 0x3fffd);

export const terminalCharWidth = (character: string) => {
  const codePoint = character.codePointAt(0) ?? 0;
  if (
    codePoint === 0 ||
    codePoint < 0x20 ||
    (codePoint >= 0x7f && codePoint < 0xa0)
  ) {
    return 0;
  }
  return isWideCodePoint(codePoint) ? 2 : 1;
};

export const terminalWidth = (value: string) =>
  Array.from(value).reduce((total, character) => total + terminalCharWidth(character), 0);

const fitToTerminalWidth = (value: string, expectedWidth: number) => {
  let width = 0;
  let fitted = "";
  for (const character of Array.from(value)) {
    const characterWidth = terminalCharWidth(character);
    if (width + characterWidth > expectedWidth) {
      break;
    }
    fitted += character;
    width += characterWidth;
  }
  return `${fitted}${" ".repeat(Math.max(0, expectedWidth - width))}`;
};

export const normalizeMapRunText = (text: string) => {
  const expectedWidth = terminalWidth(text);
  const normalized = displayText(text);
  const normalizedWidth = terminalWidth(normalized);
  if (normalizedWidth === expectedWidth) {
    return normalized;
  }
  return fitToTerminalWidth(normalized, expectedWidth);
};

const cellsFromText = (text: string, row: number, startColumn: number, keyPrefix: string) => {
  const cells: AsciiMapCell[] = [];
  let column = startColumn;
  for (const [index, character] of Array.from(text).entries()) {
    const width = Math.max(1, terminalCharWidth(character));
    cells.push({
      key: `${keyPrefix}:${index}`,
      character,
      row,
      column,
      width,
    });
    column += width;
  }
  return cells;
};

const createHumanVillageSemanticLayout = (
  hotspots: AsciiMapHotspot[],
  labels: AsciiMapLabel[],
) => {
  const semanticHotspots = hotspots.map((hotspot) => {
    const position = humanVillagePlacementForLegacyPlaceId(
      legacyPlaceIdForHotspot(hotspot),
    );
    return position
      ? {
          ...hotspot,
          row: position.row,
          column: position.column,
          height: position.height,
          width: position.width,
        }
      : hotspot;
  });
  const semanticLabels = labels.map((label) => {
    const legacyPlaceId = Number(label.key.split(":").at(-1));
    const position = humanVillagePlacementForLegacyPlaceId(legacyPlaceId);
    return {
      ...label,
      row: position ? position.row + Math.max(1, Math.floor(position.height / 2)) : label.row,
      column: position ? position.column + 1 : label.column,
    };
  });

  return {
    hotspots: semanticHotspots,
    labels: semanticLabels,
    layout: humanVillageSemanticMap,
  };
};

export const buildAsciiMapModel = (area: TextMapArea | undefined): AsciiMapModel => {
  if (!area) {
    return {
      cells: cellsFromText("NO TEXT MAP DATA", 0, 0, "missing"),
      labels: [],
      lines: ["NO TEXT MAP DATA"],
      hotspots: [],
      semanticLayout: null,
      maxColumns: terminalWidth("NO TEXT MAP DATA"),
      rowCount: 1,
    };
  }

  const cells: AsciiMapCell[] = [];
  const hotspots: AsciiMapHotspot[] = [];
  const labels: AsciiMapLabel[] = [];
  const lines = area.rows.map((row, rowIndex) => {
    let column = 0;
    return row.runs
      .map((run, runIndex) => {
        const text = normalizeMapRunText(run.text);
        const width = terminalWidth(text);
        if (run.action) {
          const locationId =
            run.action.type === "move_to_location" ? run.action.location_id : null;
          const label = displayText(run.action.title ?? run.action.label);
          hotspots.push({
            key: `${area.id}:${rowIndex}:${runIndex}:${run.action.type}:${run.action.value}`,
            action: run.action,
            row: rowIndex,
            column,
            height: 1,
            width: Math.max(1, width),
            label,
            color: run.color,
            locationId,
          });
          labels.push({
            key: `${area.id}:label:${rowIndex}:${runIndex}:${run.action.value}`,
            locationId,
            marker: displayText(run.action.label),
            row: rowIndex,
            column,
            text: label,
          });
        }
        cells.push(
          ...cellsFromText(
            run.action ? " ".repeat(width) : text,
            rowIndex,
            column,
            `${area.id}:${rowIndex}:${runIndex}`,
          ),
        );
        column += width;
        return run.action ? " ".repeat(width) : text;
      })
      .join("");
  });

  const maxColumns = Math.max(1, ...lines.map(terminalWidth));
  const semantic =
    area.id === "sato-main" ? createHumanVillageSemanticLayout(hotspots, labels) : null;

  return {
    cells: semantic ? [] : cells,
    labels: semantic?.labels ?? labels,
    lines,
    hotspots: semantic?.hotspots ?? hotspots,
    semanticLayout: semantic?.layout ?? null,
    maxColumns: semantic?.layout.columns ?? maxColumns,
    rowCount: semantic?.layout.rows ?? lines.length,
  };
};
