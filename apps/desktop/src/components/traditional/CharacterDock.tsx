import { MessageCircle, RefreshCw, Sparkles, UserRound, X } from "lucide-react";
import type { Character, Location, Relationship, WorldState } from "../../types";
import {
  canRenderImagePath,
  characterName,
  charactersAtLocation,
  findPortrait,
  locationName,
} from "./viewModel";

interface CharacterDockProps {
  currentLocation: Location | undefined;
  loading: boolean;
  onAdjustRelationship: () => void;
  onClose: () => void;
  onRollMood: () => void;
  onStartDialogue: () => void;
  onSelectCharacter: (characterId: string) => void;
  relationship: Relationship | undefined;
  selectedCharacter: Character | undefined;
  world: WorldState;
}

export const CharacterDock = ({
  currentLocation,
  loading,
  onAdjustRelationship,
  onClose,
  onRollMood,
  onStartDialogue,
  onSelectCharacter,
  relationship,
  selectedCharacter,
  world,
}: CharacterDockProps) => {
  const occupants = charactersAtLocation(world, currentLocation?.id);
  const portrait = findPortrait(world.resources, selectedCharacter?.id);
  const canRenderPortrait = canRenderImagePath(portrait?.source_path);

  return (
    <section className="character-dock" aria-label="current location characters">
      <div className="panel-heading">
        <div>
          <span className="panel-kicker">{locationName(currentLocation)}</span>
          <h2>
            <UserRound size={17} aria-hidden="true" /> 人物
          </h2>
        </div>
        <button type="button" className="icon-button" onClick={onClose} aria-label="关闭人物面板">
          <X size={17} aria-hidden="true" />
        </button>
      </div>

      {occupants.length > 0 ? (
        <>
          <div className="dock-character-list">
            {occupants.map((character) => (
              <button
                key={character.id}
                type="button"
                className={character.id === selectedCharacter?.id ? "current" : undefined}
                onClick={() => onSelectCharacter(character.id)}
                disabled={loading}
                aria-pressed={character.id === selectedCharacter?.id}
              >
                {characterName(character)}
              </button>
            ))}
          </div>

          {selectedCharacter ? (
            <div className="dock-character-detail">
              <div className="portrait-frame" aria-label="character portrait">
                {canRenderPortrait && portrait ? (
                  <img src={portrait.source_path} alt={`${characterName(selectedCharacter)} 立绘`} />
                ) : (
                  <div className="portrait-fallback">
                    <span>{Array.from(characterName(selectedCharacter))[0] ?? "人"}</span>
                    <small>
                      {portrait ? portrait.resource_id : "未绑定立绘"}
                    </small>
                  </div>
                )}
              </div>
              <dl className="compact-dl">
                <div>
                  <dt>姓名</dt>
                  <dd>{characterName(selectedCharacter)}</dd>
                </div>
                <div>
                  <dt>体力</dt>
                  <dd>{selectedCharacter.state.energy}</dd>
                </div>
                <div>
                  <dt>心情</dt>
                  <dd>{selectedCharacter.state.mood}</dd>
                </div>
                <div>
                  <dt>好感</dt>
                  <dd>{relationship?.affinity ?? "未知"}</dd>
                </div>
              </dl>
              <div className="dock-actions">
                <button type="button" onClick={onStartDialogue} disabled={loading}>
                  <MessageCircle size={16} aria-hidden="true" /> 对话
                </button>
                <button type="button" onClick={onAdjustRelationship} disabled={loading}>
                  <Sparkles size={16} aria-hidden="true" /> 交流
                </button>
                <button type="button" onClick={onRollMood} disabled={loading}>
                  <RefreshCw size={16} aria-hidden="true" /> 心情
                </button>
              </div>
            </div>
          ) : null}
        </>
      ) : (
        <p className="empty-text">此处暂无人物。</p>
      )}
    </section>
  );
};
