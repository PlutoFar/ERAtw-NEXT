import { beforeEach, describe, expect, it, vi } from "vitest";
import { createSampleContentPackage } from "./sampleContentPackage";
import { createDemoWorld } from "./demoWorld";
import { useEngine } from "./useEngine";
import type { EngineClient } from "./client";
import type { ModRegistry, SaveEnvelope, SaveRecoveryReport, WorldState } from "../types";

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
    world: recoveredWorld,
  };
  const recoveryReport: SaveRecoveryReport = {
    path: "memory://slot_1.json",
    recovered_from: "memory://slot_1.json.1.bak",
    failed_primary_backup_path: "memory://slot_1.json.2.bak",
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
    planResources: vi.fn(async () => ({ root: "", entries: [] })),
    inspectResources: vi.fn(async () => ({ root: "", entries: [] })),
    preflightResources: vi.fn(async () => ({
      root: "",
      ready: true,
      resolution: { root: "", entries: [] },
      issues: [],
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
    planModUninstall: vi.fn(async () => {
      throw new Error("not used");
    }),
    uninstallMod: vi.fn(async () => {
      throw new Error("not used");
    }),
    planEnabledMods: vi.fn(async () => ({ enabled: [], disabled: [] })),
    savePreview: vi.fn(async () => {
      throw new Error("not used");
    }),
    saveSlot: vi.fn(async () => {
      throw new Error("not used");
    }),
    recoverSlot: vi.fn(async () => structuredClone(recoveryReport)),
    preflightLoadSlot: vi.fn(async () => {
      throw new Error("not used");
    }),
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
      lastRecovery: null,
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
});
