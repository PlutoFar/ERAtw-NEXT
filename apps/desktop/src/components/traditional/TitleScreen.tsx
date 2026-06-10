import { Boxes, Play, Settings } from "lucide-react";
import { useState } from "react";
import type { ShellServices } from "./shellTypes";
import { ModPanel, SaveLoadPanel, SettingsPanel, StatusMessages } from "./ShellPanels";

type TitlePanel = "main" | "load" | "mod" | "settings";

interface TitleScreenProps {
  onEnterGame: () => void;
  services: ShellServices;
}

export const TitleScreen = ({ onEnterGame, services }: TitleScreenProps) => {
  const [panel, setPanel] = useState<TitlePanel>("main");

  const startNewGame = () => {
    void services.loadNewWorld();
    onEnterGame();
  };

  return (
    <section className="title-screen" aria-label="title screen">
      <div className="title-mark">
        <span>eraTheWorld TW NEXT</span>
        <h1>ERAtw-NEXT</h1>
        <p>ASCII MAP / MODERN GAME UI</p>
      </div>

      <nav className="title-menu" aria-label="title menu">
        <button type="button" onClick={onEnterGame} disabled={services.loading}>
          <Play size={18} aria-hidden="true" /> 继续
        </button>
        <button type="button" onClick={startNewGame} disabled={services.loading}>
          开始
        </button>
        <button type="button" onClick={() => setPanel("load")}>
          读取
        </button>
        <button type="button" onClick={() => setPanel("mod")}>
          <Boxes size={18} aria-hidden="true" /> Mod
        </button>
        <button type="button" onClick={() => setPanel("settings")}>
          <Settings size={18} aria-hidden="true" /> 设置
        </button>
      </nav>

      <div className="title-panel">
        {panel === "main" ? <StatusMessages services={services} /> : null}
        {panel === "load" ? <SaveLoadPanel services={services} /> : null}
        {panel === "mod" ? <ModPanel services={services} /> : null}
        {panel === "settings" ? <SettingsPanel /> : null}
        {panel !== "main" ? <StatusMessages services={services} /> : null}
      </div>
    </section>
  );
};
