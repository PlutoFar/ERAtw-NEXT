import { useMemo, useState } from "react";
import type { MouseEvent, ReactNode } from "react";
import {
  Check,
  MapPinned,
  Move,
  Pin,
  Search,
  X,
  ZoomIn,
  ZoomOut,
} from "lucide-react";
import { displayText } from "../../engine/displayText";
import type { Character, Location, TextMap, TextMapAction, WorldState } from "../../types";
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
  groupLocationLegendLocations,
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

interface MainStatusPanelProps {
  currentLocation: Location | undefined;
  relationshipAffinity: number | undefined;
  relationshipTrust: number | undefined;
  selectedCharacter: Character | undefined;
  textMap: TextMap | undefined;
  world: WorldState;
}

const Meter = ({
  label,
  tone,
  value,
}: {
  label: string;
  tone?: "blue" | "green" | "red";
  value: number;
}) => (
  <span className="status-meter">
    <span>{label}</span>
    <span className={`status-meter-bar ${tone ?? "blue"}`}>
      <span style={{ width: `${Math.max(4, Math.min(100, value))}%` }} />
    </span>
    <strong>{value}</strong>
  </span>
);

const TerminalRow = ({ children }: { children: ReactNode }) => (
  <div className="terminal-row">{children}</div>
);

