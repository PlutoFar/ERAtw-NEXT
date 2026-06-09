import type {
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
  event_log: ["ERAtw-NEXT M0 engine ready."],
});

const startDialogue = (world: WorldState, sceneId: string) => {
  world.active_dialogue = [
    {
      id: `${sceneId}_001`,
      speaker_id: "demo_heroine",
      text: "早上好。今天先从一个干净的新世界开始。",
    },
    {
      id: `${sceneId}_002`,
      speaker_id: "system",
      text: "该对话来自版本化 DialogueNode，不执行旧 ERB。",
    },
  ];
  world.event_log = [`播放场景 ${sceneId}。`, ...world.event_log];
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

  const character = world.characters.find((item) => item.id === kind.character_id);
  if (!character) {
    return;
  }

  character.state.energy = clamp(character.state.energy + kind.energy_delta, 0, 100);
  character.state.mood = clamp(character.state.mood + kind.mood_delta, -100, 100);
  world.event_log = [
    `事件 ${eventId} 触发：${character.display_name} 状态更新。`,
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
    return next;
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
    return next;
  }

  if (command.type === "start_dialogue") {
    startDialogue(next, command.scene_id);
  }

  if (command.type === "schedule_event") {
    const hasDuplicate = next.scheduled_events.some(
      (event) => event.id === command.event.id,
    );
    if (!command.event.id.trim() || !isValidDue(command.event.due) || hasDuplicate) {
      return next;
    }

    next.scheduled_events = [...next.scheduled_events, command.event].sort(byDueTime);
  }

  return next;
};
