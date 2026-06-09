import { invoke } from "@tauri-apps/api/core";
import { applyDemoCommand, createDemoWorld } from "./demoWorld";
import type {
  ContentPackage,
  EngineCommand,
  ResourceAsset,
  ResourceResolutionReport,
  SaveEnvelope,
  SaveSlotReport,
  WorldState,
} from "../types";

export interface EngineClient {
  snapshot(): Promise<WorldState>;
  dispatch(command: EngineCommand): Promise<WorldState>;
  installContentPackage(packageData: ContentPackage): Promise<WorldState>;
  planResources(root: string): Promise<ResourceResolutionReport>;
  inspectResources(root: string): Promise<ResourceResolutionReport>;
  savePreview(slotId: string, savedAtUnixMs: number): Promise<SaveEnvelope>;
  saveSlot(slotId: string, savedAtUnixMs: number): Promise<SaveSlotReport>;
  loadSlot(slotId: string): Promise<WorldState>;
}

const isTauriRuntime = () =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const createTauriEngineClient = (): EngineClient => ({
  snapshot: () => invoke<WorldState>("engine_snapshot"),
  dispatch: (command) => invoke<WorldState>("engine_dispatch", { command }),
  installContentPackage: (packageData) =>
    invoke<WorldState>("engine_install_content_package", {
      package: packageData,
    }),
  planResources: (root) =>
    invoke<ResourceResolutionReport>("engine_plan_resources", { root }),
  inspectResources: (root) =>
    invoke<ResourceResolutionReport>("engine_inspect_resources", { root }),
  savePreview: (slotId, savedAtUnixMs) =>
    invoke<SaveEnvelope>("engine_save_preview", {
      slotId,
      savedAtUnixMs,
    }),
  saveSlot: (slotId, savedAtUnixMs) =>
    invoke<SaveSlotReport>("engine_save_slot", {
      slotId,
      savedAtUnixMs,
    }),
  loadSlot: (slotId) =>
    invoke<WorldState>("engine_load_slot", {
      slotId,
    }),
});

const scheduledEventMinute = (event: ContentPackage["scheduled_events"][number]) =>
  Math.max(0, event.due.day - 1) * 1440 + event.due.hour * 60 + event.due.minute;

const byScheduledEventOrder = (
  left: ContentPackage["scheduled_events"][number],
  right: ContentPackage["scheduled_events"][number],
) => {
  const dueDelta = scheduledEventMinute(left) - scheduledEventMinute(right);
  if (dueDelta !== 0) {
    return dueDelta;
  }

  const priorityDelta = right.priority - left.priority;
  return priorityDelta === 0 ? left.id.localeCompare(right.id) : priorityDelta;
};

const isValidScheduledEvent = (event: ContentPackage["scheduled_events"][number]) =>
  event.id.trim() &&
  event.due.day > 0 &&
  event.due.hour >= 0 &&
  event.due.hour < 24 &&
  event.due.minute >= 0 &&
  event.due.minute < 60 &&
  (event.repeat === null ||
    (event.repeat.every_minutes > 0 && event.repeat.remaining_runs !== 0));

const isValidResource = (resource: ContentPackage["resources"][number]) =>
  resource.resource_id.trim() &&
  resource.source_path.trim() &&
  isSafeResourceSourcePath(resource.source_path) &&
  resource.license.trim() &&
  resource.license.trim() !== "unknown" &&
  resource.author.trim() &&
  resource.author.trim() !== "unknown";

const isBuiltInCharacter = (characterId: string) =>
  characterId === "player" || characterId === "system";

const hasCharacter = (characterIds: Set<string>, characterId: string) =>
  isBuiltInCharacter(characterId) || characterIds.has(characterId);

const relationshipKey = (sourceCharacterId: string, targetCharacterId: string) =>
  `${sourceCharacterId}\u0000${targetCharacterId}`;

const installedPackageKey = (namespace: string, packageId: string) =>
  `${namespace}\u0000${packageId}`;

const installedPackageById = (world: WorldState, packageId: string) =>
  world.installed_content_packages.find((packageInfo) => packageInfo.package_id === packageId);

