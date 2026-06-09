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

export type ResourceLoadStrategy = "eager" | "deferred" | "thumbnail_only";

export interface ResourceResolution {
  resource_id: string;
  source_path: string;
  resolved_path: string | null;
  media_type: ResourceMediaType;
  status: ResourceResolutionStatus;
  load_strategy: ResourceLoadStrategy;
  cache_key: string;
  cache_path: string | null;
  thumbnail_path: string | null;
  fallback: ResourceFallback;
  expected_sha256: string | null;
  actual_sha256: string | null;
}

export interface ResourceResolutionReport {
  root: string;
  low_spec: boolean;
  entries: ResourceResolution[];
}

export type ResourcePreflightIssueCode =
  | "missing"
  | "unsafe_path"
  | "hash_mismatch"
  | "io_error";

export interface ResourcePreflightIssue {
  code: ResourcePreflightIssueCode;
  resource_id: string;
  source_path: string;
  message: string;
  fallback: ResourceFallback;
}

export interface ResourcePreflightReport {
  root: string;
  low_spec: boolean;
  ready: boolean;
  resolution: ResourceResolutionReport;
  issues: ResourcePreflightIssue[];
}

export type ResourcePublishIssueSeverity = "error" | "warning";

export type ResourcePublishIssueCode =
  | "missing"
  | "unsafe_path"
  | "hash_mismatch"
  | "io_error"
  | "empty_license"
  | "unknown_license"
  | "empty_author"
  | "unknown_author"
  | "missing_sha256";

export interface ResourcePublishIssue {
  severity: ResourcePublishIssueSeverity;
  code: ResourcePublishIssueCode;
  resource_id: string;
  source_path: string;
  message: string;
  fallback: ResourceFallback;
}

export interface ResourcePublishReport {
  root: string;
  low_spec: boolean;
  ready: boolean;
  error_count: number;
  warning_count: number;
  resolution: ResourceResolutionReport;
  issues: ResourcePublishIssue[];
}

export type ResourceCacheStatus = "cached" | "skipped" | "failed";

export interface ResourceCacheEntry {
  resource_id: string;
  source_path: string;
  cache_path: string | null;
  status: ResourceCacheStatus;
  bytes_copied: number;
  message: string;
}

export interface ResourceCacheReport {
  root: string;
  low_spec: boolean;
  ready: boolean;
  cached_count: number;
  skipped_count: number;
  failed_count: number;
  resolution: ResourceResolutionReport;
  entries: ResourceCacheEntry[];
}

export type ResourceCacheCleanStatus = "kept" | "removed" | "skipped" | "failed";

export interface ResourceCacheCleanEntry {
  path: string;
  status: ResourceCacheCleanStatus;
  bytes_removed: number;
  message: string;
}

export interface ResourceCacheCleanReport {
  root: string;
  low_spec: boolean;
  ready: boolean;
  cache_root: string;
  kept_count: number;
  removed_count: number;
  skipped_count: number;
  failed_count: number;
  bytes_removed: number;
  resolution: ResourceResolutionReport;
  entries: ResourceCacheCleanEntry[];
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
  | "unknown_capability"
  | "unsafe_install_namespace"
  | "unsafe_package_version"
  | "template_target_not_empty"
  | "unsupported_package_schema"
  | "package_manifest_mismatch"
  | "install_target_exists"
  | "install_root_not_directory"
  | "install_staging_exists"
  | "install_target_missing"
  | "install_target_not_directory"
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
  staging_root: string;
  manifest_path: string;
  manifest: ModManifest;
  actions: ModInstallActionReport[];
}

export interface ModInstallReport {
  target_root: string;
  manifest: ModManifest;
  actions: ModInstallActionReport[];
}

export interface ModInstallPreflightReport {
  source_root: string;
  content_root: string | null;
  install_root: string;
  target_root: string | null;
  staging_root: string | null;
  manifest: ModManifest | null;
  ready: boolean;
  issues: ModInstallPreflightIssueReport[];
}

export interface ModInstallPreflightIssueReport {
  severity: "error" | "warning";
  path: string;
  kind: ModDiscoveryIssueKind;
  message: string;
}

export interface ModUninstallPlanReport {
  install_root: string;
  target_root: string;
  staging_root: string;
  namespace: string;
  actions: ModInstallActionReport[];
}

