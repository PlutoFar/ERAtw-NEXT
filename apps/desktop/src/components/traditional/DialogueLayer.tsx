import { X } from "lucide-react";
import { visibleChoices } from "../../engine/demoWorld";
import type { EngineCommand, WorldState } from "../../types";
import {
  canRenderImagePath,
  characterName,
  findPortrait,
} from "./viewModel";

interface DialogueLayerProps {
  dispatch: (command: EngineCommand) => void | Promise<void>;
  loading: boolean;
  onClose: () => void;
  open: boolean;
  world: WorldState;
}

export const DialogueLayer = ({
  dispatch,
  loading,
  onClose,
  open,
  world,
}: DialogueLayerProps) => {
  if (!open || world.active_dialogue.length === 0) {
    return null;
  }

  const latestNode = world.active_dialogue[world.active_dialogue.length - 1];
  const speaker = world.characters.find((character) => character.id === latestNode.speaker_id);
  const referencedPortrait = world.resources.find((resource) =>
    latestNode.resource_refs.includes(resource.resource_id),
  );
  const portrait = referencedPortrait ?? findPortrait(world.resources, speaker?.id);
  const canRenderPortrait = canRenderImagePath(portrait?.source_path);
  const choices = world.active_dialogue.flatMap((node) =>
    visibleChoices(world, node).map((choice) => ({ node, choice })),
  );

  return (
    <section className="dialogue-layer" aria-label="dialogue layer">
      <div className="dialogue-stage">
        <div className="dialogue-portrait" aria-label="dialogue portrait">
          {canRenderPortrait && portrait ? (
            <img src={portrait.source_path} alt={`${characterName(speaker, latestNode.speaker_id)} 立绘`} />
          ) : (
            <div className="portrait-fallback">
              <span>{Array.from(characterName(speaker, latestNode.speaker_id))[0] ?? "话"}</span>
              <small>{portrait ? portrait.resource_id : latestNode.speaker_id}</small>
            </div>
          )}
        </div>

        <div className="dialogue-box">
          <div className="panel-heading">
            <div>
              <span className="panel-kicker">Dialogue</span>
              <h2>{characterName(speaker, latestNode.speaker_id)}</h2>
            </div>
            <button type="button" className="icon-button" onClick={onClose} aria-label="关闭对话">
              <X size={17} aria-hidden="true" />
            </button>
          </div>
          <p>{latestNode.text}</p>
          {choices.length > 0 ? (
            <div className="choice-list">
              {choices.map(({ node, choice }) => (
                <button
                  key={`${node.id}:${choice.id}`}
                  type="button"
                  onClick={() =>
                    dispatch({
                      type: "choose_dialogue",
                      node_id: node.id,
                      choice_id: choice.id,
                    })
                  }
                  disabled={loading}
                >
                  {choice.label}
                </button>
              ))}
            </div>
          ) : null}
        </div>
      </div>
    </section>
  );
};
