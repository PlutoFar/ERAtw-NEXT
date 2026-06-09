import type { EngineCommand, WorldState } from "../types";

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
  event_log: ["ERAtw-NEXT M0 engine ready."],
});

export const applyDemoCommand = (
  world: WorldState,
  command: EngineCommand,
): WorldState => {
  const next = structuredClone(world);

  if (command.type === "advance_time") {
    const total =
      next.clock.hour * 60 + next.clock.minute + Math.max(0, command.minutes);
    const dayOffset = Math.floor(total / 1440);
    const minuteOfDay = total % 1440;
    next.clock.day += dayOffset;
    next.clock.hour = Math.floor(minuteOfDay / 60);
    next.clock.minute = minuteOfDay % 60;
    next.event_log = [`时间推进 ${command.minutes} 分钟。`, ...next.event_log];
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
    next.active_dialogue = [
      {
        id: `${command.scene_id}_001`,
        speaker_id: "demo_heroine",
        text: "早上好。今天先从一个干净的新世界开始。",
      },
      {
        id: `${command.scene_id}_002`,
        speaker_id: "system",
        text: "该对话来自版本化 DialogueNode，不执行旧 ERB。",
      },
    ];
    next.event_log = [`播放场景 ${command.scene_id}。`, ...next.event_log];
  }

  return next;
};
