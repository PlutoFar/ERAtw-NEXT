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
