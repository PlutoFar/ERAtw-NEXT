import { invoke } from "@tauri-apps/api/core";
import { applyDemoCommand, createDemoWorld } from "./demoWorld";
import type { EngineCommand, WorldState } from "../types";

export interface EngineClient {
  snapshot(): Promise<WorldState>;
  dispatch(command: EngineCommand): Promise<WorldState>;
}

const isTauriRuntime = () =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const createTauriEngineClient = (): EngineClient => ({
  snapshot: () => invoke<WorldState>("engine_snapshot"),
  dispatch: (command) => invoke<WorldState>("engine_dispatch", { command }),
});

export const createBrowserMockEngineClient = (): EngineClient => {
  let world = createDemoWorld();

  return {
    async snapshot() {
      return structuredClone(world);
    },
    async dispatch(command) {
      world = applyDemoCommand(world, command);
      return structuredClone(world);
    },
  };
};

export const createEngineClient = (): EngineClient =>
  isTauriRuntime() ? createTauriEngineClient() : createBrowserMockEngineClient();
