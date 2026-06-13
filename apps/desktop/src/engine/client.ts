// 引擎客户端：在 Tauri 中调用真实 command，在浏览器/测试中回退到镜像数据。

import { invoke } from "@tauri-apps/api/core";
import type {
  CommandResult,
  ContentPackageIndex,
  GameCommand,
  GameState,
  MapModel,
  SaveReport,
  SystemStatus,
} from "../types";
import {
  applyMockCommand,
  loadMockGame,
  mockContentPackageIndex,
  mockInitialGameState,
  mockMapModel,
  mockSystemStatus,
} from "./mockData";

export interface EngineClient {
  getSystemStatus(): Promise<SystemStatus>;
  getMapOverview(): Promise<MapModel>;
  loadContentPackage(path: string): Promise<ContentPackageIndex>;
  getLoadedContent(): Promise<ContentPackageIndex | null>;
  newGame(): Promise<GameState>;
  getGameState(): Promise<GameState | null>;
  applyGameCommand(command: GameCommand): Promise<CommandResult>;
  writeSave(path: string): Promise<SaveReport>;
  loadSave(path: string): Promise<GameState>;
  chooseContentPackageDirectory(): Promise<string | null>;
  chooseSavePath(defaultPath?: string): Promise<string | null>;
  chooseLoadSavePath(defaultPath?: string): Promise<string | null>;
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

let browserPackage: ContentPackageIndex | null = null;
let browserGame: GameState | null = null;

async function loadContentPackage(path: string): Promise<ContentPackageIndex> {
  if (runningInTauri()) {
    return invoke<ContentPackageIndex>("content_load_package", { path });
  }
  browserPackage = { ...mockContentPackageIndex, rootPath: path };
  browserGame = null;
  return browserPackage;
}

async function getLoadedContent(): Promise<ContentPackageIndex | null> {
  if (runningInTauri()) {
    return invoke<ContentPackageIndex | null>("content_get_loaded");
  }
  return browserPackage;
}

async function newGame(): Promise<GameState> {
  if (runningInTauri()) {
    return invoke<GameState>("game_new");
  }
  if (!browserPackage) {
    throw { code: "CONTENT_NOT_LOADED", message: "请先加载内容包。", details: {} };
  }
  browserGame = structuredClone(mockInitialGameState);
  return browserGame;
}

async function getGameState(): Promise<GameState | null> {
  if (runningInTauri()) {
    return invoke<GameState | null>("game_get_state");
  }
  return browserGame;
}

async function applyGameCommand(command: GameCommand): Promise<CommandResult> {
  if (runningInTauri()) {
    return invoke<CommandResult>("game_apply_command", { command });
  }
  if (!browserGame) {
    throw { code: "GAME_NOT_STARTED", message: "请先开始游戏。", details: {} };
  }
  const result = applyMockCommand(browserGame, command);
  browserGame = result.state;
  return result;
}

async function writeSave(path: string): Promise<SaveReport> {
  if (runningInTauri()) {
    return invoke<SaveReport>("save_write", { path });
  }
  if (!browserGame) {
    throw { code: "GAME_NOT_STARTED", message: "请先开始游戏。", details: {} };
  }
  return {
    schemaVersion: "save-report/v1",
    path,
    packageId: browserGame.package.packageId,
    turn: browserGame.turn,
    bytes: JSON.stringify(browserGame).length,
    stateHash: "sha256:browser-preview",
  };
}

async function loadSave(path: string): Promise<GameState> {
  if (runningInTauri()) {
    return invoke<GameState>("save_load", { path });
  }
  browserGame = loadMockGame();
  return browserGame;
}

async function chooseContentPackageDirectory(): Promise<string | null> {
  if (!runningInTauri()) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({ directory: true, multiple: false });
  return typeof selected === "string" ? selected : null;
}

async function chooseSavePath(defaultPath?: string): Promise<string | null> {
  if (!runningInTauri()) return null;
  const { save } = await import("@tauri-apps/plugin-dialog");
  return save({
    defaultPath,
    filters: [{ name: "ERAtw-NEXT Save", extensions: ["json"] }],
  });
}

async function chooseLoadSavePath(defaultPath?: string): Promise<string | null> {
  if (!runningInTauri()) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({
    directory: false,
    multiple: false,
    defaultPath,
    filters: [{ name: "ERAtw-NEXT Save", extensions: ["json"] }],
  });
  return typeof selected === "string" ? selected : null;
}

export const defaultEngineClient: EngineClient = {
  getSystemStatus,
  getMapOverview,
  loadContentPackage,
  getLoadedContent,
  newGame,
  getGameState,
  applyGameCommand,
  writeSave,
  loadSave,
  chooseContentPackageDirectory,
  chooseSavePath,
  chooseLoadSavePath,
};
