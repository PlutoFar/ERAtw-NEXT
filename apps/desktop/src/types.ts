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

export type ResourceMediaType = "image" | "audio" | "font" | "other";

export interface ResourceAsset {
  resource_id: string;
  source_path: string;
  media_type: ResourceMediaType;
  license: string;
  author: string;
  usage: string[];
  character_bindings: string[];
  tags: string[];
  sha256: string | null;
}

export type ResourceResolutionStatus =
  | "planned"
  | "ready"
  | "missing"
  | "unsafe_path"
  | "hash_mismatch"
  | "io_error";

export type ResourceFallback =
  | "placeholder_image"
  | "silent_audio"
  | "default_font"
  | "missing_resource";

export interface ResourceResolution {
  resource_id: string;
  source_path: string;
  resolved_path: string | null;
  media_type: ResourceMediaType;
  status: ResourceResolutionStatus;
  fallback: ResourceFallback;
  expected_sha256: string | null;
  actual_sha256: string | null;
}

export interface ResourceResolutionReport {
  root: string;
  entries: ResourceResolution[];
}

export type ModCapability =
  | "content"
  | "theme"
  | "rules_extension"
  | "local_file_access"
  | "network_access"
  | "system_command";

export interface ModDependency {
  namespace: string;
  version: string | null;
  required: boolean;
}

export interface ModManifest {
  namespace: string;
  name: string;
  version: string;
  engine_version: string;
  load_order: number;
  dependencies: ModDependency[];
  conflicts: string[];
  capabilities: ModCapability[];
}

export interface DiscoveredModReport {
  root_path: string;
  manifest_path: string;
  manifest: ModManifest;
}

export type ModDiscoveryIssueKind =
  | "io"
  | "json"
  | "unsafe_install_namespace"
  | "missing_namespace"
  | "missing_name"
  | "missing_version"
  | "missing_engine_version"
  | "incompatible_engine_version"
  | "duplicate_dependency"
  | "duplicate_conflict"
  | "unsafe_capability";

export interface ModDiscoveryIssueReport {
  path: string;
  kind: ModDiscoveryIssueKind;
  message: string;
}

export interface ModDiscoveryReport {
  root_path: string;
  discovered: DiscoveredModReport[];
  errors: ModDiscoveryIssueReport[];
}

export interface ModInstallPlanReport {
  source_root: string;
  install_root: string;
  target_root: string;
  manifest_path: string;
  manifest: ModManifest;
  actions: ModInstallActionReport[];
}

export type ModInstallActionReport =
  | {
      kind: "create_directory";
      path: string;
      from: null;
      to: null;
    }
  | {
      kind: "copy_directory";
      path: null;
      from: string;
      to: string;
    };

export interface ModEnablement {
  namespace: string;
  enabled: boolean;
}

export interface DisabledModReport {
  manifest: ModManifest;
  reason: "user_disabled";
}

export interface ModEnablementPlanReport {
  enabled: ModManifest[];
  disabled: DisabledModReport[];
}

export type ModLoadErrorKind =
  | "missing_namespace"
  | "missing_name"
  | "missing_version"
  | "missing_engine_version"
  | "incompatible_engine_version"
  | "duplicate_dependency"
  | "duplicate_conflict"
  | "unsafe_capability"
  | "duplicate_enablement"
  | "unknown_enablement"
  | "duplicate_namespace"
  | "missing_dependency"
  | "dependency_version_mismatch"
  | "conflict"
  | "dependency_cycle";

export interface ModLoadErrorReport {
  kind: ModLoadErrorKind;
  message: string;
}

export interface InstalledContentPackage {
  namespace: string;
  package_id: string;
  version: string;
  dependencies: ContentPackageDependencyObject[];
  conflicts: string[];
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
  resource_refs: string[];
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
  priority: number;
  repeat: ScheduledRepeat | null;
  conditions: DialogueCondition[];
  kind: ScheduledEventKind;
}

export interface ScheduledRepeat {
  every_minutes: number;
  remaining_runs: number | null;
}

export interface WorldState {
  engine_version: string;
  installed_content_packages: InstalledContentPackage[];
  clock: WorldClock;
  locations: Location[];
  characters: Character[];
  resources: ResourceAsset[];
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

export interface ContentPackageManifest {
  schema_version: string;
  namespace: string;
  package_id: string;
  version: string;
  dependencies: ContentPackageDependency[];
  conflicts: string[];
}

export type ContentPackageDependency = string | ContentPackageDependencyObject;

export interface ContentPackageDependencyObject {
  package_id: string;
  version: string | null;
  required: boolean;
}

export interface ContentPackage {
  manifest: ContentPackageManifest;
  locations: Location[];
  characters: Character[];
  relationships: Relationship[];
  resources: ResourceAsset[];
  dialogue_scenes: DialogueScene[];
  scheduled_events: ScheduledEvent[];
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
  | { type: "schedule_event"; event: ScheduledEvent }
  | { type: "cancel_event"; event_id: string };
