import { describe, expect, it } from "vitest";
import { mockMapModel } from "../engine/mockData";
import {
  buildAsciiBuffer,
  legendByKey,
  nearestNodeInDirection,
  nodesInArea,
} from "./mapGeometry";

describe("mapGeometry", () => {
  it("buildAsciiBuffer 放置了所有区域节点", () => {
    const buffer = buildAsciiBuffer(mockMapModel, "village");
    const villageNodes = nodesInArea(mockMapModel, "village");
    const nodeCells = buffer.cells.filter((c) => c.type === "node");
    expect(nodeCells).toHaveLength(villageNodes.length);
    for (const node of villageNodes) {
      expect(buffer.nodePositions[node.id]).toBeDefined();
    }
  });

  it("buildAsciiBuffer 绘制了连线字符", () => {
    const buffer = buildAsciiBuffer(mockMapModel, "village");
    const linkCells = buffer.cells.filter((c) => c.type === "link");
    expect(linkCells.length).toBeGreaterThan(0);
  });

  it("legendByKey 覆盖所有活动键", () => {
    const legend = legendByKey(mockMapModel);
    expect(legend.working.label).toBe("工作中");
    expect(legend.sleeping.color).toMatch(/^#[0-9a-f]{6}$/i);
  });

  it("nearestNodeInDirection 朝指定方向选择节点", () => {
    // plaza 在 (22,12)，其正北方向应能选到参道入口/广场北侧节点。
    const villageNodes = nodesInArea(mockMapModel, "village");
    const up = nearestNodeInDirection(villageNodes, "plaza", "up");
    expect(up).toBeDefined();
    const upNode = villageNodes.find((n) => n.id === up)!;
    expect(upNode.y).toBeLessThan(12);
  });
});
