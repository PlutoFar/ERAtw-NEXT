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

    expect(world.active_dialogue).toHaveLength(2);
    expect(world.active_dialogue[1].text).toContain("不执行旧 ERB");
  });

  it("creates a browser save preview envelope", async () => {
    const client = createBrowserMockEngineClient();

    const save = await client.savePreview("slot-1", 1000);

    expect(save.schema_version).toBe(1);
    expect(save.slot_id).toBe("slot-1");
    expect(save.world.engine_version).toBe("0.1.0-m0");
  });
});
