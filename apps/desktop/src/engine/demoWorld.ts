import type {
  DialogueChoice,
  DialogueCondition,
  DialogueEffect,
  DialogueNode,
  DialogueScene,
  EngineReplayLog,
  EngineCommand,
  Location,
  ScheduledEvent,
  ScheduledEventKind,
  ScheduledTime,
  TextMap,
  TextMapRow,
  Weather,
  WorldState,
} from "../types";

const weatherLabels: Record<Weather, string> = {
  clear: "晴",
  cloudy: "阴",
  rain: "雨",
  snow: "雪",
};

const DEMO_RNG_SEED = "1163026804";
const U64_MASK = 0xffff_ffff_ffff_ffffn;
const U64_SPACE = 1n << 64n;

const splitmix64 = (value: bigint) => {
  let next = (value + 0x9e37_79b9_7f4a_7c15n) & U64_MASK;
  next = ((next ^ (next >> 30n)) * 0xbf58_476d_1ce4_e5b9n) & U64_MASK;
  next = ((next ^ (next >> 27n)) * 0x94d0_49bb_1331_11ebn) & U64_MASK;
  return (next ^ (next >> 31n)) & U64_MASK;
};

const nextRandomU64 = (world: WorldState) => {
  const cursor = BigInt(world.random.cursor);
  const value = splitmix64(BigInt(world.random.seed) + cursor);
  world.random.cursor = (cursor + 1n).toString();
  return value;
};

const nextBoundedRandom = (world: WorldState, upperExclusive: bigint) => {
  const zone = (U64_SPACE / upperExclusive) * upperExclusive;

  while (true) {
    const value = nextRandomU64(world);
    if (value < zone) {
      return value % upperExclusive;
    }
  }
};

const rollInclusive = (world: WorldState, min: number, max: number) => {
  const span = BigInt(max - min + 1);
  const offset = Number(nextBoundedRandom(world, span));
  return min + offset;
};

const absoluteMinute = (time: ScheduledTime) =>
  Math.max(0, time.day - 1) * 1440 + time.hour * 60 + time.minute;

const scheduledTimeFromAbsoluteMinute = (value: number): ScheduledTime => {
  const minuteOfDay = value % 1440;
  return {
    day: Math.floor(value / 1440) + 1,
    hour: Math.floor(minuteOfDay / 60),
    minute: minuteOfDay % 60,
  };
};

const currentAbsoluteMinute = (world: WorldState) =>
  absoluteMinute({
    day: world.clock.day,
    hour: world.clock.hour,
    minute: world.clock.minute,
  });

const byDueTime = (left: ScheduledEvent, right: ScheduledEvent) => {
  const delta = absoluteMinute(left.due) - absoluteMinute(right.due);
  if (delta !== 0) {
    return delta;
  }

  const priorityDelta = (right.priority ?? 0) - (left.priority ?? 0);
  return priorityDelta === 0 ? left.id.localeCompare(right.id) : priorityDelta;
};

const isValidDue = (due: ScheduledTime) =>
  due.day > 0 && due.hour >= 0 && due.hour < 24 && due.minute >= 0 && due.minute < 60;

const isValidRepeat = (event: ScheduledEvent) =>
  event.repeat == null ||
  (event.repeat.every_minutes > 0 && event.repeat.remaining_runs !== 0);

const clamp = (value: number, min: number, max: number) =>
  Math.max(min, Math.min(max, value));

