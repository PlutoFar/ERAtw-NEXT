export type Season = "spring" | "summer" | "autumn" | "winter";
export type Weather = "clear" | "cloudy" | "rain" | "snow";

export interface WorldClock {
  day: number;
  hour: number;
  minute: number;
  season: Season;
  weather: Weather;
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
}

export interface WorldState {
  engine_version: string;
  clock: WorldClock;
  locations: Location[];
  characters: Character[];
  active_dialogue: DialogueNode[];
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

export type EngineCommand =
  | { type: "advance_time"; minutes: number }
  | { type: "move_character"; character_id: string; location_id: string }
  | { type: "start_dialogue"; scene_id: string };
