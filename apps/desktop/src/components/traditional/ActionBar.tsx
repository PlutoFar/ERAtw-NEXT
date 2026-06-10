import { Clock3, MessageCircle, Move, RefreshCw, Sparkles, TerminalSquare } from "lucide-react";
import type { Location } from "../../types";
import { locationName } from "./viewModel";

interface ActionBarProps {
  commandPanelOpen: boolean;
  inspectedLocation: Location | undefined;
  loading: boolean;
  onAdjustRelationship: () => void;
  onOpenMoveMap: () => void;
  onRest: () => void;
  onRollMood: () => void;
  onStartDialogue: () => void;
  onToggleCommandPanel: () => void;
}

export const ActionBar = ({
  commandPanelOpen,
  inspectedLocation,
  loading,
  onAdjustRelationship,
  onOpenMoveMap,
  onRest,
  onRollMood,
  onStartDialogue,
  onToggleCommandPanel,
}: ActionBarProps) => {
  return (
    <nav className="action-bar" aria-label="quick actions">
      <button type="button" onClick={onRest} disabled={loading}>
        <Clock3 size={17} aria-hidden="true" /> 休息
      </button>
      <button type="button" onClick={onStartDialogue} disabled={loading}>
        <MessageCircle size={17} aria-hidden="true" /> 对话
      </button>
      <button type="button" onClick={onOpenMoveMap} disabled={loading}>
        <Move size={17} aria-hidden="true" /> 移动
      </button>
      <button type="button" onClick={onAdjustRelationship} disabled={loading}>
        <Sparkles size={17} aria-hidden="true" /> 交流
      </button>
      <button type="button" onClick={onToggleCommandPanel} aria-expanded={commandPanelOpen}>
        <TerminalSquare size={17} aria-hidden="true" /> 命令
      </button>

      {commandPanelOpen ? (
        <div className="command-popover" aria-label="command panel">
          <strong>命令面板</strong>
          <button type="button" onClick={onRollMood} disabled={loading}>
            <RefreshCw size={16} aria-hidden="true" /> 随机心情
          </button>
          <span>
            目标：{inspectedLocation ? locationName(inspectedLocation) : "未选择地点"}
          </span>
        </div>
      ) : null}
    </nav>
  );
};
