import type { AsciiMapHotspot, SemanticMapFeature, SemanticMapLayout } from "./viewModel";

export interface HumanVillagePlacement {
  column: number;
  height: number;
  row: number;
  width: number;
}

const humanVillageFeatures: SemanticMapFeature[] = [
  { key: "north-forest", kind: "trees", row: 0, column: 4, width: 124, height: 12, variant: "dense" },
  { key: "west-forest", kind: "trees", row: 7, column: 0, width: 15, height: 70, variant: "dense" },
  { key: "east-forest", kind: "trees", row: 7, column: 117, width: 15, height: 70, variant: "dense" },
  { key: "south-forest", kind: "trees", row: 74, column: 0, width: 132, height: 10, variant: "riverbank" },
  { key: "south-river", kind: "river", row: 75, column: 2, width: 128, height: 7, variant: "main" },
  { key: "west-canal", kind: "canal", row: 15, column: 9, width: 4, height: 56, variant: "west" },
  { key: "east-canal", kind: "canal", row: 15, column: 119, width: 4, height: 56, variant: "east" },
  { key: "outer-wall", kind: "boundary", row: 5, column: 5, width: 122, height: 72, variant: "stone-wall" },
  { key: "north-gate", kind: "gate", row: 5, column: 60, width: 12, height: 8, label: "北大街", variant: "north" },
  { key: "south-gate", kind: "gate", row: 72, column: 60, width: 12, height: 7, label: "人里的门", variant: "south" },
  { key: "west-gate", kind: "gate", row: 43, column: 4, width: 10, height: 9, label: "西大街", variant: "west" },
  { key: "east-gate", kind: "gate", row: 43, column: 118, width: 10, height: 9, label: "东大街", variant: "east" },
  { key: "south-bridge", kind: "bridge", row: 76, column: 60, width: 12, height: 8, variant: "south" },

  { key: "main-street", kind: "road", row: 10, column: 62, width: 8, height: 65, variant: "main" },
  { key: "market-street", kind: "road", row: 45, column: 11, width: 110, height: 7, variant: "market" },
  { key: "north-street", kind: "road", row: 18, column: 17, width: 98, height: 5, variant: "lane" },
  { key: "merchant-street", kind: "road", row: 62, column: 16, width: 101, height: 5, variant: "lane" },
  { key: "west-lane", kind: "road", row: 20, column: 30, width: 5, height: 46, variant: "alley" },
  { key: "east-lane", kind: "road", row: 20, column: 96, width: 5, height: 47, variant: "alley" },
  { key: "plaza-approach-west", kind: "road", row: 37, column: 32, width: 28, height: 4, variant: "alley" },
  { key: "plaza-approach-east", kind: "road", row: 37, column: 72, width: 28, height: 4, variant: "alley" },

  { key: "hieda-yard", kind: "yard", row: 12, column: 16, width: 28, height: 23, label: "稗田院落", variant: "compound" },
  { key: "northwest-garden", kind: "water", row: 27, column: 24, width: 8, height: 5, label: "庭池", variant: "pond" },
  { key: "hieda", kind: "building", row: 15, column: 20, width: 20, height: 10, label: "稗田邸", variant: "manor" },
  { key: "akyuu", kind: "building", row: 15, column: 44, width: 12, height: 8, label: "阿求私室", variant: "residence" },
  { key: "ropeway-yard", kind: "yard", row: 12, column: 75, width: 18, height: 17, label: "索道站院", variant: "station" },
  { key: "ropeway", kind: "building", row: 14, column: 78, width: 13, height: 9, label: "索道站", variant: "station" },
  { key: "assembly-yard", kind: "yard", row: 12, column: 94, width: 22, height: 17, label: "集会所院", variant: "public" },
  { key: "assembly", kind: "building", row: 15, column: 98, width: 16, height: 9, label: "集会所", variant: "hall" },

  { key: "geidontei-yard", kind: "yard", row: 33, column: 14, width: 29, height: 15, label: "鲵吞亭院", variant: "tavern" },
  { key: "geidontei", kind: "building", row: 36, column: 17, width: 23, height: 9, label: "鲵吞亭", variant: "tavern" },
  { key: "watchtower", kind: "landmark", row: 32, column: 45, width: 9, height: 14, label: "瞭望楼", variant: "watchtower" },
  { key: "plaza", kind: "plaza", row: 37, column: 54, width: 24, height: 17, label: "广场", variant: "stone" },
  { key: "dragon-lantern", kind: "landmark", row: 33, column: 81, width: 7, height: 12, label: "龙灯", variant: "lantern" },
  { key: "keine", kind: "building", row: 28, column: 91, width: 11, height: 8, label: "慧音房间", variant: "residence" },
  { key: "terakoya-yard", kind: "yard", row: 25, column: 102, width: 19, height: 17, label: "寺子屋院", variant: "school" },
  { key: "terakoya", kind: "building", row: 28, column: 105, width: 14, height: 9, label: "寺子屋", variant: "school" },

  { key: "row-house-yard", kind: "yard", row: 52, column: 35, width: 18, height: 13, label: "长屋", variant: "row-house" },
  { key: "inn", kind: "building", row: 54, column: 16, width: 18, height: 10, label: "宿屋", variant: "inn" },
  { key: "row-front", kind: "building", row: 55, column: 37, width: 14, height: 8, label: "长屋前", variant: "row-house" },
  { key: "flower", kind: "building", row: 55, column: 54, width: 10, height: 8, label: "花屋", variant: "shop" },
  { key: "food", kind: "building", row: 55, column: 73, width: 14, height: 8, label: "食料品店", variant: "shop" },
  { key: "cafe", kind: "building", row: 55, column: 91, width: 14, height: 8, label: "咖啡馆", variant: "shop" },
  { key: "greengrocer", kind: "building", row: 55, column: 110, width: 11, height: 8, label: "八百屋", variant: "shop" },
  { key: "market-stalls", kind: "market", row: 64, column: 70, width: 35, height: 6, label: "市集", variant: "stalls" },
  { key: "southwest-field", kind: "field", row: 66, column: 18, width: 31, height: 8, label: "菜地", variant: "vegetable" },
  { key: "liquor", kind: "building", row: 68, column: 53, width: 11, height: 8, label: "酒屋", variant: "shop" },
  { key: "restaurant", kind: "building", row: 68, column: 73, width: 14, height: 8, label: "料理屋", variant: "restaurant" },
  { key: "sweets", kind: "building", row: 68, column: 91, width: 14, height: 8, label: "甘味处", variant: "shop" },
  { key: "suzunaan", kind: "building", row: 67, column: 110, width: 12, height: 9, label: "铃奈庵", variant: "bookstore" },
  { key: "bath", kind: "building", row: 49, column: 98, width: 12, height: 8, label: "钱汤", variant: "bath" },
  { key: "private-bath", kind: "water", row: 48, column: 111, width: 11, height: 9, label: "包场浴场", variant: "bath" },
];

