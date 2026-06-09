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
  effects: DialogueEffect[];
}

export type DialogueEffect =
  | {
      type: "adjust_character_state";
      character_id: string;
      energy_delta: number;
      mood_delta: number;
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
      type: "adjust_character_state";
      character_id: string;
      energy_delta: number;
      mood_delta: number;
    };

export interface ScheduledEvent {
  id: string;
  due: ScheduledTime;
  kind: ScheduledEventKind;
}

export interface WorldState {
  engine_version: string;
  clock: WorldClock;
  locations: Location[];
  characters: Character[];
  dialogue_scenes: DialogueScene[];
  active_dialogue_scene_id: string | null;
  active_dialogue: DialogueNode[];
  scheduled_events: ScheduledEvent[];
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
  | { type: "start_dialogue"; scene_id: string }
  | { type: "choose_dialogue"; node_id: string; choice_id: string }
  | { type: "schedule_event"; event: ScheduledEvent };
