import { create } from "zustand";
import { createEngineClient, type EngineClient } from "./client";
import type {
  ContentPackage,
  EngineCommand,
  ModDiscoveryReport,
  ModEnablement,
  ModEnablementPlanReport,
  ModInstallPreflightReport,
  ModInstallReport,
  ModRegistry,
  ModUninstallReport,
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
  lastModUninstall: ModUninstallReport | null;
  lastInstalledMods: ModDiscoveryReport | null;
  modEnablement: ModEnablement[];
  lastModEnablementPlan: ModEnablementPlanReport | null;
  load: () => Promise<void>;
  dispatch: (command: EngineCommand) => Promise<void>;
  installContentPackage: (packageData: ContentPackage) => Promise<void>;
  preflightModPackageInstall: (
    packageRoot: string,
    installRoot: string,
  ) => Promise<void>;
  installModPackage: (packageRoot: string, installRoot: string) => Promise<void>;
  uninstallInstalledMod: (installRoot: string, namespace: string) => Promise<void>;
  refreshInstalledMods: (installRoot: string) => Promise<void>;
  planInstalledMods: (installRoot: string) => Promise<void>;
  setModEnabled: (
    namespace: string,
    enabled: boolean,
    installRoot: string,
  ) => Promise<void>;
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
export const DEFAULT_MOD_INSTALL_ROOT = "mods/installed";

const normalizeModEnablement = (enablement: ModEnablement[]) => {
  const byNamespace = new Map<string, ModEnablement>();
  for (const entry of enablement) {
    const namespace = entry.namespace.trim();
    if (namespace) {
      byNamespace.set(namespace, { namespace, enabled: entry.enabled });
    }
  }

  return [...byNamespace.values()].sort((left, right) =>
    left.namespace.localeCompare(right.namespace),
  );
};

const planEnablementForDiscoveredMods = (
  enablement: ModEnablement[],
  installedMods: ModDiscoveryReport,
) => {
  const discoveredNamespaces = new Set(
    installedMods.discovered.map((entry) => entry.manifest.namespace),
  );

  return normalizeModEnablement(enablement).filter((entry) =>
    discoveredNamespaces.has(entry.namespace),
  );
};

const planInstalledModsForState = async (
  client: EngineClient,
  installRoot: string,
  world: WorldState | null,
  modEnablement: ModEnablement[],
  existingInstalledMods: ModDiscoveryReport | null,
) => {
  const installedMods =
    existingInstalledMods?.root_path === installRoot
      ? existingInstalledMods
      : await client.discoverMods(installRoot, world?.engine_version, []);
  const lastModEnablementPlan = await client.planEnabledMods(
    installedMods.discovered.map((entry) => entry.manifest),
    planEnablementForDiscoveredMods(modEnablement, installedMods),
    world?.engine_version,
    [],
  );

  return { lastInstalledMods: installedMods, lastModEnablementPlan };
};

const loadModEnablementForRoot = async (
  client: EngineClient,
  installRoot: string,
) => {
  try {
    return {
      modEnablement: normalizeModEnablement(
        await client.loadModEnablement(installRoot),
      ),
      settingsError: null,
    };
  } catch (error) {
    return {
      modEnablement: [],
      settingsError: String(error),
    };
  }
};

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
  lastModUninstall: null,
  lastInstalledMods: null,
  modEnablement: [],
  lastModEnablementPlan: null,
  async load() {
    set({ loading: true, error: null });
    try {
      const client = get().client;
      const world = await client.snapshot();
      const { modEnablement, settingsError } = await loadModEnablementForRoot(
        client,
        DEFAULT_MOD_INSTALL_ROOT,
      );
      set({
        world,
        modEnablement,
        error: settingsError,
        loading: false,
      });
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
      set({
        lastModPackagePreflight,
        lastModInstall: null,
        lastModUninstall: null,
        loading: false,
      });
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
      const lastInstalledMods = await get().client.discoverMods(
        installRoot,
        get().world?.engine_version,
        [],
      );
      const { modEnablement, settingsError } = await loadModEnablementForRoot(
        get().client,
        installRoot,
      );
      const { lastModEnablementPlan } = await planInstalledModsForState(
        get().client,
        installRoot,
        get().world,
        modEnablement,
        lastInstalledMods,
      );
      set({
        lastModPackagePreflight: preflight,
        lastModInstall,
        lastModUninstall: null,
        lastInstalledMods,
        modEnablement,
        lastModEnablementPlan,
        error: settingsError,
        loading: false,
      });
    } catch (error) {
      set({ error: String(error), lastModInstall: null, loading: false });
    }
  },
  async uninstallInstalledMod(installRoot, namespace) {
    set({ loading: true, error: null });
    try {
      const lastModUninstall = await get().client.uninstallMod(installRoot, namespace);
      const nextModEnablement = normalizeModEnablement(
        get().modEnablement.filter((entry) => entry.namespace !== namespace),
      );
      let savedModEnablement = nextModEnablement;
      let settingsError: string | null = null;
      try {
        savedModEnablement = normalizeModEnablement(
          await get().client.saveModEnablement(installRoot, nextModEnablement),
        );
      } catch (error) {
        settingsError = String(error);
      }
      const { lastInstalledMods, lastModEnablementPlan } =
        await planInstalledModsForState(
          get().client,
          installRoot,
          get().world,
          savedModEnablement,
          null,
        );
      set({
        lastModInstall: null,
        lastModUninstall,
        lastInstalledMods,
        modEnablement: savedModEnablement,
        lastModEnablementPlan,
        error: settingsError,
        loading: false,
      });
    } catch (error) {
      set({ error: String(error), lastModUninstall: null, loading: false });
    }
  },
  async refreshInstalledMods(installRoot) {
    set({ loading: true, error: null });
    try {
      const { modEnablement, settingsError } = await loadModEnablementForRoot(
        get().client,
        installRoot,
      );
      const { lastInstalledMods, lastModEnablementPlan } =
        await planInstalledModsForState(
          get().client,
          installRoot,
          get().world,
          modEnablement,
          null,
        );
      set({
        lastInstalledMods,
        modEnablement,
        lastModEnablementPlan,
        error: settingsError,
        loading: false,
      });
    } catch (error) {
      set({
        error: String(error),
        lastInstalledMods: null,
        lastModEnablementPlan: null,
        loading: false,
      });
    }
  },
  async planInstalledMods(installRoot) {
    set({ loading: true, error: null });
    try {
      const { modEnablement, settingsError } = await loadModEnablementForRoot(
        get().client,
        installRoot,
      );
      const { lastInstalledMods, lastModEnablementPlan } =
        await planInstalledModsForState(
          get().client,
          installRoot,
          get().world,
          modEnablement,
          get().lastInstalledMods,
        );
      set({
        lastInstalledMods,
        modEnablement,
        lastModEnablementPlan,
        error: settingsError,
        loading: false,
      });
    } catch (error) {
      set({ error: String(error), lastModEnablementPlan: null, loading: false });
    }
  },
  async setModEnabled(namespace, enabled, installRoot) {
    set({ loading: true, error: null });
    try {
      const modEnablement = normalizeModEnablement([
        ...get().modEnablement.filter((entry) => entry.namespace !== namespace),
        { namespace, enabled },
      ]);
      const savedModEnablement = await get().client.saveModEnablement(
        installRoot,
        modEnablement,
      );
      const { lastInstalledMods, lastModEnablementPlan } =
        await planInstalledModsForState(
          get().client,
          installRoot,
          get().world,
          savedModEnablement,
          get().lastInstalledMods,
        );
      set({
        modEnablement: normalizeModEnablement(savedModEnablement),
        lastInstalledMods,
        lastModEnablementPlan,
        loading: false,
      });
    } catch (error) {
      set({ error: String(error), loading: false });
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
