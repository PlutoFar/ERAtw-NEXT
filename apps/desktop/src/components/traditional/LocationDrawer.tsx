import { LocateFixed, Move, Pin, X } from "lucide-react";
import type { Location, TextMap, WorldState } from "../../types";
import {
  areaName,
  characterName,
  charactersAtLocation,
  locationName,
  terrainName,
} from "./viewModel";

interface LocationDrawerProps {
  currentLocation: Location | undefined;
  loading: boolean;
  location: Location | undefined;
  onClose: () => void;
  onMove: (locationId: string) => void;
  onPin: (locationId: string) => void;
  onSwitchArea: (location: Location | undefined) => void;
  pinnedLocationId: string | undefined;
  textMap: TextMap | undefined;
  world: WorldState;
}

export const LocationDrawer = ({
  currentLocation,
  loading,
  location,
  onClose,
  onMove,
  onPin,
  onSwitchArea,
  pinnedLocationId,
  textMap,
  world,
}: LocationDrawerProps) => {
  if (!location) {
    return null;
  }

  const occupants = charactersAtLocation(world, location.id);
  const isCurrent = location.id === currentLocation?.id;
  const isPinned = pinnedLocationId === location.id;

  return (
    <aside className="location-drawer" aria-label="location details">
      <div className="panel-heading">
        <div>
          <span className="panel-kicker">{areaName(textMap, location.map_area_id)}</span>
          <h2>{locationName(location)}</h2>
        </div>
        <button type="button" className="icon-button" onClick={onClose} aria-label="关闭地点详情">
          <X size={17} aria-hidden="true" />
        </button>
      </div>

      <dl className="compact-dl">
        <div>
          <dt>地形</dt>
          <dd>{terrainName(location.terrain)}</dd>
        </div>
        <div>
          <dt>移动</dt>
          <dd>{isCurrent ? "已在此处" : `${location.move_minutes ?? 10} 分钟`}</dd>
        </div>
        <div>
          <dt>人物</dt>
          <dd>
            {occupants.length > 0
              ? occupants.map((character) => characterName(character)).join("、")
              : "无"}
          </dd>
        </div>
      </dl>

      <div className="drawer-actions">
        <button
          type="button"
          onClick={() => onMove(location.id)}
          disabled={loading || isCurrent}
        >
          <Move size={16} aria-hidden="true" /> 移动
        </button>
        <button
          type="button"
          onClick={() => onSwitchArea(location)}
          disabled={!location.map_area_id}
        >
          <LocateFixed size={16} aria-hidden="true" /> 切区
        </button>
        <button type="button" onClick={() => onPin(location.id)}>
          <Pin size={16} aria-hidden="true" /> {isPinned ? "取消关注" : "关注"}
        </button>
      </div>
    </aside>
  );
};
