import type {
  EngineCommand,
  ModDiscoveryReport,
  ModEnablement,
  ModEnablementPlanReport,
  ModInstallPreflightReport,
  ModInstallReport,
  ModUninstallPlanReport,
  ModUninstallReport,
  SavePreflightReport,
  SaveRecoveryReport,
  SaveSlotReport,
  WorldState,
} from "../../types";

export const SAVE_SLOTS = ["slot_1", "slot_2", "slot_3"] as const;

export type SaveSlotId = (typeof SAVE_SLOTS)[number];

export interface ShellServices {
  dispatch: (command: EngineCommand) => void | Promise<void>;
  error: string | null;
  installModPackage: (packageRoot: string, installRoot: string) => void | Promise<void>;
  installSamplePackage: () => void | Promise<void>;
  lastInstalledMods: ModDiscoveryReport | null;
  lastLoadPreflight: SavePreflightReport | null;
  lastModEnablementPlan: ModEnablementPlanReport | null;
  lastModInstall: ModInstallReport | null;
  lastModPackagePreflight: ModInstallPreflightReport | null;
  lastModUninstall: ModUninstallReport | null;
  lastModUninstallPlan: ModUninstallPlanReport | null;
  lastRecovery: SaveRecoveryReport | null;
  lastSave: SaveSlotReport | null;
  loadNewWorld: () => void | Promise<void>;
  loadSlot: (slotId: SaveSlotId) => void | Promise<void>;
  loading: boolean;
  modEnablement: ModEnablement[];
  modInstallRoot: string;
  modPackageRoot: string;
  planModUninstall: (installRoot: string, namespace: string) => void | Promise<void>;
  preflightLoadSlot: (slotId: SaveSlotId) => void | Promise<void>;
  preflightModPackageInstall: (
    packageRoot: string,
    installRoot: string,
  ) => void | Promise<void>;
  recoverSlot: (slotId: SaveSlotId) => void | Promise<void>;
  refreshInstalledMods: (installRoot: string) => void | Promise<void>;
  saveSlot: (slotId: SaveSlotId) => void | Promise<void>;
  selectedSlotId: SaveSlotId;
  setModEnabled: (
    namespace: string,
    enabled: boolean,
    installRoot: string,
  ) => void | Promise<void>;
  setSelectedSlotId: (slotId: SaveSlotId) => void;
  uninstallInstalledMod: (
    installRoot: string,
    namespace: string,
  ) => void | Promise<void>;
}

export interface TraditionalViewProps {
  services: ShellServices;
  world: WorldState;
}
