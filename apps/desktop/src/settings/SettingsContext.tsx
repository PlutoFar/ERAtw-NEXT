import { createContext, useContext, useMemo, useState } from "react";
import type { ReactNode } from "react";

export type MapRenderMode = "ascii" | "svg";

export interface Settings {
  mapRenderMode: MapRenderMode;
  setMapRenderMode: (mode: MapRenderMode) => void;
}

const SettingsContext = createContext<Settings | undefined>(undefined);

interface SettingsProviderProps {
  children: ReactNode;
  initialMapRenderMode?: MapRenderMode;
}

export function SettingsProvider({
  children,
  initialMapRenderMode = "svg",
}: SettingsProviderProps) {
  const [mapRenderMode, setMapRenderMode] = useState<MapRenderMode>(initialMapRenderMode);

  const value = useMemo<Settings>(
    () => ({ mapRenderMode, setMapRenderMode }),
    [mapRenderMode],
  );

  return <SettingsContext.Provider value={value}>{children}</SettingsContext.Provider>;
}

export function useSettings(): Settings {
  const ctx = useContext(SettingsContext);
  if (!ctx) {
    throw new Error("useSettings 必须在 SettingsProvider 内使用");
  }
  return ctx;
}