const createDemoDialogueScenes = (): DialogueScene[] => [
  {
    id: "demo_morning",
    entry_node_id: "demo_morning_001",
    nodes: [
      {
        id: "demo_morning_001",
        speaker_id: "demo_heroine",
        text: "早上好。今天先从一个干净的新世界开始。",
        resource_refs: ["core.demo.heroine.neutral"],
        choices: [
          {
            id: "ask_about_engine",
            label: "询问新引擎",
            next_node_id: "demo_morning_002",
            conditions: [],
            effects: [
              {
                type: "add_log",
                message: "对话选择：询问新引擎。",
              },
            ],
          },
          {
            id: "encourage",
            label: "鼓励她",
            next_node_id: "demo_morning_003",
            conditions: [],
            effects: [
              {
                type: "adjust_character_state",
                character_id: "demo_heroine",
                energy_delta: 0,
                mood_delta: 3,
              },
              {
                type: "adjust_relationship",
                source_character_id: "player",
                target_character_id: "demo_heroine",
                affinity_delta: 2,
                trust_delta: 1,
              },
            ],
          },
          {
            id: "talk_about_trust",
            label: "谈谈信任",
            next_node_id: "demo_morning_004",
            conditions: [
              {
                type: "relationship_affinity_at_least",
                source_character_id: "player",
                target_character_id: "demo_heroine",
                value: 7,
              },
            ],
            effects: [
              {
                type: "adjust_relationship",
                source_character_id: "player",
                target_character_id: "demo_heroine",
                affinity_delta: 0,
                trust_delta: 2,
              },
            ],
          },
        ],
      },
      {
        id: "demo_morning_002",
        speaker_id: "system",
        text: "该对话来自版本化 DialogueScene，不执行旧 ERB。",
        resource_refs: [],
        choices: [],
      },
      {
        id: "demo_morning_003",
        speaker_id: "demo_heroine",
        text: "嗯。先把能稳定重放的小循环做好。",
        resource_refs: [],
        choices: [],
      },
      {
        id: "demo_morning_004",
        speaker_id: "demo_heroine",
        text: "信任会一点点积累。先从可验证的承诺开始。",
        resource_refs: [],
        choices: [],
      },
    ],
  },
];

type DemoSatoLocationSpec = [
  legacyPlaceId: number,
  id: string,
  name: string,
  asciiSymbol: string,
  terrain: string,
  areaId: string,
];

const DEMO_SATO_LOCATION_SPECS: DemoSatoLocationSpec[] = [
  [201, "school_gate", "人里的門", "門", "street", "sato-main"],
  [202, "garden", "广场", "◇", "street", "sato-main"],
  [203, "club_room", "南大街", "南", "street", "sato-main"],
  [204, "legacy.sato.204", "東大街", "東", "street", "sato-main"],
  [205, "legacy.sato.205", "北大街", "北", "street", "sato-main"],
  [206, "legacy.sato.206", "西大街", "西", "street", "sato-main"],
  [207, "legacy.sato.207", "索道站", "龍", "street", "sato-main"],
  [208, "legacy.sato.208", "雷鼓的房間", "雷", "interior", "sato-row-house"],
  [209, "legacy.sato.209", "八橋的房間", "八", "interior", "sato-row-house"],
  [210, "legacy.sato.210", "弁弁的房間", "弁", "interior", "sato-row-house"],
  [211, "legacy.sato.211", "酒屋", "酒", "interior", "sato-main"],
  [212, "legacy.sato.212", "咖啡館", "咖", "interior", "sato-main"],
  [213, "legacy.sato.213", "鈴奈庵", "鈴", "interior", "sato-suzunaan"],
  [214, "legacy.sato.214", "長屋前", "長", "street", "sato-row-house"],
  [215, "legacy.sato.215", "花屋", "花", "interior", "sato-main"],
  [216, "legacy.sato.216", "食料品店", "食", "interior", "sato-main"],
  [217, "legacy.sato.217", "甘味処", "甘", "interior", "sato-main"],
  [218, "legacy.sato.218", "料理屋", "料", "interior", "sato-main"],
  [219, "legacy.sato.219", "集会所", "集", "interior", "sato-main"],
  [220, "legacy.sato.220", "瞭望樓", "瞭", "interior", "sato-main"],
  [221, "legacy.sato.221", "稗田邸", "稗", "interior", "sato-main"],
  [222, "legacy.sato.222", "寺子屋", "寺", "interior", "sato-main"],
  [223, "legacy.sato.223", "銭湯", "湯", "interior", "sato-main"],
  [224, "legacy.sato.224", "慧音的房間", "慧", "interior", "sato-main"],
  [225, "legacy.sato.225", "宿屋", "宿", "interior", "sato-main"],
  [226, "legacy.sato.226", "小鈴私室", "鈴", "interior", "sato-suzunaan"],
  [227, "legacy.sato.227", "八百屋", "八", "interior", "sato-main"],
  [228, "legacy.sato.228", "貸切浴場", "♨", "interior", "sato-main"],
  [229, "legacy.sato.229", "阿求私室", "阿", "interior", "sato-main"],
  [230, "legacy.sato.230", "空的部屋", "空", "interior", "sato-row-house"],
  [231, "legacy.sato.231", "蛮奇的房間", "蛮", "interior", "sato-row-house"],
  [232, "legacy.sato.232", "蓮子的房間", "蓮", "interior", "sato-row-house"],
  [233, "legacy.sato.233", "梅莉的房間", "梅", "interior", "sato-row-house"],
  [234, "legacy.sato.234", "雪的房間", "雪", "interior", "sato-row-house"],
  [235, "legacy.sato.235", "舞的房間", "舞", "interior", "sato-row-house"],
  [236, "legacy.sato.236", "厠", "厠", "interior", "sato-row-house"],
  [237, "legacy.sato.237", "公用水井", "井", "interior", "sato-row-house"],
  [238, "legacy.sato.238", "鯢呑亭", "鯢", "interior", "sato-geidontei"],
  [239, "legacy.sato.239", "美宵的房間", "美", "interior", "sato-geidontei"],
  [241, "legacy.sato.241", "空的部屋", "空", "interior", "sato-row-house"],
  [242, "legacy.sato.242", "麟的房間", "麟", "interior", "sato-row-house"],
  [243, "legacy.sato.243", "空的部屋", "空", "interior", "sato-row-house"],
];