export interface ModUninstallReport {
  namespace: string;
  target_root: string;
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
    }
  | {
      kind: "move_directory";
      path: null;
      from: string;
      to: string;
    }
  | {
      kind: "delete_directory";
      path: string;
      from: null;
      to: null;
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

export interface ModRegistryEntry {
  namespace: string;
  version: string;
  conflicts: string[];
}

export interface ModRegistry {
  enabled: ModRegistryEntry[];
}

export type ModLoadErrorKind =
  | "save"
  | "unknown_capability"
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

export interface EngineReplayLog {
  schema_version: number;
  engine_version: string;
  initial_random: WorldRandom;
  commands: EngineCommand[];
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
  | {
      type: "roll_character_state";
      character_id: string;
      energy_min_delta: number;
      energy_max_delta: number;
      mood_min_delta: number;
      mood_max_delta: number;
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
    }
  | {
      type: "roll_character_state";
      character_id: string;
      energy_min_delta: number;
      energy_max_delta: number;
      mood_min_delta: number;
      mood_max_delta: number;
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
  command_log_initial_random: WorldRandom | null;
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
  replay_log: EngineReplayLog | null;
  world: WorldState;
}

export interface SaveValidationReport {
  missing_required_mods: SaveModDependency[];
  incompatible_schema: number | null;
  engine_version_mismatch: boolean;
}

export interface SavePreflightReport {
  slot_id: string;
  path: string;
  ready: boolean;
  registry: ModRegistry;
  discovery: ModDiscoveryReport;
  validation: SaveValidationReport;
  save: SaveEnvelope;
}

export interface SaveSlotReport {
  path: string;
  backup_path: string | null;
}

export interface SaveRecoveryReport {
  path: string;
  recovered_from: string;
  failed_primary_backup_path: string | null;
  save: SaveEnvelope;
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

export type ContentIssueCode =
  | "empty_package_id"
  | "empty_namespace"
  | "empty_version"
  | "empty_dependency_package_id"
  | "empty_dependency_version"
  | "duplicate_dependency_package_id"
  | "self_dependency"
  | "empty_conflict_package_id"
  | "duplicate_conflict_package_id"
  | "self_conflict"
  | "empty_location_id"
  | "duplicate_location_id"
  | "empty_location_name"
  | "empty_location_terrain"
  | "empty_character_id"
  | "duplicate_character_id"
  | "empty_character_name"
  | "empty_character_location"
  | "empty_relationship_reference"
  | "duplicate_relationship"
  | "empty_resource_id"
  | "duplicate_resource_id"
  | "empty_resource_path"
  | "unsafe_resource_path"
  | "empty_resource_license"
  | "empty_resource_author"
  | "duplicate_dialogue_scene_id"
  | "duplicate_dialogue_node_id"
  | "empty_dialogue_scene_id"
  | "empty_dialogue_node_id"
  | "empty_dialogue_text"
  | "empty_dialogue_resource_ref"
  | "invalid_dialogue_placeholder"
  | "unknown_dialogue_placeholder"
  | "dialogue_placeholder_type_mismatch"
  | "missing_entry_node"
  | "missing_choice_next_node"
  | "empty_condition_reference"
  | "invalid_condition_time"
  | "invalid_effect_random_range"
  | "unreachable_dialogue_node"
  | "empty_scheduled_event_id"
  | "duplicate_scheduled_event_id"
  | "invalid_scheduled_event_time"
  | "invalid_scheduled_repeat"
  | "invalid_scheduled_event_random_range"
  | "empty_scheduled_event_reference";

export interface ContentIssue {
  code: ContentIssueCode;
  target: string;
}

export interface ContentValidationReport {
  issues: ContentIssue[];
}

export type ContentInstallPreflightIssueCode =
  | "validation_failed"
  | "duplicate_dialogue_scene"
  | "duplicate_scheduled_event"
  | "duplicate_location"
  | "duplicate_character"
  | "duplicate_relationship"
  | "duplicate_resource"
  | "duplicate_content_package"
  | "missing_content_package_dependency"
  | "content_package_dependency_version_mismatch"
  | "content_package_conflict"
  | "missing_location_reference"
  | "missing_character_reference"
  | "missing_relationship_reference"
  | "missing_dialogue_resource"
  | "missing_scheduled_event_scene";

export interface ContentInstallPreflightIssue {
  code: ContentInstallPreflightIssueCode;
  message: string;
}

export interface ContentInstallPreflightReport {
  package_id: string;
  namespace: string;
  version: string;
  ready: boolean;
  validation: ContentValidationReport | null;
  issues: ContentInstallPreflightIssue[];
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
