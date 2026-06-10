import { beforeEach, describe, expect, it, vi } from "vitest";
import { createSampleContentPackage } from "./sampleContentPackage";
import { createDemoWorld } from "./demoWorld";
import { useEngine } from "./useEngine";
import type { EngineClient } from "./client";
import type {
  ModRegistry,
  SaveEnvelope,
  SavePreflightReport,
  SaveRecoveryReport,
  WorldState,
} from "../types";

const createMockClient = (
  world: WorldState,
  calls: {
    preflightRegistries: ModRegistry[];
    installRegistries: ModRegistry[];
  },
): EngineClient => {
  const recoveredWorld = structuredClone(world);
  recoveredWorld.clock.minute = 15;
  const recoveredSave: SaveEnvelope = {
    schema_version: 1,
    engine_version: recoveredWorld.engine_version,
    saved_at_unix_ms: 100,
    slot_id: "slot_1",
    mod_dependencies: [],
    replay_log: {
      schema_version: 1,
      engine_version: recoveredWorld.engine_version,
      initial_random: recoveredWorld.command_log_initial_random ?? recoveredWorld.random,
      commands: recoveredWorld.command_log,
    },
    world: recoveredWorld,
  };
  const recoveryReport: SaveRecoveryReport = {
    path: "memory://slot_1.json",
    recovered_from: "memory://slot_1.json.1.bak",
    failed_primary_backup_path: "memory://slot_1.json.2.bak",
    save: recoveredSave,
  };
  const preflightReport: SavePreflightReport = {
    slot_id: "slot_1",
    path: "memory://slot_1.json",
    ready: true,
    registry: { enabled: [] },
    discovery: {
      root_path: "examples/mods",
      discovered: [],
      errors: [],
    },
    validation: {
      missing_required_mods: [],
      incompatible_schema: null,
      engine_version_mismatch: false,
    },
    save: recoveredSave,
  };

  return {
    snapshot: vi.fn(async () => structuredClone(world)),
    dispatch: vi.fn(async () => structuredClone(world)),
    installContentPackage: vi.fn(async (_packageData, registry = null) => {
      if (registry) {
        calls.installRegistries.push(structuredClone(registry));
      }
      return structuredClone(world);
    }),
    preflightContentPackageInstall: vi.fn(async (_packageData, registry = null) => {
      if (registry) {
        calls.preflightRegistries.push(structuredClone(registry));
      }
      return {
        package_id: _packageData.manifest.package_id,
        namespace: _packageData.manifest.namespace,
        version: _packageData.manifest.version,
        ready: true,
        validation: { issues: [] },
        issues: [],
      };
    }),
    planResources: vi.fn(async () => ({ root: "", low_spec: false, entries: [] })),
    inspectResources: vi.fn(async () => ({ root: "", low_spec: false, entries: [] })),
    preflightResources: vi.fn(async () => ({
      root: "",
      low_spec: false,
      ready: true,
      resolution: { root: "", low_spec: false, entries: [] },
      issues: [],
    })),
    auditResourcePublication: vi.fn(async () => ({
      root: "",
      low_spec: false,
      ready: true,
      error_count: 0,
      warning_count: 0,
      resolution: { root: "", low_spec: false, entries: [] },
      issues: [],
    })),
    cacheResources: vi.fn(async () => ({
      root: "",
      low_spec: false,
      ready: true,
      cached_count: 0,
      skipped_count: 0,
      failed_count: 0,
      resolution: { root: "", low_spec: false, entries: [] },
      entries: [],
    })),
    cleanResourceCache: vi.fn(async () => ({
      root: "",
      low_spec: false,
      ready: true,
      cache_root: "",
      kept_count: 0,
      removed_count: 0,
      skipped_count: 0,
      failed_count: 0,
      bytes_removed: 0,
      resolution: { root: "", low_spec: false, entries: [] },
      entries: [],
    })),
    discoverMods: vi.fn(async (root) => ({
      root_path: root,
      discovered: [],
      errors: [],
    })),
    planModInstall: vi.fn(async () => {
      throw new Error("not used");
    }),
    installMod: vi.fn(async () => {
      throw new Error("not used");
    }),
    preflightModPackageInstall: vi.fn(async () => {
      throw new Error("not used");
    }),
    installModPackage: vi.fn(async () => {
      throw new Error("not used");
    }),
    planModUninstall: vi.fn(async () => {
      throw new Error("not used");
    }),
    uninstallMod: vi.fn(async () => {
      throw new Error("not used");
    }),
    planEnabledMods: vi.fn(async () => ({ enabled: [], disabled: [] })),
    loadModEnablement: vi.fn(async () => []),
    saveModEnablement: vi.fn(async (_installRoot, enablement) =>
      structuredClone(enablement),
    ),
    savePreview: vi.fn(async () => {
      throw new Error("not used");
    }),
    saveSlot: vi.fn(async () => {
      throw new Error("not used");
    }),
    recoverSlot: vi.fn(async () => structuredClone(recoveryReport)),
    preflightLoadSlot: vi.fn(async () => structuredClone(preflightReport)),
    loadSlot: vi.fn(async () => structuredClone(world)),
  };
};

