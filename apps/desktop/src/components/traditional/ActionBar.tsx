import { Clock3, ListChecks, Map as MapIcon, MessageCircle, Move, Sparkles } from "lucide-react";

interface ActionBarProps {
  commandPanelOpen: boolean;
  loading: boolean;
  onAdjustRelationship: () => void;
  onOpenMap: () => void;
  onOpenMoveMap: () => void;
  onRest: () => void;
  onStartDialogue: () => void;
  onToggleCommandPanel: () => void;
}

export const ActionBar = ({
  commandPanelOpen,
  loading,
  onAdjustRelationship,
  onOpenMap,
  onOpenMoveMap,
  onRest,
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
      <button type="button" onClick={onOpenMap} disabled={loading}>
        <MapIcon size={17} aria-hidden="true" /> 地图
      </button>
      <button type="button" onClick={onAdjustRelationship} disabled={loading}>
        <Sparkles size={17} aria-hidden="true" /> 交流
      </button>
      <button type="button" onClick={onToggleCommandPanel} aria-expanded={commandPanelOpen}>
        <ListChecks size={17} aria-hidden="true" /> 命令
      </button>
    </nav>
  );
};
