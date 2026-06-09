export type Season = "spring" | "summer" | "autumn" | "winter";
export type Weather = "clear" | "cloudy" | "rain" | "snow";

export interface WorldClock {
  day: number;
  hour: number;
  minute: number;
  season: Season;
  weather: Weather;
}

export interface ScheduledTime {
  day: number;
  hour: number;
  minute: number;
}

export interface Location {
  id: string;
  name: string;
  ascii_symbol: string;
  terrain: string;
}

export interface CharacterState {
  energy: number;
  mood: number;
}

export interface Character {
  id: string;
  display_name: string;
  location_id: string;
  state: CharacterState;
}

export interface Relationship {
  source_character_id: string;
  target_character_id: string;
  affinity: number;
  trust: number;
}

export interface WorldRandom {
  seed: string;
  cursor: string;
}

export interface DialogueNode {
  id: string;
  speaker_id: string;
  text: string;
  choices: DialogueChoice[];
}

export interface DialogueChoice {
  id: string;
  label: string;
  next_node_id: string | null;
  conditions: DialogueCondition[];
  effects: DialogueEffect[];
}

export type DialogueCondition =
  | {
      type: "character_at_location";
      character_id: string;
      location_id: string;
    }
  | {
      type: "character_mood_at_least";
      character_id: string;
      value: number;
    }
  | {
      type: "relationship_affinity_at_least";
      source_character_id: string;
      target_character_id: string;
      value: number;
    }
  | { type: "weather_is"; weather: Weather }
  | { type: "time_at_least"; hour: number; minute: number };

export type DialogueEffect =
  | {
      type: "adjust_character_state";
      character_id: string;
      energy_delta: number;
      mood_delta: number;
    }
  | {
      type: "adjust_relationship";
      source_character_id: string;
      target_character_id: string;
      affinity_delta: number;
      trust_delta: number;
    }
  | { type: "change_weather"; weather: Weather }
  | { type: "add_log"; message: string };

export interface DialogueScene {
  id: string;
  entry_node_id: string;
  nodes: DialogueNode[];
}

export type ScheduledEventKind =
  | { type: "change_weather"; weather: Weather }
  | { type: "start_dialogue"; scene_id: string }
  | {
      type: "adjust_relationship";
      source_character_id: string;
      target_character_id: string;
      affinity_delta: number;
      trust_delta: number;
    }
  | {
      type: "adjust_character_state";
      character_id: string;
      energy_delta: number;
      mood_delta: number;
    };

export interface ScheduledEvent {
  id: string;
  due: ScheduledTime;
  conditions: DialogueCondition[];
  kind: ScheduledEventKind;
}

export interface WorldState {
  engine_version: string;
  clock: WorldClock;
  locations: Location[];
  characters: Character[];
  relationships: Relationship[];
  dialogue_scenes: DialogueScene[];
  active_dialogue_scene_id: string | null;
  active_dialogue: DialogueNode[];
  scheduled_events: ScheduledEvent[];
  random: WorldRandom;
  command_log: EngineCommand[];
  event_log: string[];
}

export interface SaveModDependency {
  namespace: string;
  version: string;
  required: boolean;
}

export interface SaveEnvelope {
  schema_version: number;
  engine_version: string;
  saved_at_unix_ms: number;
  slot_id: string;
  mod_dependencies: SaveModDependency[];
  world: WorldState;
}

export interface SaveSlotReport {
  path: string;
  backup_path: string | null;
}

export type EngineCommand =
  | { type: "advance_time"; minutes: number }
  | { type: "move_character"; character_id: string; location_id: string }
  | {
      type: "adjust_relationship";
      source_character_id: string;
      target_character_id: string;
      affinity_delta: number;
      trust_delta: number;
    }
  | { type: "start_dialogue"; scene_id: string }
  | { type: "choose_dialogue"; node_id: string; choice_id: string }
  | {
      type: "roll_character_mood";
      character_id: string;
      min_delta: number;
      max_delta: number;
    }
  | { type: "schedule_event"; event: ScheduledEvent };