const placementByLegacyPlaceId: Record<number, HumanVillagePlacement> = {
  201: { column: 60, row: 72, width: 12, height: 7 },
  202: { column: 54, row: 37, width: 24, height: 17 },
  203: { column: 62, row: 58, width: 8, height: 16 },
  204: { column: 118, row: 43, width: 10, height: 9 },
  205: { column: 60, row: 5, width: 12, height: 10 },
  206: { column: 4, row: 43, width: 10, height: 9 },
  207: { column: 78, row: 14, width: 13, height: 9 },
  211: { column: 53, row: 68, width: 11, height: 8 },
  212: { column: 91, row: 55, width: 14, height: 8 },
  213: { column: 110, row: 67, width: 12, height: 9 },
  214: { column: 37, row: 55, width: 14, height: 8 },
  215: { column: 54, row: 55, width: 10, height: 8 },
  216: { column: 73, row: 55, width: 14, height: 8 },
  217: { column: 91, row: 68, width: 14, height: 8 },
  218: { column: 73, row: 68, width: 14, height: 8 },
  219: { column: 98, row: 15, width: 16, height: 9 },
  220: { column: 45, row: 32, width: 9, height: 14 },
  221: { column: 16, row: 12, width: 28, height: 23 },
  222: { column: 105, row: 28, width: 14, height: 9 },
  223: { column: 98, row: 49, width: 12, height: 8 },
  224: { column: 91, row: 28, width: 11, height: 8 },
  225: { column: 16, row: 54, width: 18, height: 10 },
  227: { column: 110, row: 55, width: 11, height: 8 },
  228: { column: 111, row: 48, width: 11, height: 9 },
  229: { column: 44, row: 15, width: 12, height: 8 },
  238: { column: 14, row: 33, width: 29, height: 15 },
  239: { column: 25, row: 38, width: 10, height: 7 },
};

export const humanVillageSemanticMap: SemanticMapLayout = {
  columns: 132,
  imagePrompt:
    "Top-down readable game map of Touhou Project Human Village, dark terminal-inspired single-player RPG map, walled rural Japanese village at night with forest outside, narrow south river, south bridge and gate, north-south main street, east-west market street, central stone plaza, courtyard compounds and varied tiled-roof houses, Hieda residence and Akyuu room in the northwest, Geidontei tavern west of the plaza, watchtower near the plaza, Terakoya and Suzunaan in the east, merchant street with inn, flower shop, grocery, cafe, liquor store, restaurant, sweets shop, bathhouse and private bath, canals along both sides, lantern posts, fences, gardens, wells and market stalls, no text baked into the image, clean open spaces for overlay labels and clickable hotspots, suitable for future bitmap background toggle and SVG tracing",
  renderer: "svg-village",
  rows: 84,
  features: humanVillageFeatures,
};

export const legacyPlaceIdForHotspot = (hotspot: AsciiMapHotspot) =>
  hotspot.action.type === "move_to_location" ? Number(hotspot.action.value) : NaN;

export const humanVillagePlacementForLegacyPlaceId = (
  legacyPlaceId: number,
): HumanVillagePlacement | undefined =>
  Number.isFinite(legacyPlaceId) ? placementByLegacyPlaceId[legacyPlaceId] : undefined;
