import { describe, expect, it } from "vitest";
import { createBrowserMockEngineClient } from "./client";
import { applyDemoCommand, createDemoWorld, visibleChoices } from "./demoWorld";

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
    expect(first.characters[0].state.mood).toBeGreaterThanOrEqual(5);
    expect(first.characters[0].state.mood).toBeLessThanOrEqual(15);
    expect(first.command_log[0]).toEqual({
      type: "roll_character_mood",
      character_id: "demo_heroine",
      min_delta: -5,
      max_delta: 5,
    });
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
});
