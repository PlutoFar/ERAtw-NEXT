import { useMemo, useState } from "react";
import type { MouseEvent } from "react";
import {
  LocateFixed,
  MapPinned,
  Move,
  Pin,
  Search,
  ZoomIn,
  ZoomOut,
} from "lucide-react";
import { displayText } from "../../engine/displayText";
import type { Location, TextMapAction, WorldState } from "../../types";
import { ActionBar } from "./ActionBar";
import { AsciiMapViewport } from "./AsciiMapViewport";
import { CharacterDock } from "./CharacterDock";
import { DialogueLayer } from "./DialogueLayer";
import { GameHud } from "./GameHud";
import { LocationDrawer } from "./LocationDrawer";
import type { ShellServices } from "./shellTypes";
import {
  areaName,
  characterName,
  charactersAtLocation,
  locationName,
  locationSymbol,
  terrainName,
  visibleLocationsForTextMap,
} from "./viewModel";

interface ContextMenuState {
  locationId: string;
  x: number;
  y: number;
}

interface GameScreenProps {
  onPause: () => void;
  services: ShellServices;
  world: WorldState;
}

export const GameScreen = ({ onPause, services, world }: GameScreenProps) => {
  const playerCharacter = world.characters[0];
  const currentLocation = world.locations.find(
    (location) => location.id === playerCharacter?.location_id,
  );
  const textMap = world.text_maps[0];
  const visibleLocations = useMemo(
    () => visibleLocationsForTextMap(world, textMap),
    [textMap, world],
  );
  const [selectedAreaId, setSelectedAreaId] = useState<string | undefined>();
  const [inspectedLocationId, setInspectedLocationId] = useState<string | undefined>();
  const [hoveredLocationId, setHoveredLocationId] = useState<string | undefined>();
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const [pinnedLocationId, setPinnedLocationId] = useState<string | undefined>();
  const [selectedCharacterId, setSelectedCharacterId] = useState<string | undefined>();
  const [dismissedDialogueToken, setDismissedDialogueToken] = useState<string | null>(
    null,
  );
  const [commandPanelOpen, setCommandPanelOpen] = useState(false);
  const [locationSearch, setLocationSearch] = useState("");
  const [zoom, setZoom] = useState(1);

  const activeAreaId =
    selectedAreaId ??
    currentLocation?.map_area_id ??
    textMap?.default_area_id ??
    textMap?.areas[0]?.id;
  const activeArea =
    textMap?.areas.find((area) => area.id === activeAreaId) ?? textMap?.areas[0];
  const inspectedLocation = world.locations.find(
    (location) => location.id === inspectedLocationId,
  );
  const hoveredLocation = world.locations.find(
    (location) => location.id === hoveredLocationId,
  );
  const contextLocation = world.locations.find(
    (location) => location.id === contextMenu?.locationId,
  );
  const currentLocationCharacters = charactersAtLocation(world, currentLocation?.id);
  const selectedCharacter =
    currentLocationCharacters.find((character) => character.id === selectedCharacterId) ??
    currentLocationCharacters[0] ??
    playerCharacter;
  const selectedCharacterRelationship = world.relationships.find(
    (item) =>
      item.source_character_id === "player" &&
      item.target_character_id === selectedCharacter?.id,
  );
  const filteredLocations = visibleLocations.filter((location) => {
    const query = locationSearch.trim();
    if (!query) {
      return true;
    }
    return (
      locationName(location).includes(query) ||
      location.id.includes(query) ||
      String(location.legacy_place_id ?? "").includes(query)
    );
  });
  const dialogueToken =
    world.active_dialogue.length > 0
      ? `${world.active_dialogue_scene_id ?? "dialogue"}:${
          world.active_dialogue[0]?.id ?? "entry"
        }:${world.active_dialogue.length}`
      : null;
  const dialogueOpen =
    dialogueToken !== null && dismissedDialogueToken !== dialogueToken;

  const inspectLocation = (locationId: string) => {
    const location = world.locations.find((item) => item.id === locationId);
    setInspectedLocationId(locationId);
    setContextMenu(null);
    if (location?.map_area_id) {
      setSelectedAreaId(location.map_area_id);
    }
  };

  const moveTo = (locationId: string) => {
    if (!playerCharacter || services.loading) {
      return;
    }
    const destination = world.locations.find((location) => location.id === locationId);
    setInspectedLocationId(locationId);
    setContextMenu(null);
    if (destination?.map_area_id) {
      setSelectedAreaId(destination.map_area_id);
    }
    if (destination?.id === playerCharacter.location_id) {
      return;
    }
    void services.dispatch({
      type: "move_character",
      character_id: playerCharacter.id,
      location_id: locationId,
    });
  };

  const runMapAction = (action: TextMapAction) => {
    if (action.type === "switch_area") {
      setSelectedAreaId(action.area_id);
      setContextMenu(null);
      return;
    }
    if (action.type === "back") {
      setSelectedAreaId(currentLocation?.map_area_id ?? textMap?.default_area_id);
      setContextMenu(null);
    }
  };

  const startDialogue = () => {
    setDismissedDialogueToken(null);
    void services.dispatch({ type: "start_dialogue", scene_id: "demo_morning" });
  };

  const adjustRelationship = () => {
    if (!selectedCharacter) {
      return;
    }
    void services.dispatch({
      type: "adjust_relationship",
      source_character_id: "player",
      target_character_id: selectedCharacter.id,
      affinity_delta: 1,
      trust_delta: 1,
    });
  };

  const rollMood = () => {
    if (!selectedCharacter) {
      return;
    }
    void services.dispatch({
      type: "roll_character_mood",
      character_id: selectedCharacter.id,
      min_delta: -5,
      max_delta: 5,
    });
  };

  const openContextMenu = (
    event: MouseEvent<HTMLButtonElement>,
    locationId: string,
  ) => {
    event.preventDefault();
    event.stopPropagation();
    setInspectedLocationId(locationId);
    setContextMenu({ locationId, x: event.clientX, y: event.clientY });
  };

  const pinLocation = (locationId: string) => {
    setPinnedLocationId((current) => (current === locationId ? undefined : locationId));
    setContextMenu(null);
  };

  const switchAreaFromLocation = (location: Location | undefined) => {
    if (location?.map_area_id) {
      setSelectedAreaId(location.map_area_id);
    }
    setContextMenu(null);
  };

  const focusPeopleAtLocation = (location: Location | undefined) => {
    const firstOccupant = charactersAtLocation(world, location?.id)[0];
    if (firstOccupant) {
      setSelectedCharacterId(firstOccupant.id);
    }
    setContextMenu(null);
  };

  return (
    <section
      className="game-screen"
      aria-label="game screen"
      onClick={() => setContextMenu(null)}
      onContextMenu={(event) => event.preventDefault()}
    >
      <GameHud
        currentLocation={currentLocation}
        onPause={onPause}
        playerCharacter={playerCharacter}
        selectedCharacter={selectedCharacter}
        textMap={textMap}
        world={world}
      />

      <div className="game-layout">
        <main className="map-screen" aria-label="map screen">
          <div className="map-toolbar">
            <div className="area-tabs" aria-label="text map areas">
              {textMap?.areas.map((area) => (
                <button
                  key={area.id}
                  type="button"
                  className={area.id === activeArea?.id ? "active" : undefined}
                  onClick={() => setSelectedAreaId(area.id)}
                  disabled={services.loading}
                >
                  <span>{area.kind === "outing" ? "外" : "内"}</span>
                  {displayText(area.name)}
                </button>
              ))}
            </div>
            <div className="map-tools" aria-label="map tools">
              <button
                type="button"
                className="icon-button"
                onClick={() => setZoom((value) => Math.max(0.86, value - 0.08))}
                aria-label="缩小地图"
              >
                <ZoomOut size={17} aria-hidden="true" />
              </button>
              <button
                type="button"
                className="icon-button"
                onClick={() => setZoom((value) => Math.min(1.24, value + 0.08))}
                aria-label="放大地图"
              >
                <ZoomIn size={17} aria-hidden="true" />
              </button>
              <label className="map-search">
                <Search size={16} aria-hidden="true" />
                <input
                  value={locationSearch}
                  onChange={(event) => setLocationSearch(event.currentTarget.value)}
                  placeholder="搜索地点"
                  aria-label="搜索地点"
                />
              </label>
            </div>
          </div>

          <div className="map-stage">
            <AsciiMapViewport
              area={activeArea}
              currentLocationId={currentLocation?.id}
              hoveredLocationId={hoveredLocationId}
              loading={services.loading}
              onAction={runMapAction}
              onContextMenu={openContextMenu}
              onHoverLocation={setHoveredLocationId}
              onInspectLocation={inspectLocation}
              onMoveLocation={moveTo}
              pinnedLocationId={pinnedLocationId}
              selectedLocationId={inspectedLocation?.id}
              zoom={zoom}
            />

            {hoveredLocation ? (
              <div className="map-tooltip" role="tooltip">
                <strong>{locationName(hoveredLocation)}</strong>
                <span>
                  {areaName(textMap, hoveredLocation.map_area_id)} ·{" "}
                  {terrainName(hoveredLocation.terrain)}
                </span>
                <span>
                  {hoveredLocation.id === currentLocation?.id
                    ? "当前位置"
                    : `移动约 ${hoveredLocation.move_minutes ?? 10} 分钟`}
                </span>
                <span>
                  人物：
                  {charactersAtLocation(world, hoveredLocation.id).length > 0
                    ? charactersAtLocation(world, hoveredLocation.id)
                        .map((character) => characterName(character))
                        .join("、")
                    : "无"}
                </span>
              </div>
            ) : null}
          </div>

          <details className="map-legend" aria-label="location legend">
            <summary>
              <MapPinned size={16} aria-hidden="true" /> 图例 / 地点
            </summary>
            <div className="location-legend-list">
              {filteredLocations.map((location) => {
                const occupants = charactersAtLocation(world, location.id);
                return (
                  <button
                    key={location.id}
                    type="button"
                    className={location.id === currentLocation?.id ? "current" : undefined}
                    onClick={(event) => {
                      event.stopPropagation();
                      inspectLocation(location.id);
                    }}
                    onDoubleClick={() => moveTo(location.id)}
                    onContextMenu={(event) => openContextMenu(event, location.id)}
                    disabled={services.loading}
                    aria-label={`查看 ${locationName(location)}`}
                  >
                    <span className="location-symbol">{locationSymbol(location)}</span>
                    <span>{locationName(location)}</span>
                    {occupants.length > 0 ? (
                      <small>{occupants.map((item) => characterName(item)).join("、")}</small>
                    ) : null}
                  </button>
                );
              })}
            </div>
          </details>
        </main>

        <div className="game-dock">
          <CharacterDock
            currentLocation={currentLocation}
            loading={services.loading}
            onAdjustRelationship={adjustRelationship}
            onRollMood={rollMood}
            onSelectCharacter={setSelectedCharacterId}
            onStartDialogue={startDialogue}
            relationship={selectedCharacterRelationship}
            selectedCharacter={selectedCharacter}
            world={world}
          />
          <LocationDrawer
            currentLocation={currentLocation}
            loading={services.loading}
            location={inspectedLocation}
            onClose={() => setInspectedLocationId(undefined)}
            onMove={moveTo}
            onPin={pinLocation}
            onSwitchArea={switchAreaFromLocation}
            pinnedLocationId={pinnedLocationId}
            textMap={textMap}
            world={world}
          />
        </div>
      </div>

      <ActionBar
        commandPanelOpen={commandPanelOpen}
        currentLocation={currentLocation}
        inspectedLocation={inspectedLocation}
        loading={services.loading}
        onAdjustRelationship={adjustRelationship}
        onMoveInspected={() => inspectedLocation && moveTo(inspectedLocation.id)}
        onRest={() => services.dispatch({ type: "advance_time", minutes: 30 })}
        onRollMood={rollMood}
        onStartDialogue={startDialogue}
        onToggleCommandPanel={() => setCommandPanelOpen((open) => !open)}
      />

      {contextLocation && contextMenu ? (
        <div
          className="context-menu"
          role="menu"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          aria-label={`${locationName(contextLocation)} 操作菜单`}
          onClick={(event) => event.stopPropagation()}
        >
          <button
            type="button"
            role="menuitem"
            onClick={() => moveTo(contextLocation.id)}
            disabled={services.loading || contextLocation.id === currentLocation?.id}
          >
            <Move size={15} aria-hidden="true" /> 移动到这里
          </button>
          <button
            type="button"
            role="menuitem"
            onClick={() => inspectLocation(contextLocation.id)}
          >
            <Search size={15} aria-hidden="true" /> 查看地点
          </button>
          <button
            type="button"
            role="menuitem"
            onClick={() => focusPeopleAtLocation(contextLocation)}
          >
            <MapPinned size={15} aria-hidden="true" /> 查看人物
          </button>
          <button
            type="button"
            role="menuitem"
            onClick={() => switchAreaFromLocation(contextLocation)}
          >
            <LocateFixed size={15} aria-hidden="true" /> 切换区域
          </button>
          <button
            type="button"
            role="menuitem"
            onClick={() => pinLocation(contextLocation.id)}
          >
            <Pin size={15} aria-hidden="true" />{" "}
            {pinnedLocationId === contextLocation.id ? "取消关注" : "设为关注"}
          </button>
        </div>
      ) : null}

      <DialogueLayer
        dispatch={services.dispatch}
        loading={services.loading}
        onClose={() => setDismissedDialogueToken(dialogueToken)}
        open={dialogueOpen}
        world={world}
      />
    </section>
  );
};