const createDemoSatoLocations = (): Location[] =>
  DEMO_SATO_LOCATION_SPECS.map(
    ([legacyPlaceId, id, name, asciiSymbol, terrain, areaId]) => ({
      id,
      name,
      ascii_symbol: asciiSymbol,
      terrain,
      legacy_place_id: legacyPlaceId,
      map_id: "legacy.sato",
      map_area_id: areaId,
      move_minutes: 5,
    }),
  );

const demoSatoLocationId = (legacyPlaceId: number) =>
  DEMO_SATO_LOCATION_SPECS.find(([candidate]) => candidate === legacyPlaceId)?.[1] ??
  "school_gate";

const demoSatoLocationName = (legacyPlaceId: number) =>
  DEMO_SATO_LOCATION_SPECS.find(([candidate]) => candidate === legacyPlaceId)?.[2] ??
  "人里";

const createDemoTextRow = (row: string): TextMapRow => {
  const runs: TextMapRow["runs"] = [];
  let buffer = "";
  const chars = [...row];

  for (let index = 0; index < chars.length; index += 1) {
    const current = chars[index];
    const next = chars[index + 1];
    if (/\d/.test(current) && next && /\d/.test(next)) {
      if (buffer) {
        runs.push({ text: buffer, color: null, color_token: null, action: null });
        buffer = "";
      }

      const label = `${current}${next}`;
      const legacyPlaceId = 200 + Number(label);
      runs.push({
        text: label,
        color: "#7fd7ff",
        color_token: "legacy_button",
        action: {
          type: "move_to_location",
          label,
          value: String(legacyPlaceId),
          location_id: demoSatoLocationId(legacyPlaceId),
          title: demoSatoLocationName(legacyPlaceId),
        },
      });
      index += 1;
    } else {
      buffer += current;
    }
  }

  if (buffer) {
    runs.push({ text: buffer, color: null, color_token: null, action: null });
  }

  return { runs };
};

const createDemoTextRows = (rows: string[]) => rows.map(createDemoTextRow);

