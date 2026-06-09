import { describe, expect, it } from "vitest";
import { createBrowserMockEngineClient } from "./client";
import { applyDemoCommand, createDemoWorld } from "./demoWorld";

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

  it("starts a versioned dialogue scene", () => {
    const world = applyDemoCommand(createDemoWorld(), {
      type: "start_dialogue",
      scene_id: "demo_morning",
    });

    expect(world.active_dialogue_scene_id).toBe("demo_morning");
    expect(world.active_dialogue).toHaveLength(1);
    expect(world.active_dialogue[0].choices).toHaveLength(2);
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