const isSafeResourceSourcePath = (sourcePath: string) => {
  const normalized = sourcePath.replaceAll("\\", "/");
  return (
    normalized.trim().length > 0 &&
    !normalized.startsWith("/") &&
    !/^[A-Za-z]:\//.test(normalized) &&
    normalized
      .split("/")
      .filter((part) => part.length > 0 && part !== ".")
      .every((part) => part !== "..")
  );
};

const fallbackForMediaType = (mediaType: ResourceAsset["media_type"]) => {
  if (mediaType === "image") {
    return "placeholder_image" as const;
  }
  if (mediaType === "audio") {
    return "silent_audio" as const;
  }
  if (mediaType === "font") {
    return "default_font" as const;
  }
  return "missing_resource" as const;
};

const normalizeResourcePath = (sourcePath: string) =>
  sourcePath
    .replaceAll("\\", "/")
    .split("/")
    .filter((part) => part.length > 0 && part !== ".")
    .join("/");

const joinResourcePath = (root: string, sourcePath: string) => {
  const normalizedRoot = root.replaceAll("\\", "/").replace(/\/+$/, "");
  const normalizedSource = normalizeResourcePath(sourcePath);
  return normalizedRoot ? `${normalizedRoot}/${normalizedSource}` : normalizedSource;
};

const planResourcesForBrowserWorld = (world: WorldState, root: string) => ({
  root,
  entries: world.resources.map((resource) => {
    const safe = isSafeResourceSourcePath(resource.source_path);
    return {
      resource_id: resource.resource_id,
      source_path: resource.source_path,
      resolved_path: safe ? joinResourcePath(root, resource.source_path) : null,
      media_type: resource.media_type,
      status: safe ? ("planned" as const) : ("unsafe_path" as const),
      fallback: fallbackForMediaType(resource.media_type),
      expected_sha256: resource.sha256,
      actual_sha256: null,
    };
  }),
});

const normalizeContentPackageDependency = (
  dependency: ContentPackage["manifest"]["dependencies"][number],
) =>
  typeof dependency === "string"
    ? {
        package_id: dependency,
        version: null,
        required: true,
      }
    : {
        package_id: dependency.package_id,
        version: dependency.version ?? null,
        required: dependency.required ?? true,
      };

const normalizeContentPackageDependencies = (packageData: ContentPackage) =>
  packageData.manifest.dependencies.map(normalizeContentPackageDependency);

const modDependenciesForWorld = (world: WorldState) =>
  [...world.installed_content_packages]
    .sort((left, right) => left.package_id.localeCompare(right.package_id))
    .map((packageInfo) => ({
      namespace: packageInfo.package_id,
      version: packageInfo.version,
      required: true,
    }));

const conditionRefsExist = (
  condition: ContentPackage["dialogue_scenes"][number]["nodes"][number]["choices"][number]["conditions"][number],
  characterIds: Set<string>,
  locationIds: Set<string>,
  relationshipIds: Set<string>,
) => {
  if (condition.type === "character_at_location") {
    return (
      hasCharacter(characterIds, condition.character_id) &&
      locationIds.has(condition.location_id)
    );
  }

  if (condition.type === "character_mood_at_least") {
    return hasCharacter(characterIds, condition.character_id);
  }

  if (condition.type === "relationship_affinity_at_least") {
    return (
      hasCharacter(characterIds, condition.source_character_id) &&
      hasCharacter(characterIds, condition.target_character_id) &&
      relationshipIds.has(
        relationshipKey(condition.source_character_id, condition.target_character_id),
      )
    );
  }

  return true;
};

const effectRefsExist = (
  effect: ContentPackage["dialogue_scenes"][number]["nodes"][number]["choices"][number]["effects"][number],
  characterIds: Set<string>,
  relationshipIds: Set<string>,
) => {
  if (effect.type === "adjust_character_state") {
    return hasCharacter(characterIds, effect.character_id);
  }

  if (effect.type === "adjust_relationship") {
    return (
      hasCharacter(characterIds, effect.source_character_id) &&
      hasCharacter(characterIds, effect.target_character_id) &&
      relationshipIds.has(
        relationshipKey(effect.source_character_id, effect.target_character_id),
      )
    );
  }

  return true;
};

