import { lazy, Suspense, useEffect, useState } from "react";
import * as Tabs from "@radix-ui/react-tabs";
import {
  Clock3,
  CloudSun,
  Dices,
  MessageSquareText,
  MoveRight,
  RotateCcw,
} from "lucide-react";
import { useEngine } from "./engine/useEngine";
import { visibleChoices } from "./engine/demoWorld";
import { createSampleContentPackage } from "./engine/sampleContentPackage";
import { TraditionalView } from "./components/TraditionalView";
import type { Location, WorldState } from "./types";

const ModernMap = lazy(() =>
  import("./components/ModernMap").then((module) => ({
    default: module.ModernMap,
  })),
);

const formatClock = (world: WorldState) =>
  `第 ${world.clock.day} 日 ${String(world.clock.hour).padStart(2, "0")}:${String(
    world.clock.minute,
  ).padStart(2, "0")}`;

const seasonLabels = {
  spring: "春",
  summer: "夏",
  autumn: "秋",
  winter: "冬",
};

const weatherLabels = {
  clear: "晴",
  cloudy: "阴",
  rain: "雨",
  snow: "雪",
};

const getCurrentLocation = (world: WorldState): Location | undefined => {
  const character = world.characters[0];
  return world.locations.find((location) => location.id === character?.location_id);
};

const getPlayerRelationship = (world: WorldState) => {
  const character = world.characters[0];
  return world.relationships.find(
    (relationship) =>
      relationship.source_character_id === "player" &&
      relationship.target_character_id === character?.id,
  );
};

const formatScheduledEventTime = (world: WorldState) => {
  const event = world.scheduled_events[0];
  if (!event) {
    return "无待触发事件";
  }

  return `下个事件 D${event.due.day} ${String(event.due.hour).padStart(2, "0")}:${String(
    event.due.minute,
  ).padStart(2, "0")}`;
};

const SAVE_SLOTS = ["slot_1", "slot_2", "slot_3"] as const;

