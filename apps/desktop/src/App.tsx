import { useEffect, useState } from "react";
import { TraditionalView } from "./components/TraditionalView";
import {
  SAVE_SLOTS,
  type SaveSlotId,
  type ShellServices,
} from "./components/traditional/shellTypes";
import { createSampleContentPackage } from "./engine/sampleContentPackage";
import { DEFAULT_MOD_INSTALL_ROOT, useEngine } from "./engine/useEngine";

const MOD_PACKAGE_ROOT = "packages/example.minimal_character-0.1.0";

export const App = () => {
  const {
    dispatch,
    error,
    installContentPackage,
    installModPackage,
    lastInstalledMods,
    lastLoadPreflight,
    lastModEnablementPlan,
    lastModInstall,
    lastModPackagePreflight,
    lastModUninstall,
    lastModUninstallPlan,
    lastRecovery,
    lastSave,
    load,
    loadSlot,
    loading,
    modEnablement,
    planModUninstall,
    preflightLoadSlot,
    preflightModPackageInstall,
    recoverSlot,
    refreshInstalledMods,
    saveSlot,
    setModEnabled,
    uninstallInstalledMod,
    world,
  } = useEngine();
  const [selectedSlotId, setSelectedSlotId] = useState<SaveSlotId>(SAVE_SLOTS[0]);

  useEffect(() => {
    void load();
  }, [load]);

  if (!world) {
    return (
      <main className="app-shell">
        <div className="boot-panel">正在启动 ERAtw-NEXT engine mock...</div>
      </main>
    );
  }

  const services: ShellServices = {
    dispatch,
    error,
    installModPackage,
    installSamplePackage: () => installContentPackage(createSampleContentPackage()),
    lastInstalledMods,
    lastLoadPreflight,
    lastModEnablementPlan,
    lastModInstall,
    lastModPackagePreflight,
    lastModUninstall,
    lastModUninstallPlan,
    lastRecovery,
    lastSave,
    loadNewWorld: load,
    loadSlot,
    loading,
    modEnablement,
    modInstallRoot: DEFAULT_MOD_INSTALL_ROOT,
    modPackageRoot: MOD_PACKAGE_ROOT,
    planModUninstall,
    preflightLoadSlot,
    preflightModPackageInstall,
    recoverSlot,
    refreshInstalledMods,
    saveSlot,
    selectedSlotId,
    setModEnabled,
    setSelectedSlotId,
    uninstallInstalledMod,
  };

  return <TraditionalView services={services} world={world} />;
};
