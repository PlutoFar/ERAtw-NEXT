import { invoke } from "@tauri-apps/api/core";
import { applyDemoCommand, createDemoWorld } from "./demoWorld";
import type { EngineCommand, SaveEnvelope, WorldState } from "../types";

export interface EngineClient {
  snapshot(): Promise<WorldState>;
  dispatch(command: EngineCommand): Promise<WorldState>;
  savePreview(slotId: string, savedAtUnixMs: number): Promise<SaveEnvelope>;
}

const isTauriRuntime = () =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const createTauriEngineClient = (): EngineClient => ({
  snapshot: () => invoke<WorldState>("engine_snapshot"),
  dispatch: (command) => invoke<WorldState>("engine_dispatch", { command }),
  savePreview: (slotId, savedAtUnixMs) =>
    invoke<SaveEnvelope>("engine_save_preview", {
      slotId,
      savedAtUnixMs,
    }),
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
    async savePreview(slotId, savedAtUnixMs) {
      return {
        schema_version: 1,
        engine_version: world.engine_version,
        saved_at_unix_ms: savedAtUnixMs,
        slot_id: slotId,
        mod_dependencies: [],
        world: structuredClone(world),
      };
    },
  };
};

export const createEngineClient = (): EngineClient =>
  isTauriRuntime() ? createTauriEngineClient() : createBrowserMockEngineClient();