describe("useEngine", () => {
  beforeEach(() => {
    useEngine.setState({
      world: null,
      loading: false,
      error: null,
      lastSave: null,
      lastLoadPreflight: null,
      lastRecovery: null,
      lastModPackagePreflight: null,
      lastModInstall: null,
      lastModUninstallPlan: null,
      lastModUninstall: null,
      lastInstalledMods: null,
      modEnablement: [],
      lastModEnablementPlan: null,
    });
  });

  it("preflights and installs content packages with the current content registry", async () => {
    const world = createDemoWorld();
    world.installed_content_packages.push({
      namespace: "sample",
      package_id: "sample.base",
      version: "0.1.0",
      dependencies: [],
      conflicts: ["sample.blocked"],
    });
    const calls = {
      preflightRegistries: [] as ModRegistry[],
      installRegistries: [] as ModRegistry[],
    };
    const client = createMockClient(world, calls);
    useEngine.setState({ client, world });

    await useEngine.getState().installContentPackage(createSampleContentPackage());

    const expectedRegistry = {
      enabled: [
        {
          namespace: "sample.base",
          version: "0.1.0",
          conflicts: ["sample.blocked"],
        },
      ],
    };
    expect(calls.preflightRegistries).toEqual([expectedRegistry]);
    expect(calls.installRegistries).toEqual([expectedRegistry]);
    const preflightOrder = vi.mocked(client.preflightContentPackageInstall).mock
      .invocationCallOrder[0];
    const installOrder = vi.mocked(client.installContentPackage).mock
      .invocationCallOrder[0];
    expect(preflightOrder).toBeLessThan(installOrder);
  });

  it("loads persisted mod enablement with the initial world snapshot", async () => {
    const world = createDemoWorld();
    const calls = {
      preflightRegistries: [] as ModRegistry[],
      installRegistries: [] as ModRegistry[],
    };
    const client = createMockClient(world, calls);
    vi.mocked(client.loadModEnablement).mockResolvedValue([
      { namespace: "example.minimal_character", enabled: false },
    ]);
    useEngine.setState({ client });

    await useEngine.getState().load();

    expect(client.loadModEnablement).toHaveBeenCalledWith("mods/installed");
    expect(useEngine.getState().world?.engine_version).toBe("0.1.0-m0");
    expect(useEngine.getState().modEnablement).toEqual([
      { namespace: "example.minimal_character", enabled: false },
    ]);
  });

  it("keeps the initial world snapshot when mod enablement settings fail", async () => {
    const world = createDemoWorld();
    const calls = {
      preflightRegistries: [] as ModRegistry[],
      installRegistries: [] as ModRegistry[],
    };
    const client = createMockClient(world, calls);
    vi.mocked(client.loadModEnablement).mockRejectedValue(
      new Error("settings json is invalid"),
    );
    useEngine.setState({ client });

    await useEngine.getState().load();

    expect(useEngine.getState().world?.engine_version).toBe("0.1.0-m0");
    expect(useEngine.getState().modEnablement).toEqual([]);
    expect(useEngine.getState().error).toContain("settings json is invalid");
  });

  it("recovers a save slot and updates the active world", async () => {
    const world = createDemoWorld();
    const calls = {
      preflightRegistries: [] as ModRegistry[],
      installRegistries: [] as ModRegistry[],
    };
    const client = createMockClient(world, calls);
    useEngine.setState({ client, world });

    await useEngine.getState().recoverSlot("slot_1");

    expect(client.recoverSlot).toHaveBeenCalledWith("slot_1", expect.any(Number));
    expect(useEngine.getState().world?.clock.minute).toBe(15);
    expect(useEngine.getState().lastRecovery?.recovered_from).toBe(
      "memory://slot_1.json.1.bak",
    );
  });

  it("preflights a save slot before loading", async () => {
    const world = createDemoWorld();
    const calls = {
      preflightRegistries: [] as ModRegistry[],
      installRegistries: [] as ModRegistry[],
    };
    const client = createMockClient(world, calls);
    useEngine.setState({ client, world });

    await useEngine.getState().preflightLoadSlot("slot_1");

    expect(client.preflightLoadSlot).toHaveBeenCalledWith(
      "slot_1",
      "examples/mods",
      [],
      "0.1.0-m0",
    );
    expect(useEngine.getState().lastLoadPreflight?.ready).toBe(true);
    expect(useEngine.getState().world).toEqual(world);
  });

  it("preflights a mod package with the active engine version", async () => {
    const world = createDemoWorld();
    const calls = {
      preflightRegistries: [] as ModRegistry[],
      installRegistries: [] as ModRegistry[],
    };
    const client = createMockClient(world, calls);
    vi.mocked(client.preflightModPackageInstall).mockResolvedValue({
      source_root: "packages/example.minimal_character-0.1.0",
      content_root: "packages/example.minimal_character-0.1.0/content",
      install_root: "mods/installed",
      target_root: "mods/installed/example.minimal_character",
      staging_root: "mods/installed/.installing-example.minimal_character",
      manifest: null,
      ready: true,
      issues: [],
    });
    useEngine.setState({ client, world });

    await useEngine
      .getState()
      .preflightModPackageInstall(
        "packages/example.minimal_character-0.1.0",
        "mods/installed",
      );

    expect(client.preflightModPackageInstall).toHaveBeenCalledWith(
      "packages/example.minimal_character-0.1.0",
      "mods/installed",
      "0.1.0-m0",
      [],
    );
    expect(useEngine.getState().lastModPackagePreflight?.ready).toBe(true);
  });

  it("installs a mod package after a ready preflight", async () => {
    const world = createDemoWorld();
    const calls = {
      preflightRegistries: [] as ModRegistry[],
      installRegistries: [] as ModRegistry[],
    };
    const client = createMockClient(world, calls);
    vi.mocked(client.preflightModPackageInstall).mockResolvedValue({
      source_root: "packages/example.minimal_character-0.1.0",
      content_root: "packages/example.minimal_character-0.1.0/content",
      install_root: "mods/installed",
      target_root: "mods/installed/example.minimal_character",
      staging_root: "mods/installed/.installing-example.minimal_character",
      manifest: null,
      ready: true,
      issues: [],
    });
    vi.mocked(client.installModPackage).mockResolvedValue({
      target_root: "mods/installed/example.minimal_character",
      manifest: {
        namespace: "example.minimal_character",
        name: "example.minimal_character",
        version: "0.1.0",
        engine_version: "0.1.0-m0",
        load_order: 0,
        dependencies: [],
        conflicts: [],
        capabilities: ["content"],
        resources: [],
      },
      actions: [],
    });
    useEngine.setState({ client, world });

    await useEngine
      .getState()
      .installModPackage(
        "packages/example.minimal_character-0.1.0",
        "mods/installed",
      );

    expect(client.preflightModPackageInstall).toHaveBeenCalledWith(
      "packages/example.minimal_character-0.1.0",
      "mods/installed",
      "0.1.0-m0",
      [],
    );
    expect(client.installModPackage).toHaveBeenCalledWith(
      "packages/example.minimal_character-0.1.0",
      "mods/installed",
      "0.1.0-m0",
      [],
    );
    expect(client.discoverMods).toHaveBeenCalledWith(
      "mods/installed",
      "0.1.0-m0",
      [],
    );
    expect(useEngine.getState().lastModInstall?.target_root).toBe(
      "mods/installed/example.minimal_character",
    );
    expect(useEngine.getState().lastInstalledMods?.root_path).toBe("mods/installed");
  });

  it("refreshes installed mods from the selected install root", async () => {
    const world = createDemoWorld();
    const calls = {
      preflightRegistries: [] as ModRegistry[],
      installRegistries: [] as ModRegistry[],
    };
    const client = createMockClient(world, calls);
    vi.mocked(client.discoverMods).mockResolvedValue({
      root_path: "mods/installed",
      discovered: [
        {
          root_path: "mods/installed/example.minimal_character",
          manifest_path: "mods/installed/example.minimal_character/manifest.json",
          manifest: {
            namespace: "example.minimal_character",
            name: "最小角色 Mod",
            version: "0.1.0",
            engine_version: "0.1.0-m0",
            load_order: 0,
            dependencies: [],
            conflicts: [],
            capabilities: ["content"],
            resources: [],
          },
        },
      ],
      errors: [],
    });
    vi.mocked(client.planEnabledMods).mockResolvedValue({
      enabled: [
        {
          namespace: "example.minimal_character",
          name: "最小角色 Mod",
          version: "0.1.0",
          engine_version: "0.1.0-m0",
          load_order: 0,
          dependencies: [],
          conflicts: [],
          capabilities: ["content"],
          resources: [],
        },
      ],
      disabled: [],
    });
    useEngine.setState({ client, world });

    await useEngine.getState().refreshInstalledMods("mods/installed");

    expect(client.loadModEnablement).toHaveBeenCalledWith("mods/installed");
    expect(client.discoverMods).toHaveBeenCalledWith(
      "mods/installed",
      "0.1.0-m0",
      [],
    );
    expect(useEngine.getState().lastInstalledMods?.discovered[0].root_path).toBe(
      "mods/installed/example.minimal_character",
    );
    expect(client.planEnabledMods).toHaveBeenCalledWith(
      [
        expect.objectContaining({
          namespace: "example.minimal_character",
        }),
      ],
      [],
      "0.1.0-m0",
      [],
    );
    expect(useEngine.getState().lastModEnablementPlan?.enabled[0].namespace).toBe(
      "example.minimal_character",
    );
  });

  it("updates mod enablement and replans installed mods", async () => {
    const world = createDemoWorld();
    const calls = {
      preflightRegistries: [] as ModRegistry[],
      installRegistries: [] as ModRegistry[],
    };
    const client = createMockClient(world, calls);
    vi.mocked(client.discoverMods).mockResolvedValue({
      root_path: "mods/installed",
      discovered: [
        {
          root_path: "mods/installed/example.minimal_character",
          manifest_path: "mods/installed/example.minimal_character/manifest.json",
          manifest: {
            namespace: "example.minimal_character",
            name: "最小角色 Mod",
            version: "0.1.0",
            engine_version: "0.1.0-m0",
            load_order: 0,
            dependencies: [],
            conflicts: [],
            capabilities: ["content"],
            resources: [],
          },
        },
      ],
      errors: [],
    });
    vi.mocked(client.planEnabledMods).mockResolvedValue({
      enabled: [],
      disabled: [
        {
          manifest: {
            namespace: "example.minimal_character",
            name: "最小角色 Mod",
            version: "0.1.0",
            engine_version: "0.1.0-m0",
            load_order: 0,
            dependencies: [],
            conflicts: [],
            capabilities: ["content"],
            resources: [],
          },
          reason: "user_disabled",
        },
      ],
    });
    useEngine.setState({ client, world });

    await useEngine
      .getState()
      .setModEnabled("example.minimal_character", false, "mods/installed");

    expect(client.saveModEnablement).toHaveBeenCalledWith("mods/installed", [
      { namespace: "example.minimal_character", enabled: false },
    ]);
    expect(useEngine.getState().modEnablement).toEqual([
      { namespace: "example.minimal_character", enabled: false },
    ]);
    expect(client.planEnabledMods).toHaveBeenCalledWith(
      [
        expect.objectContaining({
          namespace: "example.minimal_character",
        }),
      ],
      [{ namespace: "example.minimal_character", enabled: false }],
      "0.1.0-m0",
      [],
    );
    expect(useEngine.getState().lastModEnablementPlan?.disabled[0].reason).toBe(
      "user_disabled",
    );
  });

  it("plans a mod uninstall before execution", async () => {
    const world = createDemoWorld();
    const calls = {
      preflightRegistries: [] as ModRegistry[],
      installRegistries: [] as ModRegistry[],
    };
    const client = createMockClient(world, calls);
    vi.mocked(client.planModUninstall).mockResolvedValue({
      install_root: "mods/installed",
      target_root: "mods/installed/example.minimal_character",
      staging_root: "mods/installed/.uninstalling-example.minimal_character",
      namespace: "example.minimal_character",
      actions: [
        {
          kind: "move_directory",
          path: null,
          from: "mods/installed/example.minimal_character",
          to: "mods/installed/.uninstalling-example.minimal_character",
        },
        {
          kind: "delete_directory",
          path: "mods/installed/.uninstalling-example.minimal_character",
          from: null,
          to: null,
        },
      ],
    });
    useEngine.setState({ client, world });

    await useEngine
      .getState()
      .planModUninstall("mods/installed", "example.minimal_character");

    expect(client.planModUninstall).toHaveBeenCalledWith(
      "mods/installed",
      "example.minimal_character",
    );
    expect(client.uninstallMod).not.toHaveBeenCalled();
    expect(useEngine.getState().lastModUninstallPlan?.target_root).toBe(
      "mods/installed/example.minimal_character",
    );
  });

  it("uninstalls a mod, clears its enablement, and refreshes the plan", async () => {
    const world = createDemoWorld();
    const calls = {
      preflightRegistries: [] as ModRegistry[],
      installRegistries: [] as ModRegistry[],
    };
    const client = createMockClient(world, calls);
    vi.mocked(client.uninstallMod).mockResolvedValue({
      namespace: "example.minimal_character",
      target_root: "mods/installed/example.minimal_character",
      actions: [
        {
          kind: "move_directory",
          path: null,
          from: "mods/installed/example.minimal_character",
          to: "mods/installed/.uninstalling-example.minimal_character",
        },
        {
          kind: "delete_directory",
          path: "mods/installed/.uninstalling-example.minimal_character",
          from: null,
          to: null,
        },
      ],
    });
    vi.mocked(client.discoverMods).mockResolvedValue({
      root_path: "mods/installed",
      discovered: [],
      errors: [],
    });
    useEngine.setState({
      client,
      world,
      modEnablement: [
        { namespace: "example.minimal_character", enabled: false },
      ],
    });

    await useEngine
      .getState()
      .uninstallInstalledMod("mods/installed", "example.minimal_character");

    expect(client.uninstallMod).toHaveBeenCalledWith(
      "mods/installed",
      "example.minimal_character",
    );
    expect(client.saveModEnablement).toHaveBeenCalledWith("mods/installed", []);
    expect(client.discoverMods).toHaveBeenCalledWith(
      "mods/installed",
      "0.1.0-m0",
      [],
    );
    expect(client.planEnabledMods).toHaveBeenCalledWith(
      [],
      [],
      "0.1.0-m0",
      [],
    );
    expect(useEngine.getState().lastModUninstall?.namespace).toBe(
      "example.minimal_character",
    );
    expect(useEngine.getState().lastInstalledMods?.discovered).toEqual([]);
    expect(useEngine.getState().modEnablement).toEqual([]);
  });
});
