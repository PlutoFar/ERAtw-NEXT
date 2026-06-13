// 与 schemas/ 下 JSON Schema 契约一一对应的前端类型。

export type BuildProfile = "debug" | "release";
export type PathKind = "read_only" | "reference" | "excluded";
export type CapabilityStatus = "available" | "planned" | "disabled";
export type MilestoneStatus = "done" | "in_progress" | "planned";

export interface SystemStatus {
  schemaVersion: "system-status/v1";
  app: { name: string; stage: string; tagline: string };
  engine: { name: string; version: string };
  build: { profile: BuildProfile; gitDescribe: string | null; timestamp: string | null };
  paths: PathPlaceholder[];
  capabilities: Capability[];
  currentMilestone: string;
  milestones: Milestone[];
}

export interface PathPlaceholder {
  id: string;
  label: string;
  value: string;
  kind: PathKind;
  note: string;
}

export interface Capability {
  id: string;
  label: string;
  status: CapabilityStatus;
  description: string;
}

export interface Milestone {
  id: string;
  title: string;
  status: MilestoneStatus;
  summary: string;
}

// ===== 地图模型 (map-model/v1) =====

export type ActivityKey = "staying" | "working" | "sleeping" | "passing" | "free";
export type NodeKind = "home" | "shop" | "shrine" | "landmark" | "gate" | "public" | "nature";

export interface MapModel {
  schemaVersion: "map-model/v1";
  defaultAreaId: string;
  grid: { columns: number; rows: number };
  areas: MapArea[];
  legend: LegendEntry[];
  nodes: MapNode[];
}

export interface MapArea {
  id: string;
  label: string;
  description: string;
}

export interface LegendEntry {
  key: ActivityKey;
  label: string;
  glyph: string;
  color: string;
}

export interface MapNode {
  id: string;
  areaId: string;
  label: string;
  kind: NodeKind;
  glyph: string;
  x: number;
  y: number;
  terrain: string;
  moveMinutes: number;
  note: string;
  links: string[];
  occupants: Occupant[];
}

export interface Occupant {
  id: string;
  label: string;
  activity: ActivityKey;
}

// 引擎错误的稳定结构。
export interface EngineError {
  code: string;
  message: string;
  details: Record<string, unknown>;
}
