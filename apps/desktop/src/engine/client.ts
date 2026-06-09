import { invoke } from "@tauri-apps/api/core";
import { applyDemoCommand, createDemoWorld } from "./demoWorld";
import type {
  ContentPackage,
  EngineCommand,
  SaveEnvelope,
  SaveSlotReport,
  WorldState,
} from "../types";

export interface EngineClient {
  snapshot(): Promise<WorldState>;
  dispatch(command: EngineCommand): Promise<WorldState>;
  installContentPackage(packageData: ContentPackage): Promise<WorldState>;
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

const installPackageIntoBrowserWorld = (
  world: WorldState,
  packageData: ContentPackage,
) => {
  if (
    packageData.manifest.schema_version !== "content-package/v0" ||
    !packageData.manifest.namespace.trim() ||
    !packageData.manifest.package_id.trim()
  ) {
    return world;
  }

  const sceneIds = new Set(world.dialogue_scenes.map((scene) => scene.id));
  for (const scene of packageData.dialogue_scenes) {
    if (!scene.id.trim() || sceneIds.has(scene.id)) {
      return world;
    }
    sceneIds.add(scene.id);
  }

  const eventIds = new Set(world.scheduled_events.map((event) => event.id));
  for (const event of packageData.scheduled_events) {
    if (!isValidScheduledEvent(event) || eventIds.has(event.id)) {
      return world;
    }
    if (event.kind.type === "start_dialogue" && !sceneIds.has(event.kind.scene_id)) {
      return world;
    }
    eventIds.add(event.id);
  }

  return {
    ...world,
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
    async savePreview(slotId, savedAtUnixMs) {
      return {
        schema_version: 1,
        engine_version: world.engine_version,
        saved_at_unix_ms: savedAtUnixMs,
        slot_id: slotId,
        mod_dependencies: [],
        world: structuredClone(world),
      };
    },
    async saveSlot(slotId, savedAtUnixMs) {
      saves.set(slotId, {
        schema_version: 1,
        engine_version: world.engine_version,
        saved_at_unix_ms: savedAtUnixMs,
        slot_id: slotId,
        mod_dependencies: [],
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