export const App = () => {
  const {
    dispatch,
    error,
    installContentPackage,
    lastRecovery,
    lastSave,
    load,
    loadSlot,
    loading,
    recoverSlot,
    saveSlot,
    world,
  } =
    useEngine();
  const [selectedSlotId, setSelectedSlotId] = useState<(typeof SAVE_SLOTS)[number]>(
    SAVE_SLOTS[0],
  );

  useEffect(() => {
    void load();
  }, [load]);

  if (!world) {
    return (
      <main className="app-shell">
        <div className="boot-panel">正在启动 ERAtw-NEXT engine mock...</div>
      </main>
    );
  }

  const character = world.characters[0];
  const currentLocation = getCurrentLocation(world);
  const relationship = getPlayerRelationship(world);

  return (
    <main className="app-shell">
      <header className="top-bar">
        <div>
          <h1>ERAtw-NEXT</h1>
          <p>独立 M0 原型 · engine {world.engine_version}</p>
        </div>
        <div className="status-strip" aria-label="world status">
          <span>
            <Clock3 size={16} /> {formatClock(world)}
          </span>
          <span>
            <CloudSun size={16} /> {seasonLabels[world.clock.season]} ·{" "}
            {weatherLabels[world.clock.weather]}
          </span>
          <span>{formatScheduledEventTime(world)}</span>
          <span>RNG {world.random.cursor}</span>
          <span>{currentLocation?.name ?? "未知地点"}</span>
        </div>
      </header>

      <section className="workspace">
        <Tabs.Root defaultValue="traditional" className="mode-tabs">
          <Tabs.List className="mode-list" aria-label="UI mode">
            <Tabs.Trigger value="traditional">传统</Tabs.Trigger>
            <Tabs.Trigger value="modern">现代</Tabs.Trigger>
          </Tabs.List>

          <Tabs.Content value="traditional" className="mode-panel">
            <TraditionalView world={world} />
          </Tabs.Content>
          <Tabs.Content value="modern" className="mode-panel">
            <Suspense
              fallback={
                <div className="modern-view">
                  <div className="pixi-host loading-map" aria-label="modern map loading" />
                </div>
              }
            >
              <ModernMap world={world} />
            </Suspense>
          </Tabs.Content>
        </Tabs.Root>

        <aside className="side-panel">
          <section className="character-panel">
            <h2>{character.display_name}</h2>
            <dl>
              <div>
                <dt>位置</dt>
                <dd>{currentLocation?.name ?? character.location_id}</dd>
              </div>
              <div>
                <dt>体力</dt>
                <dd>{character.state.energy}</dd>
              </div>
              <div>
                <dt>心情</dt>
                <dd>{character.state.mood}</dd>
              </div>
              <div>
                <dt>好感</dt>
                <dd>{relationship?.affinity ?? "未知"}</dd>
              </div>
              <div>
                <dt>信赖</dt>
                <dd>{relationship?.trust ?? "未知"}</dd>
              </div>
            </dl>
          </section>

          <section className="command-panel">
            <button
              type="button"
              onClick={() => dispatch({ type: "advance_time", minutes: 30 })}
              disabled={loading}
            >
              <Clock3 size={17} /> 休息
            </button>
            <button
              type="button"
              onClick={() =>
                dispatch({
                  type: "move_character",
                  character_id: character.id,
                  location_id:
                    character.location_id === "school_gate" ? "garden" : "school_gate",
                })
              }
              disabled={loading}
            >
              <MoveRight size={17} /> 移动
            </button>
            <button
              type="button"
              onClick={() =>
                dispatch({ type: "start_dialogue", scene_id: "demo_morning" })
              }
              disabled={loading}
            >
              <MessageSquareText size={17} /> 对话
            </button>
            <button
              type="button"
              onClick={() =>
                dispatch({
                  type: "roll_character_mood",
                  character_id: character.id,
                  min_delta: -5,
                  max_delta: 5,
                })
              }
              disabled={loading}
            >
              <Dices size={17} /> 随机心情
            </button>
            <button
              type="button"
              onClick={() =>
                dispatch({
                  type: "adjust_relationship",
                  source_character_id: "player",
                  target_character_id: character.id,
                  affinity_delta: 1,
                  trust_delta: 1,
                })
              }
              disabled={loading}
            >
              <MessageSquareText size={17} /> 交流
            </button>
            <button
              type="button"
              onClick={() => installContentPackage(createSampleContentPackage())}
              disabled={loading}
            >
              <MessageSquareText size={17} /> 示例包
            </button>
            <div className="save-slot-panel" aria-label="save slots">
              {SAVE_SLOTS.map((slotId, index) => (
                <button
                  key={slotId}
                  type="button"
                  className={selectedSlotId === slotId ? "active-slot" : undefined}
                  onClick={() => setSelectedSlotId(slotId)}
                  disabled={loading}
                  aria-pressed={selectedSlotId === slotId}
                >
                  槽位 {index + 1}
                </button>
              ))}
            </div>
            <p className="slot-text">当前槽位：{selectedSlotId}</p>
            <button
              type="button"
              onClick={() => saveSlot(selectedSlotId)}
              disabled={loading}
            >
              保存
            </button>
            <button
              type="button"
              onClick={() => loadSlot(selectedSlotId)}
              disabled={loading}
            >
              读取
            </button>
            <button
              type="button"
              onClick={() => recoverSlot(selectedSlotId)}
              disabled={loading}
            >
              <RotateCcw size={17} /> 恢复
            </button>
          </section>

          {error ? <p className="error-text">{error}</p> : null}
          {lastSave ? (
            <p className="save-text">
              已保存：{lastSave.path}
              {lastSave.backup_path ? (
                <>
                  <br />
                  上次存档已备份：{lastSave.backup_path}
                </>
              ) : null}
            </p>
          ) : null}
          {lastRecovery ? (
            <p className="recovery-text">
              已恢复：{lastRecovery.path}
              <br />
              来源：{lastRecovery.recovered_from}
              {lastRecovery.failed_primary_backup_path ? (
                <>
                  <br />
                  失败主档已备份：{lastRecovery.failed_primary_backup_path}
                </>
              ) : null}
            </p>
          ) : null}

          <section className="dialogue-panel" aria-label="dialogue">
            {world.active_dialogue.length === 0 ? (
              <p className="empty-text">暂无对话。</p>
            ) : (
              world.active_dialogue.map((node) => (
                <article key={node.id}>
                  <strong>{node.speaker_id}</strong>
                  <p>{node.text}</p>
                  {visibleChoices(world, node).length > 0 ? (
                    <div className="choice-list">
                      {visibleChoices(world, node).map((choice) => (
                        <button
                          key={choice.id}
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
                </article>
              ))
            )}
          </section>
        </aside>
      </section>
    </main>
  );
};
