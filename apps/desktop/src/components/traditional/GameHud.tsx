import { Clock3, CloudSun, MapPinned, Pause, UserRound } from "lucide-react";
import type { Character, Location, TextMap, WorldState } from "../../types";
import {
  characterName,
  charactersAtLocation,
  formatClock,
  locationName,
  seasonLabels,
  weatherLabels,
} from "./viewModel";

interface GameHudProps {
  currentLocation: Location | undefined;
  onOpenCharacters: () => void;
  onPause: () => void;
  playerCharacter: Character | undefined;
  selectedCharacter: Character | undefined;
  textMap: TextMap | undefined;
  world: WorldState;
}

export const GameHud = ({
  currentLocation,
  onOpenCharacters,
  onPause,
  playerCharacter,
  selectedCharacter,
  textMap,
  world,
}: GameHudProps) => {
  const occupants = charactersAtLocation(world, currentLocation?.id);

  return (
    <header className="game-hud" aria-label="game hud">
      <div className="hud-brand">
        <strong>ERAtw-NEXT</strong>
        <span>{textMap ? textMap.name : "地图未载入"}</span>
      </div>
      <div className="hud-strip">
        <span>
          <Clock3 size={16} aria-hidden="true" /> {formatClock(world)}
        </span>
        <span>
          <CloudSun size={16} aria-hidden="true" /> {seasonLabels[world.clock.season]} ·{" "}
          {weatherLabels[world.clock.weather]}
        </span>
        <span>
          <MapPinned size={16} aria-hidden="true" /> {locationName(currentLocation)}
        </span>
        <button
          type="button"
          className="hud-status-button"
          onClick={onOpenCharacters}
          aria-label="打开人物面板"
        >
          <UserRound size={16} aria-hidden="true" />{" "}
          {selectedCharacter ? characterName(selectedCharacter) : `${occupants.length} 人`}
        </button>
        <span>
          体力 {playerCharacter?.state.energy ?? "--"} / 心情{" "}
          {playerCharacter?.state.mood ?? "--"}
        </span>
      </div>
      <button type="button" className="hud-pause" onClick={onPause} aria-label="暂停菜单">
        <Pause size={17} aria-hidden="true" /> ESC
      </button>
    </header>
  );
};
