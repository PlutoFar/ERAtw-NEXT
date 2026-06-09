import { create } from "zustand";
import { createEngineClient, type EngineClient } from "./client";
import type { EngineCommand, WorldState } from "../types";

interface EngineStore {
  client: EngineClient;
  world: WorldState | null;
  loading: boolean;
  error: string | null;
  load: () => Promise<void>;
  dispatch: (command: EngineCommand) => Promise<void>;
}

export const useEngine = create<EngineStore>((set, get) => ({
  client: createEngineClient(),
  world: null,
  loading: false,
  error: null,
  async load() {
    set({ loading: true, error: null });
    try {
      const world = await get().client.snapshot();
      set({ world, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },
  async dispatch(command) {
    set({ loading: true, error: null });
    try {
      const world = await get().client.dispatch(command);
      set({ world, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },
}));