const scheduledKindRefsExist = (
  event: ContentPackage["scheduled_events"][number],
  characterIds: Set<string>,
  relationshipIds: Set<string>,
) => {
  if (event.kind.type === "adjust_character_state") {
    return hasCharacter(characterIds, event.kind.character_id);
  }

  if (event.kind.type === "adjust_relationship") {
    return (
      hasCharacter(characterIds, event.kind.source_character_id) &&
      hasCharacter(characterIds, event.kind.target_character_id) &&
      relationshipIds.has(
        relationshipKey(event.kind.source_character_id, event.kind.target_character_id),
      )
    );
  }

  return true;
};

const installPackageIntoBrowserWorld = (
  world: WorldState,
  packageData: ContentPackage,
) => {
  if (
    packageData.manifest.schema_version !== "content-package/v0" ||
    !packageData.manifest.namespace.trim() ||
    !packageData.manifest.package_id.trim() ||
    !packageData.manifest.version.trim()
  ) {
    return world;
  }

  const installedPackageIds = new Set(
    world.installed_content_packages.map((packageInfo) =>
      installedPackageKey(packageInfo.namespace, packageInfo.package_id),
    ),
  );
  if (
    installedPackageIds.has(
      installedPackageKey(
        packageData.manifest.namespace,
        packageData.manifest.package_id,
      ),
    )
  ) {
    return world;
  }

  const dependencies = normalizeContentPackageDependencies(packageData);
  for (const dependency of dependencies) {
    if (!dependency.package_id.trim()) {
      return world;
    }

    const installed = installedPackageById(world, dependency.package_id);
    if (!installed && dependency.required) {
      return world;
    }
    if (
      installed &&
      dependency.version !== null &&
      installed.version !== dependency.version
    ) {
      return world;
    }
  }

  if (
    packageData.manifest.conflicts.some(
      (conflict) => !conflict.trim() || installedPackageById(world, conflict),
    ) ||
    world.installed_content_packages.some((packageInfo) =>
      packageInfo.conflicts.includes(packageData.manifest.package_id),
    )
  ) {
    return world;
  }

  const sceneIds = new Set(world.dialogue_scenes.map((scene) => scene.id));
  const locationIds = new Set(world.locations.map((location) => location.id));
  const characterIds = new Set(world.characters.map((character) => character.id));
  const relationshipIds = new Set(
    world.relationships.map((relationship) =>
      relationshipKey(
        relationship.source_character_id,
        relationship.target_character_id,
      ),
    ),
  );
  const resourceIds = new Set(world.resources.map((resource) => resource.resource_id));

  for (const location of packageData.locations) {
    if (
      !location.id.trim() ||
      !location.name.trim() ||
      !location.terrain.trim() ||
      locationIds.has(location.id)
    ) {
      return world;
    }
    locationIds.add(location.id);
  }

  for (const character of packageData.characters) {
    if (
      !character.id.trim() ||
      !character.display_name.trim() ||
      !character.location_id.trim() ||
      characterIds.has(character.id) ||
      !locationIds.has(character.location_id)
    ) {
      return world;
    }
    characterIds.add(character.id);
  }

  for (const relationship of packageData.relationships) {
    const key = relationshipKey(
      relationship.source_character_id,
      relationship.target_character_id,
    );
    if (
      !relationship.source_character_id.trim() ||
      !relationship.target_character_id.trim() ||
      !hasCharacter(characterIds, relationship.source_character_id) ||
      !hasCharacter(characterIds, relationship.target_character_id) ||
      relationshipIds.has(key)
    ) {
      return world;
    }
    relationshipIds.add(key);
  }

  for (const resource of packageData.resources) {
    if (!isValidResource(resource) || resourceIds.has(resource.resource_id)) {
      return world;
    }
    resourceIds.add(resource.resource_id);
  }

  for (const scene of packageData.dialogue_scenes) {
    if (!scene.id.trim() || sceneIds.has(scene.id)) {
      return world;
    }
    for (const node of scene.nodes) {
      if (
        !hasCharacter(characterIds, node.speaker_id) ||
        node.resource_refs.some((resourceId) => !resourceIds.has(resourceId))
      ) {
        return world;
      }
      for (const choice of node.choices) {
        if (
          choice.conditions.some(
            (condition) =>
              !conditionRefsExist(condition, characterIds, locationIds, relationshipIds),
          ) ||
          choice.effects.some(
            (effect) => !effectRefsExist(effect, characterIds, relationshipIds),
          )
        ) {
          return world;
        }
      }
    }
    sceneIds.add(scene.id);
  }

  const eventIds = new Set(world.scheduled_events.map((event) => event.id));
  for (const event of packageData.scheduled_events) {
    if (!isValidScheduledEvent(event) || eventIds.has(event.id)) {
      return world;
    }
    if (
      event.conditions.some(
        (condition) =>
          !conditionRefsExist(condition, characterIds, locationIds, relationshipIds),
      ) ||
      !scheduledKindRefsExist(event, characterIds, relationshipIds) ||
      (event.kind.type === "start_dialogue" && !sceneIds.has(event.kind.scene_id))
    ) {
      return world;
    }
    eventIds.add(event.id);
  }

  return {
    ...world,
    installed_content_packages: [
      ...world.installed_content_packages,
      {
        namespace: packageData.manifest.namespace,
        package_id: packageData.manifest.package_id,
        version: packageData.manifest.version,
        dependencies: structuredClone(dependencies),
        conflicts: structuredClone(packageData.manifest.conflicts),
      },
    ].sort((left, right) =>
      left.namespace.localeCompare(right.namespace) ||
      left.package_id.localeCompare(right.package_id),
    ),
    locations: [...world.locations, ...structuredClone(packageData.locations)],
    characters: [...world.characters, ...structuredClone(packageData.characters)],
    relationships: [
      ...world.relationships,
      ...structuredClone(packageData.relationships),
    ],
    resources: [...world.resources, ...structuredClone(packageData.resources)].sort(
      (left, right) => left.resource_id.localeCompare(right.resource_id),
    ),
    dialogue_scenes: [
      ...world.dialogue_scenes,
      ...structuredClone(packageData.dialogue_scenes),
    ],
    scheduled_events: [
      ...world.scheduled_events,
      ...structuredClone(packageData.scheduled_events),
    ].sort(byScheduledEventOrder),
    event_log: [
      `内容包 ${packageData.manifest.namespace}:${packageData.manifest.package_id} 已加载。`,
      ...world.event_log,
    ],
  };
};

