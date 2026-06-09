import { create } from "zustand";
import { createEngineClient, type EngineClient } from "./client";
import type { ContentPackage, EngineCommand, SaveSlotReport, WorldState } from "../types";

interface EngineStore {
  client: EngineClient;
  world: WorldState | null;
  loading: boolean;
  error: string | null;
  lastSave: SaveSlotReport | null;
  load: () => Promise<void>;
  dispatch: (command: EngineCommand) => Promise<void>;
  installContentPackage: (packageData: ContentPackage) => Promise<void>;
  saveSlot: (slotId: string) => Promise<void>;
  loadSlot: (slotId: string) => Promise<void>;
}

export const useEngine = create<EngineStore>((set, get) => ({
  client: createEngineClient(),
  world: null,
  loading: false,
  error: null,
  lastSave: null,
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
  async installContentPackage(packageData) {
    set({ loading: true, error: null });
    try {
      const world = await get().client.installContentPackage(packageData);
      set({ world, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },
  async saveSlot(slotId) {
    set({ loading: true, error: null });
    try {
      const lastSave = await get().client.saveSlot(slotId, Date.now());
      set({ lastSave, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },
  async loadSlot(slotId) {
    set({ loading: true, error: null });
    try {
      const world = await get().client.loadSlot(slotId);
      set({ world, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },
}));
