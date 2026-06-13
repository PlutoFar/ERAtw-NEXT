// 引擎客户端：在 Tauri 中调用真实 command，在浏览器/测试中回退到镜像数据。

import { invoke } from "@tauri-apps/api/core";
import type { MapModel, SystemStatus } from "../types";
import { mockMapModel, mockSystemStatus } from "./mockData";

export interface EngineClient {
  getSystemStatus(): Promise<SystemStatus>;
  getMapOverview(): Promise<MapModel>;
}

const runningInTauri = (): boolean =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function getSystemStatus(): Promise<SystemStatus> {
  if (runningInTauri()) {
    return invoke<SystemStatus>("system_get_status");
  }
  return mockSystemStatus;
}

async function getMapOverview(): Promise<MapModel> {
  if (runningInTauri()) {
    return invoke<MapModel>("map_get_overview");
  }
  return mockMapModel;
}

export const defaultEngineClient: EngineClient = {
  getSystemStatus,
  getMapOverview,
};