const createDemoSatoTextMap = (locations: Location[]): TextMap => ({
  id: "legacy.sato",
  name: "人里",
  default_area_id: "sato-main",
  locations: locations.map((location) => ({
    location_id: location.id,
    legacy_place_id: location.legacy_place_id ?? null,
    area_id: location.map_area_id ?? null,
  })),
  areas: [
    {
      id: "sato-main",
      name: "人里",
      kind: "base",
      rows: createDemoTextRows([
        "　||■■■■■■■||　　　■|＝＝|　　□■　　　■■■■■■■■■■■■■■■■■■■■",
        "　||■　| 稗田邸■||　　　■|＝＝|索道□■　　　■　　　■　　■　　　■ 慧音 ■└─┘■",
        "　||■29| 　21　■||　　　■■■■──■■　　　■■■■■■■■■■■■　24　│寺子屋■",
        "　||■■■─■■■||　　　　　　　　　　　　北　　　　　　　　　　　　■■─■■　22　■",
        "　□＝＝●　●＝＝□　　　　　　　　　　　　05　　　　　　　　　　　　　　　　│　　□■",
        "　　　　　　　　　　　　■■■■■■■■　　　　□＋□■■■──■■■　　　　■□　□■",
        "　　　　　　　　　　　　■　　■　　　■　　　07＋龍＋■　 集会所 　■　　　　■□　□■",
        "■■■■■■■■■■　　■■■■■■■■　　　　□＋□■　　 19 　　■　　　　■■─■■",
        "■　　■ ┃鯢呑 □■　　■＼／||櫓■　　　　　　　　　■■─■■─■■　　　　　　　　　",
        "■　　■ ┃ 38　□■　　■／＼■20■　　　　　　　　　■□　　　　　│　　　　　　　　　",
        "■■■■■■─■■■　　■■■■─■　　　　　　　　　■■■■■■■■　　■■■■■■●",
        "　　　　　　　　　　 西 　　　　　　　　　 広场 　　　　　　　　　　　 東 ■銭湯■ 28 ■",
        "　　　　　　　　　　 06 　　　　　　　　　　◇　　　　　　　　　　　　 04 │ 23 │ ♨ ■",
        "　　　　　　　　　　　　　　　　　　　　　　02　　　　　　　　　　　　　　■　　■━━■",
        "■■■──■　□□□　　■■─■■■　　　　　　　　　■──■■──■　　■　　│　　■",
        "■□□ 宿 ■　■■■　　■＠花屋　│　　　　　　　　　■○ 八百屋 ○■　　■　　■ ♨ ■",
        "■　　 25 ■　■　│　　■＠ 15 　■　　　　　　　　　■□□ 27 □□■　　■■■■■■■",
        "■■■　　■　■■■長屋■■■■■■─■■　　　■■■■■■■■■■■　　　　　　　　　",
        "■　│　　■　■　│ 14 ■□□ 食料品 　│　　　■　 咖啡館 □■　　■　　　　　　　　　",
        "■■■　　■　■■■　　■□□　 16 　　■　南　│　　 12 　□■甘味■　　■■■■■■■",
        "■　│　　■　■　│　　■■■■■■■■■　03　■■■■■■■■ 17 │　　■鈴奈庵　┃■",
        "■■■　　■　■■■　　■○○　酒屋　　■　　　■　 料理屋 □■　　■　　│　13　□┃■",
        "■　│　　■　■　│　　■○○　 11 　　│　　　│　　 18 　□■□□■　　■□□□　┃■",
        "■■■　　■　■■■　　■■■■■■■■■　　　■■■■■■■■■■■　　■■■■■■■",
        "■　│　　■　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　　",
        "■■■■■■■■■■■■■■■■■■■■●　01　●■■■■■■■■■■■■■■■■■■■",
      ]),
    },
    {
      id: "sato-row-house",
      name: "長屋",
      kind: "base",
      rows: createDemoTextRows([
        "■＝■■■■＝■■■＝■■■＝■■■＝■■■＝■■　　　　　　　　　",
        "■　　　┃ 雷鼓 ┃ 八橋 ┃ 弁弁 ┃　雪　┃　舞　■　　　　　　　　　",
        "■　43　┃　08　┃　09　┃　10　┃　34　┃　35　■　　　　　　　　　",
        "■■■■■■■■■■■■■■■■■■■■■■■■■　　厠　　 井戸 　",
        "■　麟　┃　　  ┃　　　┃ 蛮奇 ┃ 蓮子 ┃ 梅莉 ■　■■■　┏━┓　",
        "■　42　┃　41　┃　30　┃　31　┃　32　┃　33　■　■36■　┃□┃　",
        "■■＝■■■＝■■■＝■■■＝■■■＝■■■＝■■　　　　　　37　　",
        "　　　　　　　　　　　　14　　　　　　　　　　　　　　　　　",
        "　←南大街　　　　　　　　　　　　　　　　　　　　西大街→　",
        "　　　03　　　　　　　　01　　　　　　　　02　　　　06　　　",
        "　　　　　　　　　　人里的門↓　　　　　 広场→↓ 　　　　　",
      ]),
    },
    {
      id: "sato-suzunaan",
      name: "鈴奈庵",
      kind: "base",
      rows: createDemoTextRows([
        "　　　■■■■■■■■■■■■■■■■■■■■■■■　■■■■■■",
        "　　　■┌───────────┘　□□■　　　　■　■呂#Aﾛ日ﾛ■",
        "　　　■　┌───┐　││　││　　　　■　　　　■／■#A呂日#A■",
        "　東　│　└───┘　└┘　└┘　　□　■　　　　■／　■■■■■■",
        "　04　│　┌───┐　┌┐　┌┐　13□　■■■─■／",
        "　　　■└───────────┐　□□■　　　　│",
        "　　　■■■■■■■■■■■■■■■■■■■■■■■",
      ]),
    },
    {
      id: "sato-geidontei",
      name: "鯢呑亭",
      kind: "base",
      rows: createDemoTextRows([
        "　■■■■■■　■■■■■■■■■■■■■■■■■■■■　",
        "　■□□　　■　■└────┘　│○　　　||#B#B　#B#B■",
        "　■　 39 　＼　■　　　○　　　│○　　　||#B#B　#B#B■",
        "　■#f#j#g　■＼■─■　　　　　│○ 38 　||　　　　　■",
        "　■■□□■■　■＼■○○○○○　　　　　||#B#B　#B#B■",
        "　　　　　　　　■■■■■■■■■■＝＝■■■■■■■■　",
        "　　　　　　　　　　　　　　　┃鯢┃　　　　　　　　　　　",
        "　　　　　　　　　　　　　　　　　　 西 　　　　広场→　",
        "　　　　　　　　　　　　　　　　　　 06 　　　　　02　　",
      ]),
    },
    {
      id: "sato-outing",
      name: "人里外出",
      kind: "outing",
      rows: createDemoTextRows([
        "□＝＝＝＝＝＝三＝＝＝三三＝＝＝三＝＝＝＝＝□",
        "||┏日━／＼━ ,川川川川,  ,川川川川, ━━━━━┓||",
        "||┃・| ・п・ |／＼三三= 　..川川川　 | 04 ｜林┃||",
        "□：┼┼┼┼┼┼┼┼┼┼-01-┼┼┼┼-02=三三= 林：□",
        "||┃|_二二06二二 =三三三=　=三三三三=  =三三= 林┃||",
        "||┃林二二┃二二 =三三三 03=三三三三=  =三三= 林┃||",
        "||▲▲|__|┃　 07 ￣￣￣╦══╦￣￣￣　　￣　林┃||",
        "□＝＝＝＝＝＝三＝＝＝三三＝＝＝三＝＝＝＝＝□",
      ]),
    },
  ],
});

