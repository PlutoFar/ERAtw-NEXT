import type { AsciiMapHotspot, SemanticMapFeature, SemanticMapLayout } from "./viewModel";

export interface HumanVillagePlacement {
  column: number;
  height: number;
  row: number;
  width: number;
}

const humanVillageFeatures: SemanticMapFeature[] = [
  { key: "outer-wall", kind: "boundary", row: 1, column: 1, width: 94, height: 56 },
  { key: "north-woods", kind: "trees", row: 0, column: 4, width: 88, height: 8 },
  { key: "south-woods", kind: "trees", row: 49, column: 4, width: 88, height: 9 },
  { key: "west-woods", kind: "trees", row: 6, column: 0, width: 10, height: 44 },
  { key: "east-woods", kind: "trees", row: 6, column: 86, width: 10, height: 44 },
  { key: "north-road", kind: "road", row: 12, column: 9, width: 78, height: 5 },
  { key: "main-road", kind: "road", row: 7, column: 46, width: 7, height: 47 },
  { key: "market-road", kind: "road", row: 27, column: 9, width: 78, height: 6 },
  { key: "shop-road", kind: "road", row: 42, column: 20, width: 62, height: 5 },
  { key: "west-lane", kind: "road", row: 13, column: 10, width: 6, height: 32 },
  { key: "east-lane", kind: "road", row: 13, column: 80, width: 6, height: 32 },
  { key: "south-gate", kind: "gate", row: 52, column: 44, width: 10, height: 4, label: "人里的门" },
  { key: "north-street", kind: "gate", row: 7, column: 43, width: 12, height: 5, label: "北大街" },
  { key: "west-street", kind: "gate", row: 25, column: 5, width: 9, height: 9, label: "西大街" },
  { key: "east-street", kind: "gate", row: 25, column: 82, width: 9, height: 9, label: "东大街" },
  { key: "plaza", kind: "plaza", row: 22, column: 38, width: 20, height: 14, label: "广场" },
  { key: "hieda", kind: "building", row: 7, column: 15, width: 19, height: 11, label: "稗田邸" },
  { key: "akyuu", kind: "building", row: 7, column: 34, width: 10, height: 10, label: "阿求私室" },
  { key: "ropeway", kind: "building", row: 7, column: 56, width: 14, height: 10, label: "索道站" },
  { key: "assembly", kind: "building", row: 7, column: 71, width: 15, height: 10, label: "集会所" },
  { key: "geidontei", kind: "building", row: 19, column: 14, width: 18, height: 11, label: "鲵吞亭" },
  { key: "watchtower", kind: "landmark", row: 19, column: 34, width: 10, height: 10, label: "瞭望楼" },
  { key: "lantern", kind: "landmark", row: 19, column: 58, width: 8, height: 10, label: "龙灯" },
  { key: "keine", kind: "building", row: 19, column: 70, width: 10, height: 9, label: "慧音房间" },
  { key: "terakoya", kind: "building", row: 18, column: 80, width: 12, height: 10, label: "寺子屋" },
  { key: "inn", kind: "building", row: 35, column: 12, width: 14, height: 11, label: "宿屋" },
  { key: "row-front", kind: "building", row: 35, column: 26, width: 9, height: 11, label: "长屋前" },
  { key: "flower", kind: "building", row: 35, column: 36, width: 9, height: 9, label: "花屋" },
  { key: "food", kind: "building", row: 35, column: 55, width: 12, height: 9, label: "食料品店" },
  { key: "cafe", kind: "building", row: 35, column: 69, width: 12, height: 9, label: "咖啡馆" },
  { key: "greengrocer", kind: "building", row: 35, column: 82, width: 10, height: 9, label: "八百屋" },
  { key: "liquor", kind: "building", row: 46, column: 36, width: 9, height: 9, label: "酒屋" },
  { key: "restaurant", kind: "building", row: 46, column: 55, width: 12, height: 9, label: "料理屋" },
  { key: "sweets", kind: "building", row: 46, column: 69, width: 12, height: 9, label: "甘味处" },
  { key: "suzunaan", kind: "building", row: 46, column: 82, width: 10, height: 9, label: "铃奈庵" },
  { key: "bath", kind: "building", row: 31, column: 72, width: 10, height: 9, label: "钱汤" },
  { key: "private-bath", kind: "water", row: 31, column: 83, width: 9, height: 9, label: "包场浴场" },
];

const placementByLegacyPlaceId: Record<number, HumanVillagePlacement> = {
  201: { column: 44, row: 51, width: 10, height: 5 },
  202: { column: 38, row: 22, width: 20, height: 14 },
  203: { column: 46, row: 36, width: 7, height: 9 },
  204: { column: 82, row: 25, width: 9, height: 9 },
  205: { column: 43, row: 7, width: 12, height: 8 },
  206: { column: 5, row: 25, width: 10, height: 9 },
  207: { column: 56, row: 7, width: 14, height: 10 },
  211: { column: 36, row: 46, width: 9, height: 9 },
  212: { column: 69, row: 35, width: 12, height: 9 },
  213: { column: 82, row: 46, width: 10, height: 9 },
  214: { column: 26, row: 35, width: 9, height: 11 },
  215: { column: 36, row: 35, width: 9, height: 9 },
  216: { column: 55, row: 35, width: 12, height: 9 },
  217: { column: 69, row: 46, width: 12, height: 9 },
  218: { column: 55, row: 46, width: 12, height: 9 },
  219: { column: 71, row: 7, width: 15, height: 10 },
  220: { column: 34, row: 19, width: 10, height: 10 },
  221: { column: 15, row: 7, width: 19, height: 11 },
  222: { column: 80, row: 18, width: 12, height: 10 },
  223: { column: 72, row: 31, width: 10, height: 9 },
  224: { column: 70, row: 19, width: 10, height: 9 },
  225: { column: 12, row: 35, width: 14, height: 11 },
  227: { column: 82, row: 35, width: 10, height: 9 },
  228: { column: 83, row: 31, width: 9, height: 9 },
  229: { column: 34, row: 7, width: 10, height: 10 },
  238: { column: 14, row: 19, width: 18, height: 11 },
  239: { column: 16, row: 23, width: 14, height: 7 },
};

export const humanVillageSemanticMap: SemanticMapLayout = {
  columns: 96,
  imagePrompt:
    "Top-down readable game map of Touhou Project Human Village, black terminal-inspired UI mood, walled rural Japanese village at night, south village gate connected to a north-south main street, cross streets, central plaza, Hieda residence and Akyuu private room in the northwest, Suzunaan bookstore and Terakoya in the east, Geidontei tavern in the west, compact merchant street with inn, flower shop, grocery, cafe, liquor store, restaurant, sweets shop and bathhouse, long house district near the south, forest boundary around the village, clear roads and building footprints, no text baked into the image, designed for overlay labels and clickable hotspots",
  renderer: "css-village",
  rows: 58,
  features: humanVillageFeatures,
};

export const legacyPlaceIdForHotspot = (hotspot: AsciiMapHotspot) =>
  hotspot.action.type === "move_to_location" ? Number(hotspot.action.value) : NaN;

export const humanVillagePlacementForLegacyPlaceId = (
  legacyPlaceId: number,
): HumanVillagePlacement | undefined =>
  Number.isFinite(legacyPlaceId) ? placementByLegacyPlaceId[legacyPlaceId] : undefined;
