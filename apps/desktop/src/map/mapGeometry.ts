// 地图几何与字符画缓冲构建：两种渲染器共享的纯函数，便于单测。

import type { ActivityKey, LegendEntry, MapModel, MapNode } from "../types";

export type Direction = "up" | "down" | "left" | "right";

export function nodesInArea(model: MapModel, areaId: string): MapNode[] {
  return model.nodes.filter((node) => node.areaId === areaId);
}

export function nodeById(model: MapModel, id: string | undefined): MapNode | undefined {
  if (!id) {
    return undefined;
  }
  return model.nodes.find((node) => node.id === id);
}

export function legendByKey(model: MapModel): Record<ActivityKey, LegendEntry> {
  const out = {} as Record<ActivityKey, LegendEntry>;
  for (const entry of model.legend) {
    out[entry.key] = entry;
  }
  return out;
}

/** 节点主导活动（用于着色）：取第一个占用者的活动。 */
export function dominantActivity(node: MapNode): ActivityKey | undefined {
  return node.occupants[0]?.activity;
}

export interface AsciiCell {
  col: number;
  row: number;
  ch: string;
  type: "empty" | "link" | "node";
  nodeId?: string;
}

export interface AsciiBuffer {
  columns: number;
  rows: number;
  cells: AsciiCell[];
  /** 节点 id -> 在缓冲中的本地坐标 */
  nodePositions: Record<string, { col: number; row: number }>;
}

const HORIZONTAL = "─";
const VERTICAL = "│";
const CROSS = "┼";

/**
 * 把某区域的节点与连线渲染成一个紧凑的字符网格缓冲。
 * 连线采用先横后竖的 L 形路径，交叉处用 ┼。
 */
export function buildAsciiBuffer(model: MapModel, areaId: string, padding = 1): AsciiBuffer {
  const nodes = nodesInArea(model, areaId);
  if (nodes.length === 0) {
    return { columns: 1, rows: 1, cells: [], nodePositions: {} };
  }

  const minX = Math.min(...nodes.map((n) => n.x));
  const minY = Math.min(...nodes.map((n) => n.y));
  const maxX = Math.max(...nodes.map((n) => n.x));
  const maxY = Math.max(...nodes.map((n) => n.y));

  const columns = maxX - minX + 1 + padding * 2;
  const rows = maxY - minY + 1 + padding * 2;

  type Slot = { ch: string; type: AsciiCell["type"]; nodeId?: string };
  const grid: Slot[][] = Array.from({ length: rows }, () =>
    Array.from({ length: columns }, () => ({ ch: " ", type: "empty" as const })),
  );

  const toCol = (x: number) => x - minX + padding;
  const toRow = (y: number) => y - minY + padding;

  const nodePositions: Record<string, { col: number; row: number }> = {};
  const inArea = new Set(nodes.map((n) => n.id));
  for (const node of nodes) {
    nodePositions[node.id] = { col: toCol(node.x), row: toRow(node.y) };
  }

  const setLink = (col: number, row: number, ch: string) => {
    const slot = grid[row][col];
    if (slot.type === "node") {
      return;
    }
    if (slot.type === "link" && slot.ch !== ch) {
      slot.ch = CROSS;
    } else {
      slot.ch = ch;
      slot.type = "link";
    }
  };

  // 连线（去重：只在 id < target 时绘制一次）。
  for (const node of nodes) {
    const ay = toRow(node.y);
    const ax = toCol(node.x);
    for (const target of node.links) {
      if (!inArea.has(target) || node.id >= target) {
        continue;
      }
      const tnode = nodePositions[target];
      const bx = tnode.col;
      const by = tnode.row;
      const stepX = Math.sign(bx - ax);
      for (let cx = ax + stepX; cx !== bx && stepX !== 0; cx += stepX) {
        setLink(cx, ay, HORIZONTAL);
      }
      if (bx !== ax && by !== ay) {
        setLink(bx, ay, CROSS); // 转角
      }
      const stepY = Math.sign(by - ay);
      for (let cy = ay + stepY; cy !== by && stepY !== 0; cy += stepY) {
        setLink(bx, cy, VERTICAL);
      }
    }
  }

  // 放置节点字形（覆盖连线）。
  for (const node of nodes) {
    const pos = nodePositions[node.id];
    grid[pos.row][pos.col] = { ch: node.glyph, type: "node", nodeId: node.id };
  }

  const cells: AsciiCell[] = [];
  for (let row = 0; row < rows; row += 1) {
    for (let col = 0; col < columns; col += 1) {
      const slot = grid[row][col];
      cells.push({ col, row, ch: slot.ch, type: slot.type, nodeId: slot.nodeId });
    }
  }

  return { columns, rows, cells, nodePositions };
}

/** 键盘导航：从当前节点出发，找指定方向上最近的节点。 */
export function nearestNodeInDirection(
  nodes: MapNode[],
  fromId: string | undefined,
  dir: Direction,
): string | undefined {
  const from = nodes.find((n) => n.id === fromId) ?? nodes[0];
  if (!from) {
    return undefined;
  }
  const candidates = nodes.filter((n) => {
    if (n.id === from.id) {
      return false;
    }
    switch (dir) {
      case "up":
        return n.y < from.y;
      case "down":
        return n.y > from.y;
      case "left":
        return n.x < from.x;
      case "right":
        return n.x > from.x;
      default:
        return false;
    }
  });
  if (candidates.length === 0) {
    return undefined;
  }
  const score = (n: MapNode) => {
    const dx = n.x - from.x;
    const dy = n.y - from.y;
    // 主轴权重更高，鼓励沿方向移动。
    const along = dir === "left" || dir === "right" ? Math.abs(dx) : Math.abs(dy);
    const cross = dir === "left" || dir === "right" ? Math.abs(dy) : Math.abs(dx);
    return along + cross * 2;
  };
  candidates.sort((a, b) => score(a) - score(b));
  return candidates[0]?.id;
}
