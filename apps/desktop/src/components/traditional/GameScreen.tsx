import { useMemo, useState } from "react";
import type { MouseEvent } from "react";
import {
  Check,
  Clock3,
  Compass,
  Eye,
  HeartHandshake,
  ListChecks,
  Map as MapIcon,
  MapPinned,
  MessageCircle,
  Move,
  PanelRightOpen,
  Pin,
  RefreshCw,
  Search,
  Sparkles,
  UserRound,
  X,
  ZoomIn,
  ZoomOut,
} from "lucide-react";
import { displayText } from "../../engine/displayText";
import type {
  Character,
  Location,
  Relationship,
  TextMap,
  TextMapAction,
  WorldState,
} from "../../types";
import { ActionBar } from "./ActionBar";
import { AsciiMapViewport } from "./AsciiMapViewport";
import { CharacterDock } from "./CharacterDock";
import { DialogueLayer } from "./DialogueLayer";
import { GameHud } from "./GameHud";
import type { ShellServices } from "./shellTypes";
import {
  areaName,
  canRenderImagePath,
  characterName,
  charactersAtLocation,
  findPortrait,
  formatClock,
  locationName,
  locationSymbol,
  seasonLabels,
  terrainName,
  visibleLocationsForTextMap,
  weatherLabels,
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

type MapMode = "inspect" | "move";

interface MainStatusPanelProps {
  commandPanelOpen: boolean;
  currentLocation: Location | undefined;
  currentLocationCharacters: Character[];
  loading: boolean;
  onAdjustRelationship: () => void;
  onOpenCharacters: () => void;
  onOpenMap: () => void;
  onOpenMoveMap: () => void;
  onRest: () => void;
  onRollMood: () => void;
  onStartDialogue: () => void;
  onToggleCommandPanel: () => void;
  relationship: Relationship | undefined;
  selectedCharacter: Character | undefined;
  textMap: TextMap | undefined;
  world: WorldState;
}

const clampPercent = (value: number) => Math.max(0, Math.min(100, value));

const StatusMeter = ({
  label,
  tone,
  value,
}: {
  label: string;
  tone?: "blue" | "green" | "yellow";
  value: number;
}) => (
  <div className="hub-meter">
    <span>{label}</span>
    <strong>{value}</strong>
    <i className={tone ?? "blue"}>
      <b style={{ width: `${clampPercent(value)}%` }} />
    </i>
  </div>
);

const MainStatusPanel = ({
  commandPanelOpen,
  currentLocation,
  currentLocationCharacters,
  loading,
  onAdjustRelationship,
  onOpenCharacters,
  onOpenMap,
  onOpenMoveMap,
  onRest,
  onRollMood,
  onStartDialogue,
  onToggleCommandPanel,
  relationship,
  selectedCharacter,
  textMap,
  world,
}: MainStatusPanelProps) => {
  const portrait = findPortrait(world.resources, selectedCharacter?.id);
  const canRenderPortrait = canRenderImagePath(portrait?.source_path);
  const latestLog = world.event_log.slice(0, 4);
  const energy = selectedCharacter?.state.energy ?? 0;
  const mood = selectedCharacter?.state.mood ?? 0;
  const moodPercent = clampPercent(mood + 50);

  return (
    <main className="status-screen game-hub" aria-label="status screen">
      <section className="hub-scene-panel" aria-label="current scene">
        <div className="hub-panel-heading">
          <span className="panel-kicker">当前场景</span>
          <h1>{locationName(currentLocation)}</h1>
        </div>
        <div className="scene-meta">
          <span>
            <Clock3 size={16} aria-hidden="true" /> {formatClock(world)}
          </span>
          <span>
            <Compass size={16} aria-hidden="true" /> {seasonLabels[world.clock.season]} ·{" "}
            {weatherLabels[world.clock.weather]}
          </span>
          <span>
            <MapPinned size={16} aria-hidden="true" />{" "}
            {areaName(textMap, currentLocation?.map_area_id)}
          </span>
        </div>
        <div className="scene-action-grid" aria-label="scene actions">
          <button type="button" onClick={onStartDialogue} disabled={loading}>
            <MessageCircle size={17} aria-hidden="true" /> 对话
          </button>
          <button type="button" onClick={onAdjustRelationship} disabled={loading}>
            <HeartHandshake size={17} aria-hidden="true" /> 交流
          </button>
          <button type="button" onClick={onRest} disabled={loading}>
            <Clock3 size={17} aria-hidden="true" /> 休息
          </button>
          <button type="button" onClick={onOpenMoveMap} disabled={loading}>
            <Move size={17} aria-hidden="true" /> 移动
          </button>
          <button type="button" onClick={onOpenMap} disabled={loading}>
            <MapIcon size={17} aria-hidden="true" /> 地图
          </button>
          <button type="button" onClick={onToggleCommandPanel} aria-expanded={commandPanelOpen}>
            <ListChecks size={17} aria-hidden="true" /> 命令
          </button>
        </div>
      </section>

      <section className="hub-character-panel" aria-label="focused character">
        <div className="status-portrait" aria-label="character portrait">
          {canRenderPortrait && portrait ? (
            <img src={portrait.source_path} alt={`${characterName(selectedCharacter)} 立绘`} />
          ) : (
            <div className="portrait-fallback">
              <span>{Array.from(characterName(selectedCharacter))[0] ?? "人"}</span>
              <small>{portrait ? "立绘未加载" : "未绑定立绘"}</small>
            </div>
          )}
        </div>
        <div className="character-readout">
          <span className="panel-kicker">当前人物</span>
          <h2>{characterName(selectedCharacter)}</h2>
          <p>
            好感度:S {relationship?.affinity ?? "?"} / 信赖度:A{" "}
            {relationship?.trust ?? "?"}
          </p>
          <div className="hub-meter-stack">
            <StatusMeter label="体力" tone="green" value={energy} />
            <StatusMeter label="心情" tone="yellow" value={moodPercent} />
          </div>
          <div className="hub-inline-actions">
            <button type="button" onClick={onOpenCharacters}>
              <UserRound size={16} aria-hidden="true" /> 人物
            </button>
            <button type="button" onClick={onRollMood} disabled={loading}>
              <RefreshCw size={16} aria-hidden="true" /> 状态
            </button>
          </div>
        </div>
      </section>

      <section className="hub-occupants-panel" aria-label="location occupants">
        <div className="hub-panel-heading compact">
          <span className="panel-kicker">当前位置人物</span>
          <strong>{currentLocationCharacters.length}</strong>
        </div>
        <div className="occupant-list">
          {currentLocationCharacters.length > 0 ? (
            currentLocationCharacters.map((character) => (
              <button key={character.id} type="button" onClick={onOpenCharacters}>
                <UserRound size={16} aria-hidden="true" />
                <span>{characterName(character)}</span>
                <small>
                  体力 {character.state.energy} / 心情 {character.state.mood}
                </small>
              </button>
            ))
          ) : (
            <span className="empty-text">无人</span>
          )}
        </div>
      </section>

      <section className="hub-log-panel" aria-label="event log">
        <div className="hub-panel-heading compact">
          <span className="panel-kicker">事件</span>
          <small>
            命令 {world.command_log.length} / 计划 {world.scheduled_events.length}
          </small>
        </div>
        <ol>
          {latestLog.map((entry, index) => (
            <li key={`${entry}:${index}`}>{displayText(entry)}</li>
          ))}
        </ol>
        {commandPanelOpen ? (
          <div className="hub-command-panel" aria-label="command panel">
            <button type="button" onClick={onRollMood} disabled={loading}>
              <Sparkles size={16} aria-hidden="true" /> 随机心情
            </button>
            <button type="button" onClick={onOpenMap} disabled={loading}>
              <Eye size={16} aria-hidden="true" /> 查看地图
            </button>
            <button type="button" onClick={onOpenMoveMap} disabled={loading}>
              <Move size={16} aria-hidden="true" /> 选择目的地
            </button>
          </div>
        ) : null}
      </section>
    </main>
  );
};

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
  const [mapMode, setMapMode] = useState<MapMode | null>(null);
  const [selectedAreaId, setSelectedAreaId] = useState<string | undefined>();
  const [inspectedLocationId, setInspectedLocationId] = useState<string | undefined>();
  const [hoveredLocationId, setHoveredLocationId] = useState<string | undefined>();
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const [pinnedLocationId, setPinnedLocationId] = useState<string | undefined>();
  const [selectedCharacterId, setSelectedCharacterId] = useState<string | undefined>();
  const [characterPanelOpen, setCharacterPanelOpen] = useState(false);
  const [dismissedDialogueToken, setDismissedDialogueToken] = useState<string | null>(
    null,
  );
  const [commandPanelOpen, setCommandPanelOpen] = useState(false);
  const [locationSearch, setLocationSearch] = useState("");
  const [zoom, setZoom] = useState(1);

  const mapOpen = mapMode !== null;
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
  const selectedDestination = inspectedLocation ?? currentLocation;
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
  const areaLegendGroups = useMemo(() => {
    const groupedLocationIds = new Set<string>();
    const groups =
      textMap?.areas
        .map((area) => {
          const locations = filteredLocations.filter(
            (location) => location.map_area_id === area.id,
          );
          locations.forEach((location) => groupedLocationIds.add(location.id));
          return {
            id: area.id,
            title: displayText(area.name),
            locations,
          };
        })
        .filter((group) => group.locations.length > 0) ?? [];
    const otherLocations = filteredLocations.filter(
      (location) => !groupedLocationIds.has(location.id),
    );
    return otherLocations.length > 0
      ? [...groups, { id: "other", title: "其他", locations: otherLocations }]
      : groups;
  }, [filteredLocations, textMap]);
  const dialogueToken =
    world.active_dialogue.length > 0
      ? `${world.active_dialogue_scene_id ?? "dialogue"}:${
          world.active_dialogue[0]?.id ?? "entry"
        }:${world.active_dialogue.length}`
      : null;
  const dialogueOpen =
    dialogueToken !== null && dismissedDialogueToken !== dialogueToken;

  const selectLocation = (locationId: string) => {
    const location = world.locations.find((item) => item.id === locationId);
    setInspectedLocationId(locationId);
    setContextMenu(null);
    if (location?.map_area_id) {
      setSelectedAreaId(location.map_area_id);
    }
  };

  const openMap = (mode: MapMode = "inspect") => {
    setMapMode(mode);
    setInspectedLocationId(currentLocation?.id);
    setSelectedAreaId(currentLocation?.map_area_id ?? textMap?.default_area_id);
    setContextMenu(null);
  };

  const closeMap = () => {
    setMapMode(null);
    setContextMenu(null);
    setHoveredLocationId(undefined);
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
      closeMap();
      return;
    }
    closeMap();
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

  const focusPeopleAtLocation = (location: Location | undefined) => {
    const firstOccupant = charactersAtLocation(world, location?.id)[0];
    if (firstOccupant) {
      setSelectedCharacterId(firstOccupant.id);
      setCharacterPanelOpen(true);
    }
    setContextMenu(null);
  };

  const renderLocationButton = (location: Location) => {
    const occupants = charactersAtLocation(world, location.id);
    return (
      <button
        key={location.id}
        type="button"
        className={[
          location.id === currentLocation?.id ? "current" : "",
          location.id === selectedDestination?.id ? "selected" : "",
        ]
          .filter(Boolean)
          .join(" ")}
        onClick={(event) => {
          event.stopPropagation();
          selectLocation(location.id);
        }}
        onDoubleClick={() => moveTo(location.id)}
        onContextMenu={(event) => openContextMenu(event, location.id)}
        disabled={services.loading}
        aria-label={`选择 ${locationName(location)}`}
      >
        <span className="location-symbol">{locationSymbol(location)}</span>
        <span>{locationName(location)}</span>
        {occupants.length > 0 ? (
          <small>{occupants.map((item) => characterName(item)).join("、")}</small>
        ) : null}
      </button>
    );
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
        onOpenCharacters={() => setCharacterPanelOpen(true)}
        onPause={onPause}
        playerCharacter={playerCharacter}
        selectedCharacter={selectedCharacter}
        textMap={textMap}
        world={world}
      />

      <div className="game-layout">
        <MainStatusPanel
          commandPanelOpen={commandPanelOpen}
          currentLocation={currentLocation}
          currentLocationCharacters={currentLocationCharacters}
          loading={services.loading}
          onAdjustRelationship={adjustRelationship}
          onOpenCharacters={() => setCharacterPanelOpen(true)}
          onOpenMap={() => openMap("inspect")}
          onOpenMoveMap={() => openMap("move")}
          onRest={() => services.dispatch({ type: "advance_time", minutes: 30 })}
          onRollMood={rollMood}
          onStartDialogue={startDialogue}
          onToggleCommandPanel={() => setCommandPanelOpen((open) => !open)}
          relationship={selectedCharacterRelationship}
          selectedCharacter={selectedCharacter}
          textMap={textMap}
          world={world}
        />

        <div className="floating-panel-stack">
          {characterPanelOpen ? (
            <CharacterDock
              currentLocation={currentLocation}
              loading={services.loading}
              onAdjustRelationship={adjustRelationship}
              onClose={() => setCharacterPanelOpen(false)}
              onRollMood={rollMood}
              onSelectCharacter={setSelectedCharacterId}
              onStartDialogue={startDialogue}
              relationship={selectedCharacterRelationship}
              selectedCharacter={selectedCharacter}
              world={world}
            />
          ) : null}
        </div>
      </div>

      <ActionBar
        commandPanelOpen={commandPanelOpen}
        loading={services.loading}
        onAdjustRelationship={adjustRelationship}
        onOpenMap={() => openMap("inspect")}
        onOpenMoveMap={() => openMap("move")}
        onRest={() => services.dispatch({ type: "advance_time", minutes: 30 })}
        onStartDialogue={startDialogue}
        onToggleCommandPanel={() => setCommandPanelOpen((open) => !open)}
      />

      {mapOpen ? (
        <section
          className={`move-map-overlay ${mapMode === "move" ? "move-mode" : "inspect-mode"}`}
          aria-label={mapMode === "move" ? "movement map" : "world map"}
        >
          <header className="move-map-header">
            <div>
              <span>{mapMode === "move" ? "移动" : "地图"}</span>
              <h2>{locationName(selectedDestination)}</h2>
            </div>
            <div className="map-mode-switch" aria-label="map mode">
              <button
                type="button"
                className={mapMode === "inspect" ? "active" : undefined}
                onClick={() => setMapMode("inspect")}
              >
                <Eye size={16} aria-hidden="true" /> 浏览
              </button>
              <button
                type="button"
                className={mapMode === "move" ? "active" : undefined}
                onClick={() => setMapMode("move")}
              >
                <Move size={16} aria-hidden="true" /> 移动
              </button>
            </div>
            <div className="area-tabs" aria-label="text map areas">
              {textMap?.areas.map((area) => (
                <button
                  key={area.id}
                  type="button"
                  className={area.id === activeArea?.id ? "active" : undefined}
                  onClick={() => setSelectedAreaId(area.id)}
                  disabled={services.loading}
                >
                  {displayText(area.name)}
                </button>
              ))}
            </div>
            <div className="map-tools" aria-label="map tools">
              <button
                type="button"
                className="icon-button"
                onClick={() => setZoom((value) => Math.max(0.72, value - 0.08))}
                aria-label="缩小地图"
              >
                <ZoomOut size={17} aria-hidden="true" />
              </button>
              <button
                type="button"
                className="icon-button"
                onClick={() => setZoom((value) => Math.min(1.45, value + 0.08))}
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
              <button
                type="button"
                className="icon-button"
                onClick={closeMap}
                aria-label="关闭地图"
              >
                <X size={18} aria-hidden="true" />
              </button>
            </div>
          </header>

          <div className="move-map-layout">
            <main className="map-screen" aria-label="map screen">
              <div className="map-stage">
                <AsciiMapViewport
                  area={activeArea}
                  currentLocationId={currentLocation?.id}
                  hoveredLocationId={hoveredLocationId}
                  loading={services.loading}
                  onAction={runMapAction}
                  onContextMenu={openContextMenu}
                  onHoverLocation={setHoveredLocationId}
                  onInspectLocation={selectLocation}
                  onMoveLocation={moveTo}
                  onZoomChange={setZoom}
                  pinnedLocationId={pinnedLocationId}
                  selectedLocationId={selectedDestination?.id}
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
            </main>

            <aside className="move-map-sidebar">
              <section className="move-destination-panel" aria-label="selected destination">
                <span className="panel-kicker">
                  {mapMode === "move" ? "目的地" : "选中地点"}
                </span>
                <h3>{locationName(selectedDestination)}</h3>
                <dl className="compact-dl">
                  <div>
                    <dt>区域</dt>
                    <dd>{areaName(textMap, selectedDestination?.map_area_id)}</dd>
                  </div>
                  <div>
                    <dt>地形</dt>
                    <dd>{terrainName(selectedDestination?.terrain)}</dd>
                  </div>
                  <div>
                    <dt>移动</dt>
                    <dd>
                      {selectedDestination?.id === currentLocation?.id
                        ? "当前位置"
                        : `${selectedDestination?.move_minutes ?? 10} 分钟`}
                    </dd>
                  </div>
                  <div>
                    <dt>人物</dt>
                    <dd>
                      {charactersAtLocation(world, selectedDestination?.id).length > 0
                        ? charactersAtLocation(world, selectedDestination?.id)
                            .map((character) => characterName(character))
                            .join("、")
                        : "无"}
                    </dd>
                  </div>
                </dl>
                <div className="drawer-actions">
                  <button
                    type="button"
                    onClick={() => selectedDestination && moveTo(selectedDestination.id)}
                    disabled={
                      services.loading ||
                      !selectedDestination ||
                      selectedDestination.id === currentLocation?.id
                    }
                  >
                    <Check size={16} aria-hidden="true" />{" "}
                    {mapMode === "move" ? "确定移动" : "移动到这里"}
                  </button>
                  <button
                    type="button"
                    onClick={() => focusPeopleAtLocation(selectedDestination)}
                  >
                    <UserRound size={16} aria-hidden="true" /> 人物
                  </button>
                  <button
                    type="button"
                    onClick={() => selectedDestination && pinLocation(selectedDestination.id)}
                    disabled={!selectedDestination}
                  >
                    <Pin size={16} aria-hidden="true" />{" "}
                    {selectedDestination?.id === pinnedLocationId ? "取消关注" : "关注"}
                  </button>
                </div>
              </section>

              <section className="move-map-legend" aria-label="location legend">
                <h3>
                  <PanelRightOpen size={16} aria-hidden="true" /> 图例 / 地点
                </h3>
                {areaLegendGroups.map((group, index) => (
                  <details
                    className="move-map-legend-group"
                    key={group.id}
                    open={group.id === activeArea?.id || index === 0}
                  >
                    <summary>
                      <span>{group.title}</span>
                      <small>{group.locations.length}</small>
                    </summary>
                    <div className="location-legend-list">
                      {group.locations.map((location) => renderLocationButton(location))}
                    </div>
                  </details>
                ))}
                {areaLegendGroups.length === 0 ? (
                  <p className="empty-legend-text">没有匹配地点</p>
                ) : null}
              </section>
            </aside>
          </div>
        </section>
      ) : null}

      {mapOpen && contextLocation && contextMenu ? (
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
            onClick={() => selectLocation(contextLocation.id)}
          >
            <Search size={15} aria-hidden="true" /> 设为目的地
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