export const createDemoWorld = (): WorldState => {
  const locations = createDemoSatoLocations();

  return {
  engine_version: "0.1.0-m0",
  installed_content_packages: [],
  clock: {
    day: 1,
    hour: 8,
    minute: 0,
    season: "spring",
    weather: "clear",
  },
  locations,
  text_maps: [createDemoSatoTextMap(locations)],
  characters: [
    {
      id: "demo_heroine",
      display_name: "示例角色",
      location_id: "school_gate",
      state: {
        energy: 80,
        mood: 10,
      },
    },
  ],
  resources: [
    {
      resource_id: "core.demo.heroine.neutral",
      source_path: "assets/demo/heroine-neutral.webp",
      media_type: "image",
      license: "project-demo",
      author: "ERAtw-NEXT",
      usage: ["portrait"],
      character_bindings: ["demo_heroine"],
      tags: ["neutral"],
      sha256: null,
    },
  ],
  relationships: [
    {
      source_character_id: "player",
      target_character_id: "demo_heroine",
      affinity: 5,
      trust: 0,
    },
  ],
  dialogue_scenes: createDemoDialogueScenes(),
  active_dialogue_scene_id: null,
  active_dialogue: [],
  scheduled_events: [
    {
      id: "demo_clouds_at_gate",
      due: { day: 1, hour: 8, minute: 30 },
      priority: 0,
      repeat: null,
      conditions: [],
      kind: { type: "change_weather", weather: "cloudy" },
    },
    {
      id: "demo_morning_mood",
      due: { day: 1, hour: 9, minute: 0 },
      priority: 0,
      repeat: null,
      conditions: [],
      kind: {
        type: "adjust_character_state",
        character_id: "demo_heroine",
        energy_delta: -3,
        mood_delta: 5,
      },
    },
  ],
  random: {
    seed: DEMO_RNG_SEED,
    cursor: "0",
  },
  command_log_initial_random: null,
  command_log: [],
  event_log: ["ERAtw-NEXT M0 engine ready."],
  };
};

const recordCommand = (
  world: WorldState,
  command: EngineCommand,
  initialRandom: WorldState["random"],
): WorldState => {
  return {
    ...world,
    command_log_initial_random:
      world.command_log.length === 0
        ? structuredClone(initialRandom)
        : world.command_log_initial_random,
    command_log: [...world.command_log, structuredClone(command)],
  };
};

const startDialogue = (world: WorldState, sceneId: string): boolean => {
  const scene = world.dialogue_scenes.find((item) => item.id === sceneId);
  const entry = scene?.nodes.find((node) => node.id === scene.entry_node_id);
  if (!scene || !entry) {
    return false;
  }

  world.active_dialogue_scene_id = scene.id;
  world.active_dialogue = [structuredClone(entry)];
  world.event_log = [`播放场景 ${sceneId}。`, ...world.event_log];
  return true;
};

