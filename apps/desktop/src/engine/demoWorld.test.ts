import { describe, expect, it } from "vitest";
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

  it("starts a versioned dialogue scene", () => {
    const world = applyDemoCommand(createDemoWorld(), {
      type: "start_dialogue",
      scene_id: "demo_morning",
    });

    expect(world.active_dialogue).toHaveLength(2);
    expect(world.active_dialogue[1].text).toContain("不执行旧 ERB");
  });
});