const MainStatusPanel = ({
  currentLocation,
  relationshipAffinity,
  relationshipTrust,
  selectedCharacter,
  textMap,
  world,
}: MainStatusPanelProps) => {
  const portrait = findPortrait(world.resources, selectedCharacter?.id);
  const canRenderPortrait = canRenderImagePath(portrait?.source_path);
  const energy = selectedCharacter?.state.energy ?? 0;
  const mood = selectedCharacter?.state.mood ?? 0;

  return (
    <main className="status-screen" aria-label="status screen">
      <div className="terminal-frame">
        <TerminalRow>
          <span>******************************************************</span>
        </TerminalRow>
        <TerminalRow>
          <strong>{formatClock(world)}</strong>
          <span>春・晴</span>
          <span>気温 12.5°C</span>
          <span>&lt;可吃饭&gt;</span>
        </TerminalRow>
        <TerminalRow>
          <span>TSP</span>
          <span className="tsp-bar" />
          <span>(6680/6680)[时间停止可]</span>
        </TerminalRow>
        <TerminalRow>
          <span>{locationName(currentLocation)}</span>
          <span>清洁度:最高</span>
          <span>{areaName(textMap, currentLocation?.map_area_id)}</span>
        </TerminalRow>
        <TerminalRow>
          <strong>[{characterName(selectedCharacter)}]</strong>
        </TerminalRow>
        <TerminalRow>
          <span>
            {characterName(selectedCharacter)}
            (好感度:S {relationshipAffinity ?? "?"}, 信赖度:A{" "}
            {relationshipTrust ?? "?"}, 欲求不满度:1%)
          </span>
          <span>子供の数:0</span>
          <span>愤怒:</span>
        </TerminalRow>
        <TerminalRow>
          <span>工作情报: 寺子屋的医护</span>
          <span>平日 8時～14時</span>
          <span>职场: {locationName(currentLocation)}</span>
          <span>战斗能力:B3</span>
        </TerminalRow>

        <section className="terminal-section" aria-label="status values">
          <h2>▼[-][Status]--------[能力表示]</h2>
          <div className="status-meter-grid">
            <Meter label="天明 体" tone="green" value={Math.min(100, energy)} />
            <Meter label="気" value={80} />
            <Meter label="酒" tone="red" value={0} />
            <Meter label="精" tone="green" value={70} />
            <Meter label={`${characterName(selectedCharacter)} 体`} tone="green" value={energy} />
            <Meter label="気" value={Math.max(4, mood * 8)} />
            <Meter label="酒" tone="red" value={0} />
          </div>
          <TerminalRow>
            <span>情绪:</span>
            <span>理性:★★★★★</span>
          </TerminalRow>
        </section>

        <section className="terminal-section" aria-label="palam values">
          <h2>▼[-][Palam]--------</h2>
          <div className="palam-grid">
            {[
              "快C",
              "快V",
              "快A",
              "快B",
              "快M",
              "润滑",
              "恭顺",
              "欲情",
              "屈服",
              "习得",
              "耻情",
              "苦痛",
              "恐怖",
              "好意",
              "优越",
              "反感",
              "不快",
              "抑郁",
              "眠气",
            ].map((item) => (
              <span key={item}>
                {item}Lv 0 <span className="palam-bar" /> 0
              </span>
            ))}
          </div>
        </section>

        <section className="terminal-section look-section" aria-label="look values">
          <div>
            <h2>▼[-][Look]--------[画像表示][表示設定]</h2>
            <TerminalRow>【上半身】 女式衬衫</TerminalRow>
            <TerminalRow>【下半身】 裙子</TerminalRow>
            <TerminalRow>【★857】 美丽な符卡</TerminalRow>
          </div>
          <div className="status-portrait" aria-label="character portrait">
            {canRenderPortrait && portrait ? (
              <img src={portrait.source_path} alt={`${characterName(selectedCharacter)} 立绘`} />
            ) : (
              <div className="portrait-fallback">
                <span>{Array.from(characterName(selectedCharacter))[0] ?? "人"}</span>
                <small>{portrait ? portrait.resource_id : "未绑定立绘"}</small>
              </div>
            )}
          </div>
        </section>

        <section className="terminal-command-lines" aria-label="terminal commands">
          <TerminalRow>
            ====== Act_COM == [★] [V]=[A]=[S M]=[奉仕]=[性骚扰]=[家务]=[口上]=[自动避孕套装备]=[自动喘息: 开启] ======
          </TerminalRow>
          <TerminalRow>会话[300] 泡茶[301]身体接触[302] 劝酒[332] 膝枕[336]</TerminalRow>
          <TerminalRow>
            <strong>带出去[351]</strong> 休憩[403]日记本[406] 学习[412]制作料理[413] 演奏[416]
          </TerminalRow>
          <TerminalRow>
            ▼[-][Ex_COM] ================================================================
          </TerminalRow>
          <TerminalRow>
            <strong>移动[400]</strong> 外出[405] 污渍显示[801]能力表示[803]道具确认[805]居场所察知[811]
          </TerminalRow>
          <TerminalRow>&lt;上回指令:&gt;</TerminalRow>
        </section>
      </div>
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
  const [mapOpen, setMapOpen] = useState(false);
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
  const legendGroups = useMemo(
    () => groupLocationLegendLocations(filteredLocations),
    [filteredLocations],
  );
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

  const openMoveMap = () => {
    setMapOpen(true);
    setInspectedLocationId(currentLocation?.id);
    setSelectedAreaId(currentLocation?.map_area_id ?? textMap?.default_area_id);
  };

  const closeMoveMap = () => {
    setMapOpen(false);
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
      closeMoveMap();
      return;
    }
    closeMoveMap();
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
          currentLocation={currentLocation}
          relationshipAffinity={selectedCharacterRelationship?.affinity}
          relationshipTrust={selectedCharacterRelationship?.trust}
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
        inspectedLocation={selectedDestination}
        loading={services.loading}
        onAdjustRelationship={adjustRelationship}
        onOpenMoveMap={openMoveMap}
        onRest={() => services.dispatch({ type: "advance_time", minutes: 30 })}
        onRollMood={rollMood}
        onStartDialogue={startDialogue}
        onToggleCommandPanel={() => setCommandPanelOpen((open) => !open)}
      />

      {mapOpen ? (
        <section className="move-map-overlay" aria-label="movement map">
          <header className="move-map-header">
            <div>
              <span>移动</span>
              <h2>{locationName(selectedDestination)}</h2>
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
                  <span>{area.kind === "outing" ? "外" : "内"}</span>
                  {displayText(area.name)}
                </button>
              ))}
            </div>
            <div className="map-tools" aria-label="map tools">
              <button
                type="button"
                className="icon-button"
                onClick={() => setZoom((value) => Math.max(0.78, value - 0.08))}
                aria-label="缩小地图"
              >
                <ZoomOut size={17} aria-hidden="true" />
              </button>
              <button
                type="button"
                className="icon-button"
                onClick={() => setZoom((value) => Math.min(1.18, value + 0.08))}
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
                onClick={closeMoveMap}
                aria-label="关闭移动地图"
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
                <span className="panel-kicker">目的地</span>
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
                    <Check size={16} aria-hidden="true" /> 确定移动
                  </button>
                  <button type="button" onClick={closeMoveMap}>
                    <X size={16} aria-hidden="true" /> 取消
                  </button>
                </div>
              </section>

              <section className="move-map-legend" aria-label="location legend">
                <h3>
                  <MapPinned size={16} aria-hidden="true" /> 图例 / 地点
                </h3>
                {legendGroups.map((group, index) => (
                  <details
                    className="move-map-legend-group"
                    key={group.id}
                    open={index < 2}
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
                {legendGroups.length === 0 ? (
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