const findDialogueNode = (
  world: WorldState,
  sceneId: string,
  nodeId: string,
): DialogueNode | undefined =>
  world.dialogue_scenes
    .find((scene) => scene.id === sceneId)
    ?.nodes.find((node) => node.id === nodeId);

const applyCharacterStateDelta = (
  world: WorldState,
  characterId: string,
  energyDelta: number,
  moodDelta: number,
) => {
  const character = world.characters.find((item) => item.id === characterId);
  if (!character) {
    return false;
  }

  character.state.energy = clamp(character.state.energy + energyDelta, 0, 100);
  character.state.mood = clamp(character.state.mood + moodDelta, -100, 100);
  return true;
};

const applyRelationshipDelta = (
  world: WorldState,
  sourceCharacterId: string,
  targetCharacterId: string,
  affinityDelta: number,
  trustDelta: number,
) => {
  const target = world.characters.find((item) => item.id === targetCharacterId);
  const relationship = world.relationships.find(
    (item) =>
      item.source_character_id === sourceCharacterId &&
      item.target_character_id === targetCharacterId,
  );
  if (!target || !relationship) {
    return false;
  }

  relationship.affinity = clamp(relationship.affinity + affinityDelta, -100, 100);
  relationship.trust = clamp(relationship.trust + trustDelta, -100, 100);
  return true;
};

const applyDialogueEffect = (world: WorldState, effect: DialogueEffect) => {
  if (effect.type === "adjust_character_state") {
    return applyCharacterStateDelta(
      world,
      effect.character_id,
      effect.energy_delta,
      effect.mood_delta,
    );
  }

  if (effect.type === "adjust_relationship") {
    return applyRelationshipDelta(
      world,
      effect.source_character_id,
      effect.target_character_id,
      effect.affinity_delta,
      effect.trust_delta,
    );
  }

  if (effect.type === "roll_character_state") {
    if (
      effect.energy_min_delta > effect.energy_max_delta ||
      effect.mood_min_delta > effect.mood_max_delta
    ) {
      return false;
    }

    const energyDelta = rollInclusive(
      world,
      effect.energy_min_delta,
      effect.energy_max_delta,
    );
    const moodDelta = rollInclusive(
      world,
      effect.mood_min_delta,
      effect.mood_max_delta,
    );
    const updated = applyCharacterStateDelta(
      world,
      effect.character_id,
      energyDelta,
      moodDelta,
    );
    if (!updated) {
      return false;
    }

    const character = world.characters.find(
      (item) => item.id === effect.character_id,
    );
    world.event_log = [
      `${character?.display_name ?? effect.character_id} 状态随机变化：体力 ${
        energyDelta >= 0 ? "+" : ""
      }${energyDelta}（范围 ${
        effect.energy_min_delta >= 0 ? "+" : ""
      }${effect.energy_min_delta}..=${
        effect.energy_max_delta >= 0 ? "+" : ""
      }${effect.energy_max_delta}），心情 ${moodDelta >= 0 ? "+" : ""}${moodDelta}（范围 ${
        effect.mood_min_delta >= 0 ? "+" : ""
      }${effect.mood_min_delta}..=${
        effect.mood_max_delta >= 0 ? "+" : ""
      }${effect.mood_max_delta}）。`,
      ...world.event_log,
    ];
    return true;
  }

  if (effect.type === "change_weather") {
    world.clock.weather = effect.weather;
    return true;
  }

  world.event_log = [effect.message, ...world.event_log];
  return true;
};

const dialogueConditionMet = (
  world: WorldState,
  condition: DialogueCondition,
) => {
  if (condition.type === "character_at_location") {
    return world.characters.some(
      (character) =>
        character.id === condition.character_id &&
        character.location_id === condition.location_id,
    );
  }

  if (condition.type === "character_mood_at_least") {
    return world.characters.some(
      (character) =>
        character.id === condition.character_id &&
        character.state.mood >= condition.value,
    );
  }

  if (condition.type === "relationship_affinity_at_least") {
    return world.relationships.some(
      (relationship) =>
        relationship.source_character_id === condition.source_character_id &&
        relationship.target_character_id === condition.target_character_id &&
        relationship.affinity >= condition.value,
    );
  }

  if (condition.type === "weather_is") {
    return world.clock.weather === condition.weather;
  }

  return (
    world.clock.hour > condition.hour ||
    (world.clock.hour === condition.hour && world.clock.minute >= condition.minute)
  );
};

