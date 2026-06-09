import { create } from "zustand";
import { createEngineClient, type EngineClient } from "./client";
import type {
  ContentPackage,
  EngineCommand,
  ModInstallPreflightReport,
  ModInstallReport,
  ModRegistry,
  SavePreflightReport,
  SaveRecoveryReport,
  SaveSlotReport,
  WorldState,
} from "../types";

interface EngineStore {
  client: EngineClient;
  world: WorldState | null;
  loading: boolean;
  error: string | null;
  lastSave: SaveSlotReport | null;
  lastLoadPreflight: SavePreflightReport | null;
  lastRecovery: SaveRecoveryReport | null;
  lastModPackagePreflight: ModInstallPreflightReport | null;
  lastModInstall: ModInstallReport | null;
  load: () => Promise<void>;
  dispatch: (command: EngineCommand) => Promise<void>;
  installContentPackage: (packageData: ContentPackage) => Promise<void>;
  preflightModPackageInstall: (
    packageRoot: string,
    installRoot: string,
  ) => Promise<void>;
  installModPackage: (packageRoot: string, installRoot: string) => Promise<void>;
  saveSlot: (slotId: string) => Promise<void>;
  preflightLoadSlot: (slotId: string) => Promise<void>;
  loadSlot: (slotId: string) => Promise<void>;
  recoverSlot: (slotId: string) => Promise<void>;
}

const contentRegistryForWorld = (world: WorldState | null): ModRegistry => ({
  enabled:
    world?.installed_content_packages.map((packageInfo) => ({
      namespace: packageInfo.package_id,
      version: packageInfo.version,
      conflicts: [...packageInfo.conflicts],
    })) ?? [],
});

const formatContentPreflightIssues = (
  issues: Awaited<
    ReturnType<EngineClient["preflightContentPackageInstall"]>
  >["issues"],
) => issues.map((issue) => issue.message).join("; ");

const DEFAULT_MOD_ROOT = "examples/mods";

export const useEngine = create<EngineStore>((set, get) => ({
  client: createEngineClient(),
  world: null,
  loading: false,
  error: null,
  lastSave: null,
  lastLoadPreflight: null,
  lastRecovery: null,
  lastModPackagePreflight: null,
  lastModInstall: null,
  async load() {
    set({ loading: true, error: null });
    try {
      const world = await get().client.snapshot();
      set({ world, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },
  async dispatch(command) {
    set({ loading: true, error: null });
    try {
      const world = await get().client.dispatch(command);
      set({ world, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },
  async installContentPackage(packageData) {
    set({ loading: true, error: null });
    try {
      const registry = contentRegistryForWorld(get().world);
      const preflight = await get().client.preflightContentPackageInstall(
        packageData,
        registry,
      );
      if (!preflight.ready) {
        throw new Error(formatContentPreflightIssues(preflight.issues));
      }

      const world = await get().client.installContentPackage(packageData, registry);
      set({ world, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },
  async preflightModPackageInstall(packageRoot, installRoot) {
    set({ loading: true, error: null });
    try {
      const lastModPackagePreflight = await get().client.preflightModPackageInstall(
        packageRoot,
        installRoot,
        get().world?.engine_version,
        [],
      );
      set({ lastModPackagePreflight, lastModInstall: null, loading: false });
    } catch (error) {
      set({ error: String(error), lastModPackagePreflight: null, loading: false });
    }
  },
  async installModPackage(packageRoot, installRoot) {
    set({ loading: true, error: null });
    try {
      const existingPreflight = get().lastModPackagePreflight;
      const preflight =
        existingPreflight?.source_root === packageRoot &&
        existingPreflight.install_root === installRoot
          ? existingPreflight
          : await get().client.preflightModPackageInstall(
              packageRoot,
              installRoot,
              get().world?.engine_version,
              [],
            );

      if (!preflight.ready) {
        throw new Error(preflight.issues.map((issue) => issue.message).join("; "));
      }

      const lastModInstall = await get().client.installModPackage(
        packageRoot,
        installRoot,
        get().world?.engine_version,
        [],
      );
      set({ lastModPackagePreflight: preflight, lastModInstall, loading: false });
    } catch (error) {
      set({ error: String(error), lastModInstall: null, loading: false });
    }
  },
  async saveSlot(slotId) {
    set({ loading: true, error: null });
    try {
      const lastSave = await get().client.saveSlot(slotId, Date.now());
      set({ lastSave, lastLoadPreflight: null, lastRecovery: null, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },
  async preflightLoadSlot(slotId) {
    set({ loading: true, error: null });
    try {
      const lastLoadPreflight = await get().client.preflightLoadSlot(
        slotId,
        DEFAULT_MOD_ROOT,
        [],
        get().world?.engine_version,
      );
      set({ lastLoadPreflight, loading: false });
    } catch (error) {
      set({ error: String(error), lastLoadPreflight: null, loading: false });
    }
  },
  async loadSlot(slotId) {
    set({ loading: true, error: null });
    try {
      const world = await get().client.loadSlot(slotId);
      set({ world, lastLoadPreflight: null, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },
  async recoverSlot(slotId) {
    set({ loading: true, error: null });
    try {
      const lastRecovery = await get().client.recoverSlot(slotId, Date.now());
      set({ world: lastRecovery.save.world, lastRecovery, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },
}));
