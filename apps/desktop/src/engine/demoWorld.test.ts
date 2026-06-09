import { describe, expect, it } from "vitest";
import { createBrowserMockEngineClient } from "./client";
import {
  applyDemoCommand,
  createDemoWorld,
  replayDemoCommandLog,
  visibleChoices,
} from "./demoWorld";
import { createSampleContentPackage } from "./sampleContentPackage";

describe("demo engine adapter", () => {
  it("creates deterministic demo state", () => {
    expect(createDemoWorld()).toEqual(createDemoWorld());
  });

  it("advances time across day boundary", () => {
    const world = applyDemoCommand(createDemoWorld(), {
      type: "advance_time",
      minutes: 17 * 60,
    });

    expect(world.clock.day).toBe(2);
    expect(world.clock.hour).toBe(1);
    expect(world.clock.minute).toBe(0);
  });

  it("triggers scheduled events deterministically", () => {
    const first = applyDemoCommand(createDemoWorld(), {
      type: "advance_time",
      minutes: 60,
    });
    const second = applyDemoCommand(createDemoWorld(), {
      type: "advance_time",
      minutes: 60,
    });

    expect(first).toEqual(second);
    expect(first.clock.weather).toBe("cloudy");
    expect(first.scheduled_events).toHaveLength(0);
    expect(first.characters[0].state.energy).toBe(77);
    expect(first.characters[0].state.mood).toBe(15);
  });

  it("keeps conditional scheduled events pending until conditions pass", () => {
    const scheduled = applyDemoCommand(createDemoWorld(), {
      type: "schedule_event",
      event: {
        id: "trust_dialogue",
        due: { day: 1, hour: 8, minute: 10 },
        priority: 0,
        repeat: null,
        conditions: [
          {
            type: "relationship_affinity_at_least",
            source_character_id: "player",
            target_character_id: "demo_heroine",
            value: 7,
          },
        ],
        kind: { type: "start_dialogue", scene_id: "demo_morning" },
      },
    });
    const waiting = applyDemoCommand(scheduled, {
      type: "advance_time",
      minutes: 10,
    });

    expect(waiting.active_dialogue_scene_id).toBeNull();
    expect(waiting.scheduled_events.some((event) => event.id === "trust_dialogue")).toBe(
      true,
    );

    const unlocked = applyDemoCommand(waiting, {
      type: "adjust_relationship",
      source_character_id: "player",
      target_character_id: "demo_heroine",
      affinity_delta: 2,
      trust_delta: 0,
    });
    const triggered = applyDemoCommand(unlocked, {
      type: "advance_time",
      minutes: 1,
    });

    expect(triggered.active_dialogue_scene_id).toBe("demo_morning");
    expect(
      triggered.scheduled_events.some((event) => event.id === "trust_dialogue"),
    ).toBe(false);
  });

  it("orders same-time scheduled events by priority", () => {
    const base = createDemoWorld();
    base.scheduled_events = [];

    const withLow = applyDemoCommand(base, {
      type: "schedule_event",
      event: {
        id: "low_priority",
        due: { day: 1, hour: 8, minute: 10 },
        priority: 0,
        repeat: null,
        conditions: [],
        kind: {
          type: "adjust_character_state",
          character_id: "demo_heroine",
          energy_delta: 0,
          mood_delta: 1,
        },
      },
    });
    const scheduled = applyDemoCommand(withLow, {
      type: "schedule_event",
      event: {
        id: "high_priority",
        due: { day: 1, hour: 8, minute: 10 },
        priority: 10,
        repeat: null,
        conditions: [],
        kind: {
          type: "adjust_character_state",
          character_id: "demo_heroine",
          energy_delta: 0,
          mood_delta: 1,
        },
      },
    });

    expect(scheduled.scheduled_events[0].id).toBe("high_priority");

    const advanced = applyDemoCommand(scheduled, {
      type: "advance_time",
      minutes: 10,
    });
    const highIndex = advanced.event_log.findIndex((entry) =>
      entry.includes("high_priority"),
    );
    const lowIndex = advanced.event_log.findIndex((entry) =>
      entry.includes("low_priority"),
    );

    expect(highIndex).toBeGreaterThan(lowIndex);
  });

  it("cancels scheduled events transactionally", () => {
    const cancelled = applyDemoCommand(createDemoWorld(), {
      type: "cancel_event",
      event_id: "demo_clouds_at_gate",
    });

    expect(
      cancelled.scheduled_events.some((event) => event.id === "demo_clouds_at_gate"),
    ).toBe(false);
    expect(cancelled.command_log[0]).toEqual({
      type: "cancel_event",
      event_id: "demo_clouds_at_gate",
    });

    const rejected = applyDemoCommand(cancelled, {
      type: "cancel_event",
      event_id: "missing",
    });

    expect(rejected).toEqual(cancelled);
  });

  it("catches up repeating events and exhausts remaining runs", () => {
    const base = createDemoWorld();
    base.scheduled_events = [];
    const scheduled = applyDemoCommand(base, {
      type: "schedule_event",
      event: {
        id: "morning_tick",
        due: { day: 1, hour: 8, minute: 10 },
        priority: 0,
        repeat: {
          every_minutes: 10,
          remaining_runs: 2,
        },
        conditions: [],
        kind: {
          type: "adjust_character_state",
          character_id: "demo_heroine",
          energy_delta: 0,
          mood_delta: 2,
        },
      },
    });

    const advanced = applyDemoCommand(scheduled, {
      type: "advance_time",
      minutes: 30,
    });

    expect(advanced.characters[0].state.mood).toBe(14);
    expect(
      advanced.scheduled_events.some((event) => event.id === "morning_tick"),
    ).toBe(false);
    expect(
      advanced.event_log.filter((entry) => entry.includes("morning_tick")),
    ).toHaveLength(2);
  });

  it("rolls scheduled character state events deterministically", () => {
    const base = createDemoWorld();
    base.scheduled_events = [];
    const scheduled = applyDemoCommand(base, {
      type: "schedule_event",
      event: {
        id: "random_state_tick",
        due: { day: 1, hour: 8, minute: 10 },
        priority: 0,
        repeat: null,
        conditions: [],
        kind: {
          type: "roll_character_state",
          character_id: "demo_heroine",
          energy_min_delta: -3,
          energy_max_delta: 0,
          mood_min_delta: -2,
          mood_max_delta: 4,
        },
      },
    });

    const first = applyDemoCommand(scheduled, {
      type: "advance_time",
      minutes: 10,
    });
    const replayBase = createDemoWorld();
    replayBase.scheduled_events = [];
    const replayed = replayDemoCommandLog(replayBase, {
      schema_version: 1,
      engine_version: first.engine_version,
      initial_random: first.command_log_initial_random!,
      commands: first.command_log,
    });

    expect(first).toEqual(replayed);
    expect(first.random.cursor).toBe("2");
    expect(first.characters[0].state.energy).toBeGreaterThanOrEqual(77);
    expect(first.characters[0].state.energy).toBeLessThanOrEqual(80);
    expect(first.characters[0].state.mood).toBeGreaterThanOrEqual(8);
    expect(first.characters[0].state.mood).toBeLessThanOrEqual(14);
  });

  it("rejects invalid repeating events", () => {
    const world = createDemoWorld();
    const rejected = applyDemoCommand(world, {
      type: "schedule_event",
      event: {
        id: "bad_repeat",
        due: { day: 1, hour: 8, minute: 10 },
        priority: 0,
        repeat: {
          every_minutes: 0,
          remaining_runs: null,
        },
        conditions: [],
        kind: { type: "change_weather", weather: "rain" },
      },
    });

    expect(rejected).toEqual(world);
  });

  it("starts a versioned dialogue scene", () => {
    const world = applyDemoCommand(createDemoWorld(), {
      type: "start_dialogue",
      scene_id: "demo_morning",
    });

    expect(world.active_dialogue_scene_id).toBe("demo_morning");
    expect(world.active_dialogue).toHaveLength(1);
    expect(world.active_dialogue[0].choices).toHaveLength(3);
    expect(visibleChoices(world, world.active_dialogue[0])).toHaveLength(2);
  });

  it("applies dialogue choice effects", () => {
    const started = applyDemoCommand(createDemoWorld(), {
      type: "start_dialogue",
      scene_id: "demo_morning",
    });
    const world = applyDemoCommand(started, {
      type: "choose_dialogue",
      node_id: "demo_morning_001",
      choice_id: "encourage",
    });

    expect(world.active_dialogue).toHaveLength(2);
    expect(world.active_dialogue[1].text).toContain("稳定重放");
    expect(world.characters[0].state.mood).toBe(13);
    expect(world.relationships[0].affinity).toBe(7);
    expect(world.relationships[0].trust).toBe(1);
  });

  it("gates dialogue choices by conditions", () => {
    const started = applyDemoCommand(createDemoWorld(), {
      type: "start_dialogue",
      scene_id: "demo_morning",
    });
    const rejected = applyDemoCommand(started, {
      type: "choose_dialogue",
      node_id: "demo_morning_001",
      choice_id: "talk_about_trust",
    });

    expect(rejected).toEqual(started);
    expect(rejected.command_log).toHaveLength(1);

    const unlocked = applyDemoCommand(createDemoWorld(), {
      type: "adjust_relationship",
      source_character_id: "player",
      target_character_id: "demo_heroine",
      affinity_delta: 2,
      trust_delta: 0,
    });
    const unlockedStarted = applyDemoCommand(unlocked, {
      type: "start_dialogue",
      scene_id: "demo_morning",
    });

    expect(
      visibleChoices(unlockedStarted, unlockedStarted.active_dialogue[0]).some(
        (choice) => choice.id === "talk_about_trust",
      ),
    ).toBe(true);

    const chosen = applyDemoCommand(unlockedStarted, {
      type: "choose_dialogue",
      node_id: "demo_morning_001",
      choice_id: "talk_about_trust",
    });

    expect(chosen.active_dialogue[1].text).toContain("信任会一点点积累");
    expect(chosen.relationships[0].trust).toBe(2);
  });

  it("records successful commands only", () => {
    const advanced = applyDemoCommand(createDemoWorld(), {
      type: "advance_time",
      minutes: 30,
    });
    const rejected = applyDemoCommand(advanced, {
      type: "move_character",
      character_id: "demo_heroine",
      location_id: "missing",
    });

    expect(rejected.command_log).toHaveLength(1);
    expect(rejected.command_log[0]).toEqual({
      type: "advance_time",
      minutes: 30,
    });
  });

  it("adjusts relationships through command api", () => {
    const world = applyDemoCommand(createDemoWorld(), {
      type: "adjust_relationship",
      source_character_id: "player",
      target_character_id: "demo_heroine",
      affinity_delta: 120,
      trust_delta: 2,
    });

    expect(world.relationships[0].affinity).toBe(100);
    expect(world.relationships[0].trust).toBe(2);
    expect(world.command_log[0]).toEqual({
      type: "adjust_relationship",
      source_character_id: "player",
      target_character_id: "demo_heroine",
      affinity_delta: 120,
      trust_delta: 2,
    });
  });

  it("rolls character mood from explicit rng state", () => {
    const first = applyDemoCommand(createDemoWorld(), {
      type: "roll_character_mood",
      character_id: "demo_heroine",
      min_delta: -5,
      max_delta: 5,
    });
    const second = applyDemoCommand(createDemoWorld(), {
      type: "roll_character_mood",
      character_id: "demo_heroine",
      min_delta: -5,
      max_delta: 5,
    });

    expect(first).toEqual(second);
    expect(first.random.cursor).toBe("1");
    expect(first.command_log_initial_random).toEqual({
      seed: "1163026804",
      cursor: "0",
    });
    expect(first.characters[0].state.mood).toBeGreaterThanOrEqual(5);
    expect(first.characters[0].state.mood).toBeLessThanOrEqual(15);
    expect(first.command_log[0]).toEqual({
      type: "roll_character_mood",
      character_id: "demo_heroine",
      min_delta: -5,
      max_delta: 5,
    });
  });

  it("rolls dialogue character state effects deterministically", () => {
    const base = createDemoWorld();
    base.dialogue_scenes[0].nodes[0].choices.push({
      id: "random_state",
      label: "随机状态",
      next_node_id: null,
      conditions: [],
      effects: [
        {
          type: "roll_character_state",
          character_id: "demo_heroine",
          energy_min_delta: -3,
          energy_max_delta: 0,
          mood_min_delta: -2,
          mood_max_delta: 4,
        },
      ],
    });
    const started = applyDemoCommand(base, {
      type: "start_dialogue",
      scene_id: "demo_morning",
    });

    const first = applyDemoCommand(started, {
      type: "choose_dialogue",
      node_id: "demo_morning_001",
      choice_id: "random_state",
    });
    const second = applyDemoCommand(started, {
      type: "choose_dialogue",
      node_id: "demo_morning_001",
      choice_id: "random_state",
    });

    expect(first).toEqual(second);
    expect(first.random.cursor).toBe("2");
    expect(first.characters[0].state.energy).toBeGreaterThanOrEqual(77);
    expect(first.characters[0].state.energy).toBeLessThanOrEqual(80);
    expect(first.characters[0].state.mood).toBeGreaterThanOrEqual(8);
    expect(first.characters[0].state.mood).toBeLessThanOrEqual(14);
  });

  it("rejects invalid dialogue random effects without consuming rng state", () => {
    const base = createDemoWorld();
    base.dialogue_scenes[0].nodes[0].choices.push({
      id: "invalid_random_state",
      label: "非法随机状态",
      next_node_id: null,
      conditions: [],
      effects: [
        {
          type: "roll_character_state",
          character_id: "demo_heroine",
          energy_min_delta: 1,
          energy_max_delta: -1,
          mood_min_delta: 0,
          mood_max_delta: 1,
        },
      ],
    });
    const started = applyDemoCommand(base, {
      type: "start_dialogue",
      scene_id: "demo_morning",
    });
    const rejected = applyDemoCommand(started, {
      type: "choose_dialogue",
      node_id: "demo_morning_001",
      choice_id: "invalid_random_state",
    });

    expect(rejected).toEqual(started);
    expect(rejected.random.cursor).toBe("0");
    expect(rejected.command_log).toHaveLength(1);
  });

  it("rejects invalid random commands without consuming rng state", () => {
    const world = applyDemoCommand(createDemoWorld(), {
      type: "roll_character_mood",
      character_id: "demo_heroine",
      min_delta: 5,
      max_delta: -5,
    });

    expect(world.random.cursor).toBe("0");
    expect(world.command_log).toHaveLength(0);
    expect(world.characters[0].state.mood).toBe(10);
  });

  it("replays browser command logs from captured rng state", () => {
    const base = createDemoWorld();
    base.random = {
      seed: "987654321",
      cursor: "7",
    };

    const rolled = applyDemoCommand(base, {
      type: "roll_character_mood",
      character_id: "demo_heroine",
      min_delta: -5,
      max_delta: 5,
    });
    const advanced = applyDemoCommand(rolled, {
      type: "advance_time",
      minutes: 30,
    });
    const replayed = replayDemoCommandLog(createDemoWorld(), {
      schema_version: 1,
      engine_version: advanced.engine_version,
      initial_random: advanced.command_log_initial_random!,
      commands: advanced.command_log,
    });

    expect(advanced.command_log_initial_random).toEqual({
      seed: "987654321",
      cursor: "7",
    });
    expect(replayed).toEqual(advanced);
  });

  it("creates a browser save preview envelope", async () => {
    const client = createBrowserMockEngineClient();

    const save = await client.savePreview("slot-1", 1000);

    expect(save.schema_version).toBe(1);
    expect(save.slot_id).toBe("slot-1");
    expect(save.world.engine_version).toBe("0.1.0-m0");
  });

  it("exposes browser save and load slot operations", async () => {
    const client = createBrowserMockEngineClient();

    const report = await client.saveSlot("slot_1", 1000);
    await client.dispatch({
      type: "move_character",
      character_id: "demo_heroine",
      location_id: "garden",
    });
    const world = await client.loadSlot("slot_1");

    expect(report.path).toBe("browser-memory://slot_1.json");
    expect(report.backup_path).toBeNull();
    expect(world.engine_version).toBe("0.1.0-m0");
    expect(world.characters[0].location_id).toBe("school_gate");
  });

  it("installs browser content packages and triggers their scheduled events", async () => {
    const client = createBrowserMockEngineClient();

    const installed = await client.installContentPackage(createSampleContentPackage());
    const advanced = await client.dispatch({ type: "advance_time", minutes: 20 });

    expect(
      installed.dialogue_scenes.some((scene) => scene.id === "sample_event_dialogue"),
    ).toBe(true);
    expect(installed.locations.some((location) => location.id === "sample_studio")).toBe(
      true,
    );
    expect(
      installed.characters.some((character) => character.id === "sample_guest"),
    ).toBe(true);
    expect(
      installed.relationships.some(
        (relationship) =>
          relationship.source_character_id === "player" &&
          relationship.target_character_id === "sample_guest",
      ),
    ).toBe(true);
    expect(
      installed.resources.some(
        (resource) => resource.resource_id === "sample.event_pack.guest.smile",
      ),
    ).toBe(true);
    expect(installed.installed_content_packages).toEqual([
      {
        namespace: "sample",
        package_id: "sample.event_pack",
        version: "0.1.0",
        dependencies: [],
        conflicts: [],
      },
    ]);
    expect(installed.scheduled_events[0].id).toBe("sample_content_dialogue_at_0820");
    expect(advanced.active_dialogue_scene_id).toBe("sample_event_dialogue");
    expect(advanced.active_dialogue[0].speaker_id).toBe("sample_guest");
    expect(advanced.active_dialogue[0].text).toContain("随内容包新增的角色");
  });

  it("records installed browser content packages in save dependencies", async () => {
    const client = createBrowserMockEngineClient();

    await client.installContentPackage(createSampleContentPackage());
    const save = await client.savePreview("slot-1", 1000);

    expect(save.mod_dependencies).toEqual([
      {
        namespace: "sample.event_pack",
        version: "0.1.0",
        required: true,
      },
    ]);
  });

  it("preflights browser saves against enabled mod registry", async () => {
    const client = createBrowserMockEngineClient();
    const packageData = createSampleContentPackage();
    packageData.manifest.package_id = "example.minimal_character";

    await client.installContentPackage(packageData);
    await client.saveSlot("slot_1", 1000);

    const ready = await client.preflightLoadSlot(
      "slot_1",
      "examples/mods",
      [],
      "0.1.0-m0",
    );
    const blocked = await client.preflightLoadSlot(
      "slot_1",
      "examples/mods",
      [{ namespace: "example.minimal_character", enabled: false }],
      "0.1.0-m0",
    );

    expect(ready.ready).toBe(true);
    expect(ready.registry.enabled).toEqual([
      {
        namespace: "example.minimal_character",
        version: "0.1.0",
        conflicts: [],
      },
    ]);
    expect(ready.validation.missing_required_mods).toEqual([]);
    expect(blocked.ready).toBe(false);
    expect(blocked.validation.missing_required_mods).toEqual([
      {
        namespace: "example.minimal_character",
        version: "0.1.0",
        required: true,
      },
    ]);
  });

  it("recovers browser saves from the latest backup", async () => {
    const client = createBrowserMockEngineClient();
    await client.saveSlot("slot_1", 100);
    await client.dispatch({
      type: "advance_time",
      minutes: 30,
    });
    await client.saveSlot("slot_1", 200);

    const recovered = await client.recoverSlot("slot_1", 300);
    const loaded = await client.loadSlot("slot_1");

    expect(recovered.path).toBe("browser-memory://slot_1.json");
    expect(recovered.failed_primary_backup_path).toBe(
      "browser-memory://slot_1.json.failed.300.bak",
    );
    expect(recovered.save.saved_at_unix_ms).toBe(100);
    expect(loaded.clock.minute).toBe(0);
  });

  it("rotates browser save backups to the latest ten entries", async () => {
    const client = createBrowserMockEngineClient();

    for (let savedAt = 0; savedAt < 12; savedAt += 1) {
      await client.saveSlot("slot_1", savedAt);
      await client.dispatch({
        type: "advance_time",
        minutes: 1,
      });
    }

    const recovered = await client.recoverSlot("slot_1", 20);

    expect(recovered.recovered_from).toBe("browser-memory://slot_1.json.backup-10");
    expect(recovered.save.saved_at_unix_ms).toBe(10);
  });

  it("rejects browser content packages with missing dependencies or conflicts", async () => {
    const client = createBrowserMockEngineClient();
    const packageData = createSampleContentPackage();
    packageData.manifest.dependencies = [
      {
        package_id: "sample.missing",
        version: null,
        required: true,
      },
    ];

    const unchanged = await client.installContentPackage(packageData);

    expect(unchanged.installed_content_packages).toEqual([]);

    const base = createSampleContentPackage();
    const conflict = createSampleContentPackage();
    conflict.manifest.package_id = "sample.conflict";
    conflict.manifest.conflicts = ["sample.event_pack"];
    await client.installContentPackage(base);
    const afterConflict = await client.installContentPackage(conflict);

    expect(afterConflict.installed_content_packages).toEqual([
      {
        namespace: "sample",
        package_id: "sample.event_pack",
        version: "0.1.0",
        dependencies: [],
        conflicts: [],
      },
    ]);
  });

  it("accepts browser content package dependency strings as required dependencies", async () => {
    const client = createBrowserMockEngineClient();
    await client.installContentPackage(createSampleContentPackage());
    const addon = createSampleContentPackage();
    addon.manifest.package_id = "sample.addon";
    addon.manifest.dependencies = ["sample.event_pack"];
    addon.locations = [];
    addon.characters = [];
    addon.relationships = [];
    addon.resources = [];
    addon.dialogue_scenes = [];
    addon.scheduled_events = [];

    const installed = await client.installContentPackage(addon);

    expect(
      installed.installed_content_packages.find(
        (packageInfo) => packageInfo.package_id === "sample.addon",
      )?.dependencies,
    ).toEqual([
      {
        package_id: "sample.event_pack",
        version: null,
        required: true,
      },
    ]);
  });

  it("installs browser content packages with registry dependencies", async () => {
    const client = createBrowserMockEngineClient();
    const addon = createSampleContentPackage();
    addon.manifest.package_id = "sample.addon";
    addon.manifest.dependencies = [
      {
        package_id: "sample.base",
        version: "0.1.0",
        required: true,
      },
    ];
    addon.locations = [];
    addon.characters = [];
    addon.relationships = [];
    addon.resources = [];
    addon.dialogue_scenes = [];
    addon.scheduled_events = [];

    const rejected = await client.installContentPackage(addon);
    const installed = await client.installContentPackage(addon, {
      enabled: [
        {
          namespace: "sample.base",
          version: "0.1.0",
          conflicts: [],
        },
      ],
    });

    expect(rejected.installed_content_packages).toEqual([]);
    expect(
      installed.installed_content_packages.some(
        (packageInfo) => packageInfo.package_id === "sample.addon",
      ),
    ).toBe(true);
  });

  it("preflights browser content package registry dependencies", async () => {
    const client = createBrowserMockEngineClient();
    const addon = createSampleContentPackage();
    addon.manifest.package_id = "sample.addon";
    addon.manifest.dependencies = [
      {
        package_id: "sample.base",
        version: "0.1.0",
        required: true,
      },
    ];
    addon.locations = [];
    addon.characters = [];
    addon.relationships = [];
    addon.resources = [];
    addon.dialogue_scenes = [];
    addon.scheduled_events = [];

    const blocked = await client.preflightContentPackageInstall(addon);
    const ready = await client.preflightContentPackageInstall(addon, {
      enabled: [
        {
          namespace: "sample.base",
          version: "0.1.0",
          conflicts: [],
        },
      ],
    });

    expect(blocked.ready).toBe(false);
    expect(blocked.issues[0].code).toBe("missing_content_package_dependency");
    expect(ready.ready).toBe(true);
    expect(ready.issues).toEqual([]);
  });

  it("plans browser resource loads with safe paths and fallbacks", async () => {
    const client = createBrowserMockEngineClient();

    const report = await client.planResources("mods/sample");

    expect(report.low_spec).toBe(false);
    expect(report.entries[0]).toMatchObject({
      resource_id: "core.demo.heroine.neutral",
      source_path: "assets/demo/heroine-neutral.webp",
      resolved_path: "mods/sample/assets/demo/heroine-neutral.webp",
      media_type: "image",
      status: "planned",
      load_strategy: "eager",
      fallback: "placeholder_image",
      expected_sha256: null,
      actual_sha256: null,
    });
    expect(report.entries[0].cache_key).toMatch(
      /^core\.demo\.heroine\.neutral-[a-f0-9]{16}$/,
    );
    expect(report.entries[0].cache_path).toContain(
      "mods/sample/.eratw-cache/resources/core.demo.heroine.neutral-",
    );
    expect(report.entries[0].cache_path).toMatch(/\.webp$/);
    expect(report.entries[0].thumbnail_path).toBeNull();
  });

  it("preflights browser resource loads", async () => {
    const client = createBrowserMockEngineClient();

    const report = await client.preflightResources("mods/sample");

    expect(report.ready).toBe(true);
    expect(report.issues).toEqual([]);
    expect(report.resolution.entries[0]).toMatchObject({
      resource_id: "core.demo.heroine.neutral",
      status: "planned",
      fallback: "placeholder_image",
    });
  });

  it("plans browser resource loads for low spec mode", async () => {
    const client = createBrowserMockEngineClient();

    const report = await client.preflightResources("mods/sample", true);

    expect(report.ready).toBe(true);
    expect(report.low_spec).toBe(true);
    expect(report.resolution.low_spec).toBe(true);
    expect(report.resolution.entries[0]).toMatchObject({
      resource_id: "core.demo.heroine.neutral",
      load_strategy: "thumbnail_only",
      cache_path: expect.stringContaining(
        "mods/sample/.eratw-cache/resources/core.demo.heroine.neutral-",
      ) as string,
      thumbnail_path: expect.stringContaining(
        "mods/sample/.eratw-cache/thumbnails/core.demo.heroine.neutral-",
      ) as string,
    });
    expect(report.resolution.entries[0].thumbnail_path).toMatch(/\.webp$/);
  });

  it("audits browser resources before publication", async () => {
    const client = createBrowserMockEngineClient();

    const report = await client.auditResourcePublication("mods/sample", true);

    expect(report.ready).toBe(true);
    expect(report.low_spec).toBe(true);
    expect(report.error_count).toBe(0);
    expect(report.warning_count).toBe(1);
    expect(report.issues).toEqual([
      {
        severity: "warning",
        code: "missing_sha256",
        resource_id: "core.demo.heroine.neutral",
        source_path: "assets/demo/heroine-neutral.webp",
        message: "resource sha256 is missing: core.demo.heroine.neutral",
        fallback: "placeholder_image",
      },
    ]);
  });

  it("simulates browser resource cache reports", async () => {
    const client = createBrowserMockEngineClient();

    const report = await client.cacheResources("mods/sample", true);

    expect(report.ready).toBe(true);
    expect(report.low_spec).toBe(true);
    expect(report.cached_count).toBe(1);
    expect(report.skipped_count).toBe(0);
    expect(report.failed_count).toBe(0);
    expect(report.entries[0]).toMatchObject({
      resource_id: "core.demo.heroine.neutral",
      status: "cached",
      cache_path: expect.stringContaining(
        "mods/sample/.eratw-cache/resources/core.demo.heroine.neutral-",
      ) as string,
    });
  });

  it("simulates browser resource cache clean reports", async () => {
    const client = createBrowserMockEngineClient();

    const report = await client.cleanResourceCache("mods/sample", true);

    expect(report.ready).toBe(true);
    expect(report.low_spec).toBe(true);
    expect(report.cache_root).toBe("mods/sample/.eratw-cache");
    expect(report.kept_count).toBe(2);
    expect(report.removed_count).toBe(0);
    expect(report.failed_count).toBe(0);
    expect(report.entries.map((entry) => entry.status)).toEqual(["kept", "kept"]);
    expect(report.entries[0].path).toContain(
      "mods/sample/.eratw-cache/resources/core.demo.heroine.neutral-",
    );
    expect(report.entries[1].path).toContain(
      "mods/sample/.eratw-cache/thumbnails/core.demo.heroine.neutral-",
    );
  });

  it("discovers browser mod manifests through the engine client", async () => {
    const client = createBrowserMockEngineClient();

    const report = await client.discoverMods("examples/mods", "0.1.0-m0");

    expect(report.discovered).toHaveLength(1);
    expect(report.discovered[0]).toMatchObject({
      root_path: "examples/mods/minimal-character",
      manifest_path: "examples/mods/minimal-character/manifest.json",
      manifest: {
        namespace: "example.minimal_character",
        name: "最小角色 Mod",
        version: "0.1.0",
        engine_version: "0.1.0-m0",
        load_order: 0,
        dependencies: [],
        conflicts: [],
        capabilities: ["content"],
      },
    });
    expect(report.errors).toEqual([]);
  });

  it("reports browser mod discovery compatibility errors", async () => {
    const client = createBrowserMockEngineClient();

    const report = await client.discoverMods("examples/mods", "9.9.9");

    expect(report.discovered).toEqual([]);
    expect(report.errors[0]).toMatchObject({
      path: "examples/mods/minimal-character/manifest.json",
      kind: "incompatible_engine_version",
    });
  });

  it("plans browser mod install operations without copying files", async () => {
    const client = createBrowserMockEngineClient();

    const plan = await client.planModInstall(
      "downloads/minimal-character",
      "mods/installed",
      "0.1.0-m0",
    );

    expect(plan).toMatchObject({
      source_root: "downloads/minimal-character",
      install_root: "mods/installed",
      target_root: "mods/installed/example.minimal_character",
      staging_root: "mods/installed/.installing-example.minimal_character",
      manifest_path: "downloads/minimal-character/manifest.json",
      manifest: {
        namespace: "example.minimal_character",
      },
      actions: [
        {
          kind: "create_directory",
          path: "mods/installed",
          from: null,
          to: null,
        },
        {
          kind: "copy_directory",
          path: null,
          from: "downloads/minimal-character",
          to: "mods/installed/.installing-example.minimal_character",
        },
        {
          kind: "move_directory",
          path: null,
          from: "mods/installed/.installing-example.minimal_character",
          to: "mods/installed/example.minimal_character",
        },
      ],
    });
  });

  it("simulates browser mod install execution report", async () => {
    const client = createBrowserMockEngineClient();

    const report = await client.installMod(
      "downloads/minimal-character",
      "mods/installed",
      "0.1.0-m0",
    );

    expect(report).toMatchObject({
      target_root: "mods/installed/example.minimal_character",
      manifest: {
        namespace: "example.minimal_character",
      },
    });
    expect(report.actions.map((action) => action.kind)).toEqual([
      "create_directory",
      "copy_directory",
      "move_directory",
    ]);
  });

  it("preflights browser mod package installation", async () => {
    const client = createBrowserMockEngineClient();

    const report = await client.preflightModPackageInstall(
      "packages/example.minimal_character-0.1.0",
      "mods/installed",
      "0.1.0-m0",
    );

    expect(report).toMatchObject({
      ready: true,
      source_root: "packages/example.minimal_character-0.1.0",
      content_root: "packages/example.minimal_character-0.1.0/content",
      target_root: "mods/installed/example.minimal_character",
      manifest: {
        namespace: "example.minimal_character",
      },
    });
    expect(report.issues).toEqual([
      {
        severity: "warning",
        path: "packages/example.minimal_character-0.1.0/content/assets/readme.txt",
        kind: "resource_publication_warning",
        message:
          "resource sha256 is missing: example.minimal_character.assets.readme",
      },
    ]);
  });

  it("preflights browser mod package compatibility errors", async () => {
    const client = createBrowserMockEngineClient();

    const report = await client.preflightModPackageInstall(
      "packages/example.minimal_character-0.1.0",
      "mods/installed",
      "9.9.9",
    );

    expect(report.ready).toBe(false);
    expect(report.issues[0]).toMatchObject({
      severity: "error",
      kind: "incompatible_engine_version",
    });
  });

  it("simulates browser mod package install execution report", async () => {
    const client = createBrowserMockEngineClient();

    const report = await client.installModPackage(
      "packages/example.minimal_character-0.1.0",
      "mods/installed",
      "0.1.0-m0",
    );

    expect(report).toMatchObject({
      target_root: "mods/installed/example.minimal_character",
      manifest: {
        namespace: "example.minimal_character",
      },
    });
    expect(report.actions.map((action) => action.kind)).toEqual([
      "create_directory",
      "copy_directory",
      "move_directory",
    ]);
  });

  it("discovers browser mods after package installation", async () => {
    const client = createBrowserMockEngineClient();

    expect(
      (await client.discoverMods("mods/installed", "0.1.0-m0")).discovered,
    ).toEqual([]);

    await client.installModPackage(
      "packages/example.minimal_character-0.1.0",
      "mods/installed",
      "0.1.0-m0",
    );

    const discovery = await client.discoverMods("mods/installed", "0.1.0-m0");

    expect(discovery.discovered).toHaveLength(1);
    expect(discovery.discovered[0]).toMatchObject({
      root_path: "mods/installed/example.minimal_character",
      manifest_path: "mods/installed/example.minimal_character/manifest.json",
      manifest: {
        namespace: "example.minimal_character",
      },
    });
  });

  it("blocks duplicate browser package installation after install", async () => {
    const client = createBrowserMockEngineClient();

    await client.installModPackage(
      "packages/example.minimal_character-0.1.0",
      "mods/installed",
      "0.1.0-m0",
    );

    const preflight = await client.preflightModPackageInstall(
      "packages/example.minimal_character-0.1.0",
      "mods/installed",
      "0.1.0-m0",
    );

    expect(preflight.ready).toBe(false);
    expect(preflight.issues).toContainEqual(
      expect.objectContaining({
        severity: "error",
        kind: "install_target_exists",
        path: "mods/installed/example.minimal_character",
      }),
    );
    await expect(
      client.installModPackage(
        "packages/example.minimal_character-0.1.0",
        "mods/installed",
        "0.1.0-m0",
      ),
    ).rejects.toMatchObject({
      kind: "install_target_exists",
    });
  });

  it("rejects browser mod package install when preflight is blocked", async () => {
    const client = createBrowserMockEngineClient();

    await expect(
      client.installModPackage(
        "packages/example.minimal_character-0.1.0",
        "mods/installed",
        "9.9.9",
      ),
    ).rejects.toMatchObject({
      kind: "incompatible_engine_version",
    });
  });

  it("reports browser mod install compatibility errors", async () => {
    const client = createBrowserMockEngineClient();

    await expect(
      client.planModInstall("downloads/minimal-character", "mods/installed", "9.9.9"),
    ).rejects.toMatchObject({
      kind: "incompatible_engine_version",
    });
  });

  it("plans browser mod uninstall operations", async () => {
    const client = createBrowserMockEngineClient();

    const plan = await client.planModUninstall(
      "mods/installed",
      "example.minimal_character",
    );

    expect(plan).toMatchObject({
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
  });

  it("simulates browser mod uninstall execution report", async () => {
    const client = createBrowserMockEngineClient();

    const report = await client.uninstallMod(
      "mods/installed",
      "example.minimal_character",
    );

    expect(report).toMatchObject({
      namespace: "example.minimal_character",
      target_root: "mods/installed/example.minimal_character",
    });
    expect(report.actions.map((action) => action.kind)).toEqual([
      "move_directory",
      "delete_directory",
    ]);
  });

  it("reports browser mod uninstall unsafe namespace errors", async () => {
    const client = createBrowserMockEngineClient();

    await expect(
      client.planModUninstall("mods/installed", "../outside"),
    ).rejects.toMatchObject({
      kind: "unsafe_install_namespace",
    });
  });

  it("plans browser enabled mods from discovered manifests", async () => {
    const client = createBrowserMockEngineClient();
    const discovery = await client.discoverMods("examples/mods", "0.1.0-m0");

    const plan = await client.planEnabledMods(
      discovery.discovered.map((entry) => entry.manifest),
      [],
      "0.1.0-m0",
    );

    expect(plan.enabled.map((manifest) => manifest.namespace)).toEqual([
      "example.minimal_character",
    ]);
    expect(plan.disabled).toEqual([]);
  });

  it("reports browser enabled mod dependency errors", async () => {
    const client = createBrowserMockEngineClient();
    const base = {
      ...sampleBrowserMod("core.base"),
      load_order: -10,
    };
    const addon = {
      ...sampleBrowserMod("example.addon"),
      dependencies: [
        {
          namespace: "core.base",
          version: null,
          required: true,
        },
      ],
    };

    await expect(
      client.planEnabledMods(
        [base, addon],
        [{ namespace: "core.base", enabled: false }],
        "0.1.0-m0",
      ),
    ).rejects.toMatchObject({
      kind: "missing_dependency",
    });
  });

  it("requires explicit browser mod capability authorization", async () => {
    const client = createBrowserMockEngineClient();
    const unsafeMod = {
      ...sampleBrowserMod("example.unsafe"),
      capabilities: ["network_access" as const],
    };

    await expect(
      client.planEnabledMods([unsafeMod], [], "0.1.0-m0"),
    ).rejects.toMatchObject({
      kind: "unsafe_capability",
    });

    const plan = await client.planEnabledMods(
      [unsafeMod],
      [],
      "0.1.0-m0",
      ["network_access"],
    );

    expect(plan.enabled.map((manifest) => manifest.namespace)).toEqual([
      "example.unsafe",
    ]);
  });

  it("rejects browser content packages with unsafe resource paths", async () => {
    const client = createBrowserMockEngineClient();
    const packageData = createSampleContentPackage();
    packageData.resources[0].source_path = "../outside.webp";

    const unchanged = await client.installContentPackage(packageData);

    expect(unchanged.resources).toHaveLength(1);
    expect(unchanged.installed_content_packages).toEqual([]);
  });

  it("preflights browser content package dialogue placeholders", async () => {
    const client = createBrowserMockEngineClient();
    const packageData = createSampleContentPackage();
    packageData.dialogue_scenes[0].nodes[0].text =
      "未知变量 {{ legacy.mood }} 和类型错误 {{ clock.day:text }}。";

    const preflight = await client.preflightContentPackageInstall(packageData);
    const unchanged = await client.installContentPackage(packageData);

    expect(preflight.ready).toBe(false);
    expect(preflight.issues).toEqual([
      {
        code: "validation_failed",
        message: "content validation failed with 2 issue(s)",
      },
    ]);
    expect(preflight.validation?.issues.map((issue) => issue.code)).toEqual([
      "unknown_dialogue_placeholder",
      "dialogue_placeholder_type_mismatch",
    ]);
    expect(unchanged.installed_content_packages).toEqual([]);
  });

  it("preflights browser content package random effect ranges", async () => {
    const client = createBrowserMockEngineClient();
    const packageData = createSampleContentPackage();
    packageData.dialogue_scenes[0].nodes[0].choices[0].effects = [
      {
        type: "roll_character_state",
        character_id: "sample_guest",
        energy_min_delta: 2,
        energy_max_delta: -1,
        mood_min_delta: -1,
        mood_max_delta: 1,
      },
    ];

    const preflight = await client.preflightContentPackageInstall(packageData);
    const unchanged = await client.installContentPackage(packageData);

    expect(preflight.ready).toBe(false);
    expect(preflight.validation?.issues).toEqual([
      {
        code: "invalid_effect_random_range",
        target: "sample_event_dialogue:sample_event_entry:acknowledge",
      },
    ]);
    expect(unchanged.installed_content_packages).toEqual([]);
  });

  it("preflights browser content package scheduled random event ranges", async () => {
    const client = createBrowserMockEngineClient();
    const packageData = createSampleContentPackage();
    packageData.scheduled_events[0].kind = {
      type: "roll_character_state",
      character_id: "sample_guest",
      energy_min_delta: 2,
      energy_max_delta: -1,
      mood_min_delta: -1,
      mood_max_delta: 1,
    };

    const preflight = await client.preflightContentPackageInstall(packageData);
    const unchanged = await client.installContentPackage(packageData);

    expect(preflight.ready).toBe(false);
    expect(preflight.validation?.issues).toEqual([
      {
        code: "invalid_scheduled_event_random_range",
        target: "scheduled_event:sample_content_dialogue_at_0820",
      },
    ]);
    expect(unchanged.installed_content_packages).toEqual([]);
  });
});

const sampleBrowserMod = (namespace: string) => ({
  namespace,
  name: namespace,
  version: "0.1.0",
  engine_version: "0.1.0-m0",
  load_order: 0,
  dependencies: [],
  conflicts: [],
  capabilities: ["content" as const],
  resources: [],
});