export const visibleChoices = (world: WorldState, node: DialogueNode) =>
  node.choices.filter((choice) =>
    choice.conditions.every((condition) => dialogueConditionMet(world, condition)),
  );

const chooseDialogue = (
  world: WorldState,
  nodeId: string,
  choiceId: string,
): boolean => {
  const sceneId = world.active_dialogue_scene_id;
  if (!sceneId) {
    return false;
  }

  const activeNode = world.active_dialogue.find((node) => node.id === nodeId);
  const choice: DialogueChoice | undefined = activeNode
    ? visibleChoices(world, activeNode).find((item) => item.id === choiceId)
    : undefined;
  if (!choice) {
    return false;
  }

  const staged = structuredClone(world);
  for (const effect of choice.effects) {
    if (!applyDialogueEffect(staged, effect)) {
      return false;
    }
  }

  if (choice.next_node_id) {
    const nextNode = findDialogueNode(staged, sceneId, choice.next_node_id);
    if (nextNode) {
      staged.active_dialogue = [
        ...staged.active_dialogue,
        structuredClone(nextNode),
      ];
    }
  } else {
    staged.active_dialogue_scene_id = null;
  }

  staged.event_log = [`选择对话 ${nodeId} / ${choiceId}。`, ...staged.event_log];
  Object.assign(world, staged);
  return true;
};

const applyScheduledEventKind = (
  world: WorldState,
  eventId: string,
  kind: ScheduledEventKind,
) => {
  if (kind.type === "change_weather") {
    world.clock.weather = kind.weather;
    world.event_log = [
      `事件 ${eventId} 触发：天气变为${weatherLabels[kind.weather]}。`,
      ...world.event_log,
    ];
    return;
  }

  if (kind.type === "start_dialogue") {
    startDialogue(world, kind.scene_id);
    return;
  }

  if (kind.type === "adjust_relationship") {
    applyRelationshipDelta(
      world,
      kind.source_character_id,
      kind.target_character_id,
      kind.affinity_delta,
      kind.trust_delta,
    );
    world.event_log = [
      `事件 ${eventId} 触发：关系 ${kind.source_character_id} -> ${kind.target_character_id} 更新。`,
      ...world.event_log,
    ];
    return;
  }

  if (kind.type === "roll_character_state") {
    if (
      kind.energy_min_delta > kind.energy_max_delta ||
      kind.mood_min_delta > kind.mood_max_delta
    ) {
      return;
    }
    const energyDelta = rollInclusive(
      world,
      kind.energy_min_delta,
      kind.energy_max_delta,
    );
    const moodDelta = rollInclusive(world, kind.mood_min_delta, kind.mood_max_delta);
    applyCharacterStateDelta(world, kind.character_id, energyDelta, moodDelta);
    const character = world.characters.find((item) => item.id === kind.character_id);
    world.event_log = [
      `事件 ${eventId} 触发：${
        character?.display_name ?? kind.character_id
      } 随机状态结算。`,
      ...world.event_log,
    ];
    return;
  }

  applyCharacterStateDelta(world, kind.character_id, kind.energy_delta, kind.mood_delta);
  const character = world.characters.find((item) => item.id === kind.character_id);
  world.event_log = [
    `事件 ${eventId} 触发：${character?.display_name ?? kind.character_id} 状态更新。`,
    ...world.event_log,
  ];
};

const nextRepeatedEvent = (event: ScheduledEvent): ScheduledEvent | null => {
  if (!event.repeat) {
    return null;
  }

  const repeat = { ...event.repeat };
  if (repeat.remaining_runs !== null) {
    repeat.remaining_runs = Math.max(0, repeat.remaining_runs - 1);
    if (repeat.remaining_runs === 0) {
      return null;
    }
  }

  return {
    ...event,
    due: scheduledTimeFromAbsoluteMinute(
      absoluteMinute(event.due) + repeat.every_minutes,
    ),
    repeat,
  };
};

const triggerDueEvents = (world: WorldState, endMinute: number) => {
  const dueEvents = world.scheduled_events
    .filter((event) => absoluteMinute(event.due) <= endMinute)
    .sort(byDueTime);
  const pendingEvents = world.scheduled_events.filter(
    (event) => absoluteMinute(event.due) > endMinute,
  );

  while (dueEvents.length > 0) {
    const event = dueEvents.shift()!;
    const conditionsMet = event.conditions.every((condition) =>
      dialogueConditionMet(world, condition),
    );
    if (conditionsMet) {
      applyScheduledEventKind(world, event.id, event.kind);
      const nextEvent = nextRepeatedEvent(event);
      if (nextEvent) {
        if (absoluteMinute(nextEvent.due) <= endMinute) {
          dueEvents.push(nextEvent);
          dueEvents.sort(byDueTime);
        } else {
          pendingEvents.push(nextEvent);
        }
      }
    } else {
      pendingEvents.push(event);
    }
  }

  world.scheduled_events = pendingEvents.sort(byDueTime);
};

