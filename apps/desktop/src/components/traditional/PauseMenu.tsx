import { DoorOpen, Play, Save, Settings } from "lucide-react";
import { useState } from "react";
import type { ShellServices } from "./shellTypes";
import { ModPanel, SaveLoadPanel, SettingsPanel, StatusMessages } from "./ShellPanels";

type PausePanel = "save" | "load" | "mod" | "settings";

interface PauseMenuProps {
  onResume: () => void;
  onReturnTitle: () => void;
  open: boolean;
  services: ShellServices;
}

export const PauseMenu = ({
  onResume,
  onReturnTitle,
  open,
  services,
}: PauseMenuProps) => {
  const [panel, setPanel] = useState<PausePanel>("save");

  if (!open) {
    return null;
  }

  return (
    <section className="pause-overlay" aria-label="pause menu">
      <div className="pause-menu">
        <div className="pause-menu-list" aria-label="pause actions">
          <strong>暂停</strong>
          <button type="button" onClick={onResume}>
            <Play size={17} aria-hidden="true" /> 继续游戏
          </button>
          <button type="button" onClick={() => setPanel("save")}>
            <Save size={17} aria-hidden="true" /> 保存
          </button>
          <button type="button" onClick={() => setPanel("load")}>
            读取
          </button>
          <button type="button" onClick={() => setPanel("mod")}>
            Mod
          </button>
          <button type="button" onClick={() => setPanel("settings")}>
            <Settings size={17} aria-hidden="true" /> 设置
          </button>
          <button type="button" onClick={onReturnTitle}>
            <DoorOpen size={17} aria-hidden="true" /> 返回标题
          </button>
        </div>

        <div className="pause-panel">
          {panel === "save" || panel === "load" ? (
            <SaveLoadPanel services={services} />
          ) : null}
          {panel === "mod" ? <ModPanel services={services} /> : null}
          {panel === "settings" ? <SettingsPanel /> : null}
          <StatusMessages services={services} />
        </div>
      </div>
    </section>
  );
};
