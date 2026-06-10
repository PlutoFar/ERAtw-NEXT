import { useMemo, useState } from "react";
import type { MouseEvent } from "react";
import {
  Clock3,
  LocateFixed,
  MapPinned,
  MessageCircle,
  Move,
  Pin,
  RefreshCw,
  Search,
  Sparkles,
  UserRound,
} from "lucide-react";
import { displayText } from "../engine/displayText";
import type {
  Character,
  EngineCommand,
  Location,
  ResourceAsset,
  TextMap,
  TextMapAction,
  WorldState,
} from "../types";

interface TraditionalViewProps {
  world: WorldState;
  dispatch: (command: EngineCommand) => void | Promise<void>;
  loading?: boolean;
}

interface ContextMenuState {
  locationId: string;
  x: number;
  y: number;
}

const formatClock = (world: WorldState) =>
  `第${world.clock.day}日 ${String(world.clock.hour).padStart(2, "0")}:${String(
    world.clock.minute,
  ).padStart(2, "0")}`;

const weatherLabels = {
  clear: "晴",
  cloudy: "阴",
  rain: "雨",
  snow: "雪",
};

const seasonLabels = {
  spring: "春",
  summer: "夏",
  autumn: "秋",
  winter: "冬",
};

const terrainLabels: Record<string, string> = {
  street: "街道",
  interior: "室内",
  grass: "户外",
};

const locationValue = (legacyPlaceId: number | null | undefined) =>
  legacyPlaceId === null || legacyPlaceId === undefined
    ? "??"
    : String(legacyPlaceId).slice(-2).padStart(2, "0");

const mapLocationIds = (textMap: TextMap | undefined) =>
  new Set(textMap?.locations.map((location) => location.location_id) ?? []);

const charactersAtLocation = (world: WorldState, locationId: string | undefined) =>
  locationId
    ? world.characters.filter((character) => character.location_id === locationId)
    : [];

const areaName = (textMap: TextMap | undefined, areaId: string | null | undefined) =>
  displayText(textMap?.areas.find((area) => area.id === areaId)?.name, "未知区域");

const locationName = (location: Location | undefined, fallback = "未知地点") =>
  displayText(location?.name, fallback);

const characterName = (character: Character | undefined, fallback = "未知人物") =>
  displayText(character?.display_name, fallback);

const terrainName = (terrain: string | undefined) =>
  terrain ? terrainLabels[terrain] ?? displayText(terrain) : "未知地形";

const findPortrait = (
  resources: ResourceAsset[],
  characterId: string | undefined,
): ResourceAsset | undefined =>
  characterId
    ? resources.find(
        (resource) =>
          resource.media_type === "image" &&
          resource.usage.includes("portrait") &&
          resource.character_bindings.includes(characterId),
      )
    : undefined;

const canRenderImagePath = (sourcePath: string | undefined) =>
  !!sourcePath &&
  (sourcePath.startsWith("/") ||
    sourcePath.startsWith("http://") ||
    sourcePath.startsWith("https://") ||
    sourcePath.startsWith("data:image/"));