export const applyDemoCommand = (
  world: WorldState,
  command: EngineCommand,
): WorldState => {
  const next = structuredClone(world);
  const initialRandom =
    next.command_log_initial_random ?? structuredClone(next.random);

  if (command.type === "advance_time") {
    const total = currentAbsoluteMinute(next) + Math.max(0, command.minutes);
    const minuteOfDay = total % 1440;
    next.clock.day = Math.floor(total / 1440) + 1;
    next.clock.hour = Math.floor(minuteOfDay / 60);
    next.clock.minute = minuteOfDay % 60;
    next.event_log = [`时间推进 ${command.minutes} 分钟。`, ...next.event_log];
    triggerDueEvents(next, total);
    return recordCommand(next, command, initialRandom);
  }

  if (command.type === "move_character") {
    const character = next.characters.find(
      (item) => item.id === command.character_id,
    );
    const location = next.locations.find((item) => item.id === command.location_id);
    if (!character || !location) {
      return next;
    }

    character.location_id = location.id;
    next.event_log = [
      `${character.display_name} 移动到 ${location.name}。`,
      ...next.event_log,
    ];
    return recordCommand(next, command, initialRandom);
  }

  if (command.type === "adjust_relationship") {
    const updated = applyRelationshipDelta(
      next,
      command.source_character_id,
      command.target_character_id,
      command.affinity_delta,
      command.trust_delta,
    );
    if (!updated) {
      return next;
    }

    next.event_log = [
      `关系 ${command.source_character_id} -> ${command.target_character_id} 更新。`,
      ...next.event_log,
    ];
    return recordCommand(next, command, initialRandom);
  }

  if (command.type === "start_dialogue") {
    return startDialogue(next, command.scene_id)
      ? recordCommand(next, command, initialRandom)
      : next;
  }

  if (command.type === "choose_dialogue") {
    return chooseDialogue(next, command.node_id, command.choice_id)
      ? recordCommand(next, command, initialRandom)
      : next;
  }

  if (command.type === "roll_character_mood") {
    const character = next.characters.find(
      (item) => item.id === command.character_id,
    );
    if (!character || command.min_delta > command.max_delta) {
      return next;
    }

    const delta = rollInclusive(next, command.min_delta, command.max_delta);
    character.state.mood = clamp(character.state.mood + delta, -100, 100);
    next.event_log = [
      `${character.display_name} 心情随机变化 ${delta >= 0 ? "+" : ""}${delta}（范围 ${
        command.min_delta >= 0 ? "+" : ""
      }${command.min_delta}..=${command.max_delta >= 0 ? "+" : ""}${
        command.max_delta
      }）。`,
      ...next.event_log,
    ];
    return recordCommand(next, command, initialRandom);
  }

  if (command.type === "schedule_event") {
    const hasDuplicate = next.scheduled_events.some(
      (event) => event.id === command.event.id,
    );
    if (
      !command.event.id.trim() ||
      !isValidDue(command.event.due) ||
      !isValidRepeat(command.event) ||
      hasDuplicate
    ) {
      return next;
    }

    next.scheduled_events = [...next.scheduled_events, command.event].sort(byDueTime);
    return recordCommand(next, command, initialRandom);
  }

  if (command.type === "cancel_event") {
    if (!command.event_id.trim()) {
      return next;
    }

    const pendingEvents = next.scheduled_events.filter(
      (event) => event.id !== command.event_id,
    );
    if (pendingEvents.length === next.scheduled_events.length) {
      return next;
    }

    next.scheduled_events = pendingEvents;
    next.event_log = [`计划事件 ${command.event_id} 已取消。`, ...next.event_log];
    return recordCommand(next, command, initialRandom);
  }

  return next;
};

export const replayDemoCommandLog = (
  world: WorldState,
  replayLog: EngineReplayLog,
): WorldState => {
  let next = structuredClone(world);
  next.random = structuredClone(replayLog.initial_random);
  next.command_log_initial_random = null;
  next.command_log = [];

  for (const command of replayLog.commands) {
    next = applyDemoCommand(next, command);
  }

  return next;
};
