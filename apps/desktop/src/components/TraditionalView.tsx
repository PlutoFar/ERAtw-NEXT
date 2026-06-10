import { useMemo, useState } from "react";
import type { EngineCommand, TextMap, TextMapAction, WorldState } from "../types";

interface TraditionalViewProps {
  world: WorldState;
  dispatch: (command: EngineCommand) => void | Promise<void>;
  loading?: boolean;
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

const locationValue = (legacyPlaceId: number | null | undefined) =>
  legacyPlaceId === null || legacyPlaceId === undefined
    ? "??"
    : String(legacyPlaceId).slice(-2).padStart(2, "0");

const mapLocationIds = (textMap: TextMap | undefined) =>
  new Set(textMap?.locations.map((location) => location.location_id) ?? []);

export const TraditionalView = ({
  world,
  dispatch,
  loading = false,
}: TraditionalViewProps) => {
  const character = world.characters[0];
  const relationship = world.relationships.find(
    (item) =>
      item.source_character_id === "player" &&
      item.target_character_id === character?.id,
  );
  const currentLocation = world.locations.find(
    (location) => location.id === character?.location_id,
  );
  const textMap = world.text_maps[0];
  const locationIds = useMemo(() => mapLocationIds(textMap), [textMap]);
  const [selectedAreaId, setSelectedAreaId] = useState<string | undefined>();
  const [commandMode, setCommandMode] = useState<"root" | "move">("root");

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

  const moveTo = (locationId: string) => {
    if (!character || loading) {
      return;
    }
    const destination = world.locations.find((location) => location.id === locationId);
    if (destination?.map_area_id) {
      setSelectedAreaId(destination.map_area_id);
    }
    void dispatch({
      type: "move_character",
      character_id: character.id,
      location_id: locationId,
    });
  };

  const runAction = (action: TextMapAction) => {
    if (action.type === "move_to_location") {
      moveTo(action.location_id);
    } else if (action.type === "switch_area") {
      setSelectedAreaId(action.area_id);
    } else {
      setCommandMode("root");
    }
  };

  const startDialogue = () => {
    void dispatch({ type: "start_dialogue", scene_id: "demo_morning" });
  };

  const adjustRelationship = () => {
    if (!character) {
      return;
    }
    void dispatch({
      type: "adjust_relationship",
      source_character_id: "player",
      target_character_id: character.id,
      affinity_delta: 1,
      trust_delta: 1,
    });
  };

  const rollMood = () => {
    if (!character) {
      return;
    }
    void dispatch({
      type: "roll_character_mood",
      character_id: character.id,
      min_delta: -5,
      max_delta: 5,
    });
  };

  return (
    <div className="traditional-view">
      <section className="terminal-screen" aria-label="era traditional ui">
        <div className="terminal-line terminal-title">eraTheWorld TW NEXT</div>
        <div className="terminal-line">
          {formatClock(world)} / {seasonLabels[world.clock.season]} /{" "}
          {weatherLabels[world.clock.weather]} / {textMap?.name ?? "地图未载入"}
        </div>
        <div className="terminal-rule" />
        <div className="terminal-line">
          MASTER 位置：
          <span className="terminal-current">
            {currentLocation?.name ?? character?.location_id ?? "未知"}
          </span>
        </div>
        <div className="terminal-line">
          体力：{character?.state.energy ?? "--"} 心情：
          {character?.state.mood ?? "--"} 好感：{relationship?.affinity ?? "--"} 信赖：
          {relationship?.trust ?? "--"}
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
              [{area.kind === "outing" ? "外" : "内"}] {area.name}
            </button>
          ))}
        </div>

        <div className="terminal-map" aria-label="era text map">
          {activeArea ? (
            activeArea.rows.map((row, rowIndex) => (
              <div className="terminal-map-row" key={`${activeArea.id}:${rowIndex}`}>
                {row.runs.map((run, runIndex) => {
                  const action = run.action;
                  const isCurrent =
                    action?.type === "move_to_location" &&
                    action.location_id === currentLocation?.id;
                  if (action) {
                    return (
                      <button
                        key={`${rowIndex}:${runIndex}`}
                        type="button"
                        className={isCurrent ? "terminal-map-button current" : "terminal-map-button"}
                        style={{ color: isCurrent ? undefined : run.color ?? undefined }}
                        onClick={() => runAction(action)}
                        disabled={loading}
                        title={action.title ?? action.label}
                      >
                        {run.text}
                      </button>
                    );
                  }
                  return (
                    <span
                      key={`${rowIndex}:${runIndex}`}
                      style={{ color: run.color ?? undefined }}
                    >
                      {run.text}
                    </span>
                  );
                })}
              </div>
            ))
          ) : (
            <div className="terminal-map-row">NO TEXT MAP DATA</div>
          )}
        </div>

        <div className="terminal-rule" />
        {commandMode === "root" ? (
          <div className="terminal-command-grid" aria-label="era commands">
            <button
              type="button"
              onClick={() => dispatch({ type: "advance_time", minutes: 30 })}
              disabled={loading}
            >
              [000] 休息
            </button>
            <button type="button" onClick={startDialogue} disabled={loading}>
              [100] 对话
            </button>
            <button type="button" onClick={rollMood} disabled={loading}>
              [200] 随机心情
            </button>
            <button type="button" onClick={adjustRelationship} disabled={loading}>
              [300] 交流
            </button>
            <button
              type="button"
              onClick={() => setCommandMode("move")}
              disabled={loading}
            >
              [400] 移动
            </button>
          </div>
        ) : (
          <div className="terminal-move-list" aria-label="era move destinations">
            <button type="button" onClick={() => setCommandMode("root")}>
              [999] 返回
            </button>
            {visibleLocations.map((location) => (
              <button
                key={location.id}
                type="button"
                className={location.id === currentLocation?.id ? "current" : undefined}
                onClick={() => moveTo(location.id)}
                disabled={loading}
              >
                [{locationValue(location.legacy_place_id)}] {location.name}
              </button>
            ))}
          </div>
        )}

        <div className="terminal-rule" />
        <ol className="terminal-log" aria-label="event log">
          {world.event_log.slice(0, 8).map((entry, index) => (
            <li key={`${entry}-${index}`}>{entry}</li>
          ))}
        </ol>
      </section>
    </div>
  );
};
