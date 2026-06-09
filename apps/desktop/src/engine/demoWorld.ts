import type {
  DialogueChoice,
  DialogueCondition,
  DialogueEffect,
  DialogueNode,
  DialogueScene,
  EngineCommand,
  ScheduledEvent,
  ScheduledEventKind,
  ScheduledTime,
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

const currentAbsoluteMinute = (world: WorldState) =>
  absoluteMinute({
    day: world.clock.day,
    hour: world.clock.hour,
    minute: world.clock.minute,
  });

const byDueTime = (left: ScheduledEvent, right: ScheduledEvent) => {
  const delta = absoluteMinute(left.due) - absoluteMinute(right.due);
  return delta === 0 ? left.id.localeCompare(right.id) : delta;
};

const isValidDue = (due: ScheduledTime) =>
  due.day > 0 && due.hour >= 0 && due.hour < 24 && due.minute >= 0 && due.minute < 60;

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
        choices: [],
      },
      {
        id: "demo_morning_003",
        speaker_id: "demo_heroine",
        text: "嗯。先把能稳定重放的小循环做好。",
        choices: [],
      },
      {
        id: "demo_morning_004",
        speaker_id: "demo_heroine",
        text: "信任会一点点积累。先从可验证的承诺开始。",
        choices: [],
      },
    ],
  },
];

export const createDemoWorld = (): WorldState => ({
  engine_version: "0.1.0-m0",
  clock: {
    day: 1,
    hour: 8,
    minute: 0,
    season: "spring",
    weather: "clear",
  },
  locations: [
    {
      id: "school_gate",
      name: "校门",
      ascii_symbol: "門",
      terrain: "street",
    },
    {
      id: "club_room",
      name: "社团室",
      ascii_symbol: "部",
      terrain: "interior",
    },
    {
      id: "garden",
      name: "庭园",
      ascii_symbol: "庭",
      terrain: "grass",
    },
  ],
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
      kind: { type: "change_weather", weather: "cloudy" },
    },
    {
      id: "demo_morning_mood",
      due: { day: 1, hour: 9, minute: 0 },
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
  command_log: [],
  event_log: ["ERAtw-NEXT M0 engine ready."],
});

const recordCommand = (world: WorldState, command: EngineCommand): WorldState => ({
  ...world,
  command_log: [...world.command_log, structuredClone(command)],
});

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
    return;
  }

  character.state.energy = clamp(character.state.energy + energyDelta, 0, 100);
  character.state.mood = clamp(character.state.mood + moodDelta, -100, 100);
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
    applyCharacterStateDelta(
      world,
      effect.character_id,
      effect.energy_delta,
      effect.mood_delta,
    );
    return;
  }

  if (effect.type === "adjust_relationship") {
    applyRelationshipDelta(
      world,
      effect.source_character_id,
      effect.target_character_id,
      effect.affinity_delta,
      effect.trust_delta,
    );
    return;
  }

  if (effect.type === "change_weather") {
    world.clock.weather = effect.weather;
    return;
  }

  world.event_log = [effect.message, ...world.event_log];
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

  for (const effect of choice.effects) {
    applyDialogueEffect(world, effect);
  }

  if (choice.next_node_id) {
    const nextNode = findDialogueNode(world, sceneId, choice.next_node_id);
    if (nextNode) {
      world.active_dialogue = [...world.active_dialogue, structuredClone(nextNode)];
    }
  } else {
    world.active_dialogue_scene_id = null;
  }

  world.event_log = [`选择对话 ${nodeId} / ${choiceId}。`, ...world.event_log];
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

  applyCharacterStateDelta(
    world,
    kind.character_id,
    kind.energy_delta,
    kind.mood_delta,
  );
  const character = world.characters.find((item) => item.id === kind.character_id);
  world.event_log = [
    `事件 ${eventId} 触发：${character?.display_name ?? kind.character_id} 状态更新。`,
    ...world.event_log,
  ];
};

const triggerDueEvents = (world: WorldState, endMinute: number) => {
  const dueEvents = world.scheduled_events
    .filter((event) => absoluteMinute(event.due) <= endMinute)
    .sort(byDueTime);
  world.scheduled_events = world.scheduled_events.filter(
    (event) => absoluteMinute(event.due) > endMinute,
  );

  for (const event of dueEvents) {
    applyScheduledEventKind(world, event.id, event.kind);
  }
};

export const applyDemoCommand = (
  world: WorldState,
  command: EngineCommand,
): WorldState => {
  const next = structuredClone(world);

  if (command.type === "advance_time") {
    const total = currentAbsoluteMinute(next) + Math.max(0, command.minutes);
    const minuteOfDay = total % 1440;
    next.clock.day = Math.floor(total / 1440) + 1;
    next.clock.hour = Math.floor(minuteOfDay / 60);
    next.clock.minute = minuteOfDay % 60;
    next.event_log = [`时间推进 ${command.minutes} 分钟。`, ...next.event_log];
    triggerDueEvents(next, total);
    return recordCommand(next, command);
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
    return recordCommand(next, command);
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
    return recordCommand(next, command);
  }

  if (command.type === "start_dialogue") {
    return startDialogue(next, command.scene_id)
      ? recordCommand(next, command)
      : next;
  }

  if (command.type === "choose_dialogue") {
    return chooseDialogue(next, command.node_id, command.choice_id)
      ? recordCommand(next, command)
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
    return recordCommand(next, command);
  }

  if (command.type === "schedule_event") {
    const hasDuplicate = next.scheduled_events.some(
      (event) => event.id === command.event.id,
    );
    if (!command.event.id.trim() || !isValidDue(command.event.due) || hasDuplicate) {
      return next;
    }

    next.scheduled_events = [...next.scheduled_events, command.event].sort(byDueTime);
    return recordCommand(next, command);
  }

  return next;
};