export const TraditionalView = ({
  world,
  dispatch,
  loading = false,
}: TraditionalViewProps) => {
  const playerCharacter = world.characters[0];
  const currentLocation = world.locations.find(
    (location) => location.id === playerCharacter?.location_id,
  );
  const textMap = world.text_maps[0];
  const locationIds = useMemo(() => mapLocationIds(textMap), [textMap]);
  const [selectedAreaId, setSelectedAreaId] = useState<string | undefined>();
  const [inspectedLocationId, setInspectedLocationId] = useState<string | undefined>();
  const [hoveredLocationId, setHoveredLocationId] = useState<string | undefined>();
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const [pinnedLocationId, setPinnedLocationId] = useState<string | undefined>();
  const [selectedCharacterId, setSelectedCharacterId] = useState<string | undefined>();

  const activeAreaId =
    selectedAreaId ??
    currentLocation?.map_area_id ??
    textMap?.default_area_id ??
    textMap?.areas[0]?.id;
  const activeArea =
    textMap?.areas.find((area) => area.id === activeAreaId) ?? textMap?.areas[0];
  const visibleLocations = world.locations.filter((location) =>
    locationIds.has(location.id),
  );
  const inspectedLocation =
    world.locations.find((location) => location.id === inspectedLocationId) ??
    currentLocation ??
    visibleLocations[0];
  const hoveredLocation = world.locations.find(
    (location) => location.id === hoveredLocationId,
  );
  const contextLocation = world.locations.find(
    (location) => location.id === contextMenu?.locationId,
  );
  const currentLocationCharacters = charactersAtLocation(world, currentLocation?.id);
  const inspectedLocationCharacters = charactersAtLocation(world, inspectedLocation?.id);
  const selectedCharacter =
    currentLocationCharacters.find((character) => character.id === selectedCharacterId) ??
    currentLocationCharacters[0] ??
    playerCharacter;
  const selectedCharacterRelationship = world.relationships.find(
    (item) =>
      item.source_character_id === "player" &&
      item.target_character_id === selectedCharacter?.id,
  );
  const selectedPortrait = findPortrait(world.resources, selectedCharacter?.id);
  const canRenderPortrait = canRenderImagePath(selectedPortrait?.source_path);

  const moveTo = (locationId: string) => {
    if (!playerCharacter || loading) {
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
    void dispatch({
      type: "move_character",
      character_id: playerCharacter.id,
      location_id: locationId,
    });
  };

  const inspectLocation = (locationId: string) => {
    const location = world.locations.find((item) => item.id === locationId);
    setInspectedLocationId(locationId);
    setContextMenu(null);
    if (location?.map_area_id) {
      setSelectedAreaId(location.map_area_id);
    }
  };

  const runAction = (action: TextMapAction) => {
    if (action.type === "move_to_location") {
      inspectLocation(action.location_id);
    } else if (action.type === "switch_area") {
      setSelectedAreaId(action.area_id);
      setContextMenu(null);
    } else {
      setSelectedAreaId(currentLocation?.map_area_id ?? textMap?.default_area_id);
      setContextMenu(null);
    }
  };

  const startDialogue = () => {
    void dispatch({ type: "start_dialogue", scene_id: "demo_morning" });
  };

  const adjustRelationship = () => {
    if (!selectedCharacter) {
      return;
    }
    void dispatch({
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
    void dispatch({
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
    setInspectedLocationId(locationId);
    setContextMenu({ locationId, x: event.clientX, y: event.clientY });
  };

  const pinLocation = (locationId: string) => {
    setPinnedLocationId((current) => (current === locationId ? undefined : locationId));
    setContextMenu(null);
  };

  const setCurrentAreaFromLocation = (location: Location | undefined) => {
    if (location?.map_area_id) {
      setSelectedAreaId(location.map_area_id);
    }
    setContextMenu(null);
  };

  const renderLocationSummary = (location: Location) => {
    const occupants = charactersAtLocation(world, location.id);
    const isCurrent = location.id === currentLocation?.id;
    return (
      <>
        <strong>{locationName(location)}</strong>
        <span>
          {areaName(textMap, location.map_area_id)} · {terrainName(location.terrain)}
        </span>
        <span>{isCurrent ? "当前位置" : `移动约 ${location.move_minutes ?? 10} 分钟`}</span>
        <span>
          人物：
          {occupants.length > 0
            ? occupants.map((character) => characterName(character)).join("、")
            : "无"}
        </span>
      </>
    );
  };

  return (
    <div
      className="traditional-view"
      onClick={() => setContextMenu(null)}
      onContextMenu={(event) => event.preventDefault()}
    >
      <section className="terminal-screen" aria-label="era traditional ui">
        <div className="terminal-line terminal-title">eraTheWorld TW NEXT</div>
        <div className="terminal-line">
          {formatClock(world)} / {seasonLabels[world.clock.season]} /{" "}
          {weatherLabels[world.clock.weather]} / {displayText(textMap?.name, "地图未载入")}
        </div>
        <div className="terminal-rule" />
        <div className="terminal-line">
          MASTER 位置：
          <span className="terminal-current">
            {currentLocation ? locationName(currentLocation) : playerCharacter?.location_id ?? "未知"}
          </span>
        </div>
        <div className="terminal-line">
          体力：{playerCharacter?.state.energy ?? "--"} 心情：
          {playerCharacter?.state.mood ?? "--"} 好感：
          {selectedCharacterRelationship?.affinity ?? "--"} 信赖：
          {selectedCharacterRelationship?.trust ?? "--"}
        </div>

        <div className="terminal-area-tabs" aria-label="text map areas">
          {textMap?.areas.map((area) => (
            <button
              key={area.id}
              type="button"
              className={area.id === activeArea?.id ? "active" : undefined}
              onClick={() => setSelectedAreaId(area.id)}
              disabled={loading}
            >
              <span>{area.kind === "outing" ? "外" : "内"}</span>
              {displayText(area.name)}
            </button>
          ))}
        </div>

        <div className="terminal-map-wrap">
          <div className="terminal-map" aria-label="era text map">
            {activeArea ? (
              activeArea.rows.map((row, rowIndex) => (
                <div className="terminal-map-row" key={`${activeArea.id}:${rowIndex}`}>
                  {row.runs.map((run, runIndex) => {
                    const action = run.action;
                    const isCurrent =
                      action?.type === "move_to_location" &&
                      action.location_id === currentLocation?.id;
                    const actionLocation =
                      action?.type === "move_to_location"
                        ? world.locations.find(
                            (location) => location.id === action.location_id,
                          )
                        : undefined;
                    if (action) {
                      const label =
                        action.type === "move_to_location"
                          ? locationName(actionLocation, action.title ?? action.label)
                          : displayText(action.title ?? action.label);
                      return (
                        <button
                          key={`${rowIndex}:${runIndex}`}
                          type="button"
                          className={
                            isCurrent
                              ? "terminal-map-button current"
                              : "terminal-map-button"
                          }
                          style={{ color: isCurrent ? undefined : run.color ?? undefined }}
                          onClick={(event) => {
                            event.stopPropagation();
                            runAction(action);
                          }}
                          onDoubleClick={() => {
                            if (action.type === "move_to_location") {
                              moveTo(action.location_id);
                            }
                          }}
                          onContextMenu={(event) => {
                            if (action.type === "move_to_location") {
                              openContextMenu(event, action.location_id);
                            }
                          }}
                          onFocus={() => {
                            if (action.type === "move_to_location") {
                              setHoveredLocationId(action.location_id);
                            }
                          }}
                          onBlur={() => setHoveredLocationId(undefined)}
                          onMouseEnter={() => {
                            if (action.type === "move_to_location") {
                              setHoveredLocationId(action.location_id);
                            }
                          }}
                          onMouseLeave={() => setHoveredLocationId(undefined)}
                          disabled={loading}
                          aria-label={label}
                        >
                          {displayText(run.text)}
                        </button>
                      );
                    }
                    return (
                      <span
                        key={`${rowIndex}:${runIndex}`}
                        style={{ color: run.color ?? undefined }}
                      >
                        {displayText(run.text)}
                      </span>
                    );
                  })}
                </div>
              ))
            ) : (
              <div className="terminal-map-row">NO TEXT MAP DATA</div>
            )}
          </div>
          {hoveredLocation ? (
            <div className="terminal-hover-card" role="tooltip">
              {renderLocationSummary(hoveredLocation)}
            </div>
          ) : null}
          {contextLocation && contextMenu ? (
            <div
              className="terminal-context-menu"
              role="menu"
              style={{ left: contextMenu.x, top: contextMenu.y }}
              aria-label={`${locationName(contextLocation)} 操作菜单`}
              onClick={(event) => event.stopPropagation()}
            >
              <button
                type="button"
                role="menuitem"
                onClick={() => moveTo(contextLocation.id)}
                disabled={loading || contextLocation.id === currentLocation?.id}
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
                onClick={() => setCurrentAreaFromLocation(contextLocation)}
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
        </div>

        <div className="terminal-rule" />
        <div className="terminal-command-grid" aria-label="era commands">
          <button
            type="button"
            onClick={() => dispatch({ type: "advance_time", minutes: 30 })}
            disabled={loading}
          >
            <Clock3 size={16} aria-hidden="true" /> 休息
          </button>
          <button type="button" onClick={startDialogue} disabled={loading}>
            <MessageCircle size={16} aria-hidden="true" /> 对话
          </button>
          <button type="button" onClick={rollMood} disabled={loading}>
            <RefreshCw size={16} aria-hidden="true" /> 随机心情
          </button>
          <button type="button" onClick={adjustRelationship} disabled={loading}>
            <Sparkles size={16} aria-hidden="true" /> 交流
          </button>
          <button
            type="button"
            onClick={() => currentLocation && inspectLocation(currentLocation.id)}
            disabled={loading || !currentLocation}
          >
            <MapPinned size={16} aria-hidden="true" /> 当前位置
          </button>
        </div>

        {inspectedLocation ? (
          <section className="terminal-location-panel" aria-label="location details">
            <div className="terminal-panel-heading">
              <h2>
                <MapPinned size={17} aria-hidden="true" /> {locationName(inspectedLocation)}
              </h2>
              <div className="terminal-badges">
                {inspectedLocation.id === currentLocation?.id ? <span>当前</span> : null}
                {pinnedLocationId === inspectedLocation.id ? <span>关注</span> : null}
              </div>
            </div>
            <dl>
              <div>
                <dt>区域</dt>
                <dd>{areaName(textMap, inspectedLocation.map_area_id)}</dd>
              </div>
              <div>
                <dt>地形</dt>
                <dd>{terrainName(inspectedLocation.terrain)}</dd>
              </div>
              <div>
                <dt>移动</dt>
                <dd>
                  {inspectedLocation.id === currentLocation?.id
                    ? "已在此处"
                    : `${inspectedLocation.move_minutes ?? 10} 分钟`}
                </dd>
              </div>
              <div>
                <dt>人物</dt>
                <dd>
                  {inspectedLocationCharacters.length > 0
                    ? inspectedLocationCharacters
                        .map((character) => characterName(character))
                        .join("、")
                    : "无"}
                </dd>
              </div>
            </dl>
            <div className="terminal-action-row">
              <button
                type="button"
                onClick={() => moveTo(inspectedLocation.id)}
                disabled={loading || inspectedLocation.id === currentLocation?.id}
              >
                <Move size={16} aria-hidden="true" /> 移动到这里
              </button>
              <button
                type="button"
                onClick={() => setCurrentAreaFromLocation(inspectedLocation)}
                disabled={!inspectedLocation.map_area_id}
              >
                <LocateFixed size={16} aria-hidden="true" /> 切换区域
              </button>
              <button type="button" onClick={() => pinLocation(inspectedLocation.id)}>
                <Pin size={16} aria-hidden="true" />{" "}
                {pinnedLocationId === inspectedLocation.id ? "取消关注" : "设为关注"}
              </button>
            </div>
          </section>
        ) : null}

        <section className="terminal-location-list" aria-label="move destinations">
          {visibleLocations.map((location) => {
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
                disabled={loading}
                aria-label={`查看 ${locationName(location)}`}
              >
                <span className="terminal-location-symbol">
                  {displayText(location.ascii_symbol) || locationValue(location.legacy_place_id)}
                </span>
                <span>{locationName(location)}</span>
                {occupants.length > 0 ? (
                  <small>{occupants.map((item) => characterName(item)).join("、")}</small>
                ) : null}
              </button>
            );
          })}
        </section>

        <section className="terminal-character-strip" aria-label="current location characters">
          <div className="terminal-panel-heading">
            <h2>
              <UserRound size={17} aria-hidden="true" /> 当前位置人物
            </h2>
            <span>{locationName(currentLocation)}</span>
          </div>
          {currentLocationCharacters.length > 0 ? (
            <>
              <div className="terminal-character-list">
                {currentLocationCharacters.map((character) => (
                  <button
                    key={character.id}
                    type="button"
                    className={character.id === selectedCharacter?.id ? "current" : undefined}
                    onClick={() => setSelectedCharacterId(character.id)}
                    disabled={loading}
                    aria-pressed={character.id === selectedCharacter?.id}
                  >
                    {characterName(character)}
                  </button>
                ))}
              </div>
              {selectedCharacter ? (
                <div className="terminal-character-detail">
                  <div className="terminal-portrait" aria-label="character portrait">
                    {canRenderPortrait && selectedPortrait ? (
                      <img
                        src={selectedPortrait.source_path}
                        alt={`${characterName(selectedCharacter)} 立绘`}
                      />
                    ) : (
                      <div className="terminal-portrait-fallback">
                        <span>{Array.from(characterName(selectedCharacter))[0] ?? "人"}</span>
                        <small>
                          {selectedPortrait
                            ? displayText(selectedPortrait.resource_id)
                            : "未绑定立绘"}
                        </small>
                      </div>
                    )}
                  </div>
                  <dl>
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
                      <dd>{selectedCharacterRelationship?.affinity ?? "未知"}</dd>
                    </div>
                  </dl>
                </div>
              ) : null}
            </>
          ) : (
            <p className="terminal-empty-text">此处暂无人物。</p>
          )}
        </section>

        <div className="terminal-rule" />
        <ol className="terminal-log" aria-label="event log">
          {world.event_log.slice(0, 8).map((entry, index) => (
            <li key={`${entry}-${index}`}>{displayText(entry)}</li>
          ))}
        </ol>
      </section>
    </div>
  );
};