export const createBrowserMockEngineClient = (): EngineClient => {
  let world = createDemoWorld();
  const saves = new Map<string, SaveEnvelope>();

  return {
    async snapshot() {
      return structuredClone(world);
    },
    async dispatch(command) {
      world = applyDemoCommand(world, command);
      return structuredClone(world);
    },
    async installContentPackage(packageData) {
      world = installPackageIntoBrowserWorld(world, packageData);
      return structuredClone(world);
    },
    async planResources(root) {
      return structuredClone(planResourcesForBrowserWorld(world, root));
    },
    async inspectResources(root) {
      return structuredClone(planResourcesForBrowserWorld(world, root));
    },
    async savePreview(slotId, savedAtUnixMs) {
      return {
        schema_version: 1,
        engine_version: world.engine_version,
        saved_at_unix_ms: savedAtUnixMs,
        slot_id: slotId,
        mod_dependencies: modDependenciesForWorld(world),
        world: structuredClone(world),
      };
    },
    async saveSlot(slotId, savedAtUnixMs) {
      saves.set(slotId, {
        schema_version: 1,
        engine_version: world.engine_version,
        saved_at_unix_ms: savedAtUnixMs,
        slot_id: slotId,
        mod_dependencies: modDependenciesForWorld(world),
        world: structuredClone(world),
      });

      return {
        path: `browser-memory://${slotId}.json`,
        backup_path: null,
      };
    },
    async loadSlot(slotId) {
      const save = saves.get(slotId);
      if (save) {
        world = structuredClone(save.world);
      }

      return structuredClone(world);
    },
  };
};

export const createEngineClient = (): EngineClient =>
  isTauriRuntime() ? createTauriEngineClient() : createBrowserMockEngineClient();
