import { lazy, Suspense, useEffect, useState } from "react";
import * as Tabs from "@radix-ui/react-tabs";
import {
  Boxes,
  Clock3,
  CloudSun,
  MessageSquareText,
  RotateCcw,
  Trash2,
} from "lucide-react";
import { DEFAULT_MOD_INSTALL_ROOT, useEngine } from "./engine/useEngine";
import { visibleChoices } from "./engine/demoWorld";
import { createSampleContentPackage } from "./engine/sampleContentPackage";
import { displayText } from "./engine/displayText";
import { TraditionalView } from "./components/TraditionalView";
import type { Location, ModInstallActionReport, WorldState } from "./types";

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

const modInstallActionLabels: Record<ModInstallActionReport["kind"], string> = {
  create_directory: "创建目录",
  copy_directory: "复制目录",
  move_directory: "移动目录",
  delete_directory: "删除目录",
};

const formatModInstallActionTarget = (action: ModInstallActionReport) => {
  if (action.path) {
    return action.path;
  }
  if (action.from && action.to) {
    return `${action.from} -> ${action.to}`;
  }
  return "无路径";
};

const SAVE_SLOTS = ["slot_1", "slot_2", "slot_3"] as const;

export const App = () => {
  const {
    dispatch,
    error,
    installContentPackage,
    lastLoadPreflight,
    lastRecovery,
    lastSave,
    lastModInstall,
    lastModUninstallPlan,
    lastModUninstall,
    lastInstalledMods,
    lastModEnablementPlan,
    modEnablement,
    load,
    loadSlot,
    loading,
    lastModPackagePreflight,
    preflightLoadSlot,
    preflightModPackageInstall,
    installModPackage,
    planModUninstall,
    refreshInstalledMods,
    setModEnabled,
    uninstallInstalledMod,
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
          <span>{displayText(currentLocation?.name, "未知地点")}</span>
        </div>
      </header>

      <section className="workspace">
        <Tabs.Root defaultValue="traditional" className="mode-tabs">
          <Tabs.List className="mode-list" aria-label="UI mode">
            <Tabs.Trigger value="traditional">传统</Tabs.Trigger>
            <Tabs.Trigger value="modern">现代</Tabs.Trigger>
          </Tabs.List>

          <Tabs.Content value="traditional" className="mode-panel">
            <TraditionalView world={world} dispatch={dispatch} loading={loading} />
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
                <dd>
                  {currentLocation
                    ? displayText(currentLocation.name)
                    : character.location_id}
                </dd>
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
              onClick={() => installContentPackage(createSampleContentPackage())}
              disabled={loading}
            >
              <MessageSquareText size={17} /> 示例包
            </button>
            <button
              type="button"
              onClick={() =>
                preflightModPackageInstall(
                  "packages/example.minimal_character-0.1.0",
                  DEFAULT_MOD_INSTALL_ROOT,
                )
              }
              disabled={loading}
            >
              <MessageSquareText size={17} /> Mod 预检
            </button>
            <button
              type="button"
              onClick={() => refreshInstalledMods(DEFAULT_MOD_INSTALL_ROOT)}
              disabled={loading}
            >
              <Boxes size={17} /> 已装 Mod
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
              onClick={() => preflightLoadSlot(selectedSlotId)}
              disabled={loading}
            >
              预检读取
            </button>
            {lastLoadPreflight?.slot_id === selectedSlotId && lastLoadPreflight.ready ? (
              <button
                type="button"
                onClick={() => loadSlot(selectedSlotId)}
                disabled={loading}
              >
                确认读取
              </button>
            ) : null}
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
          {lastLoadPreflight?.slot_id === selectedSlotId ? (
            <p
              className={
                lastLoadPreflight.ready ? "preflight-text" : "preflight-error-text"
              }
            >
              读档预检：{lastLoadPreflight.ready ? "可读取" : "已阻止"}
              <br />
              路径：{lastLoadPreflight.path}
              {lastLoadPreflight.validation.missing_required_mods.length > 0 ? (
                <>
                  <br />
                  缺少 Mod：
                  {lastLoadPreflight.validation.missing_required_mods
                    .map((dependency) => `${dependency.namespace}@${dependency.version}`)
                    .join("、")}
                </>
              ) : null}
              {lastLoadPreflight.validation.incompatible_schema !== null ? (
                <>
                  <br />
                  不兼容 schema：{lastLoadPreflight.validation.incompatible_schema}
                </>
              ) : null}
              {lastLoadPreflight.validation.engine_version_mismatch ? (
                <>
                  <br />
                  引擎版本不同：{lastLoadPreflight.save.engine_version}
                </>
              ) : null}
            </p>
          ) : null}
          {lastModPackagePreflight ? (
            <section
              className="mod-preflight-panel"
              aria-label="mod package preflight"
            >
              <h2>Mod 包预检</h2>
              <p
                className={
                  lastModPackagePreflight.ready
                    ? "preflight-text"
                    : "preflight-error-text"
                }
              >
                {lastModPackagePreflight.ready ? "可安装" : "已阻止"}
                {lastModPackagePreflight.manifest
                  ? `：${lastModPackagePreflight.manifest.namespace}`
                  : ""}
                <br />
                包：{lastModPackagePreflight.source_root}
                <br />
                目标：{lastModPackagePreflight.target_root ?? "未生成"}
              </p>
              {lastModPackagePreflight.issues.length > 0 ? (
                <ul className="mod-preflight-issues">
                  {lastModPackagePreflight.issues.map((issue) => (
                    <li key={`${issue.kind}:${issue.path}:${issue.message}`}>
                      <strong>{issue.severity === "error" ? "错误" : "警告"}</strong>
                      <span>{issue.message}</span>
                      <small>{issue.path}</small>
                    </li>
                  ))}
                </ul>
              ) : (
                <p className="empty-text">没有预检问题。</p>
              )}
              {lastModPackagePreflight.ready ? (
                <button
                  type="button"
                  className="mod-install-button"
                  onClick={() =>
                    installModPackage(
                      lastModPackagePreflight.source_root,
                      lastModPackagePreflight.install_root,
                    )
                  }
                  disabled={loading}
                >
                  安装 Mod 包
                </button>
              ) : null}
            </section>
          ) : null}
          {lastModInstall ? (
            <p className="mod-install-text" aria-label="mod install result">
              已安装 Mod：{lastModInstall.manifest.namespace}
              <br />
              目标：{lastModInstall.target_root}
            </p>
          ) : null}
          {lastModUninstall ? (
            <p className="mod-install-text" aria-label="mod uninstall result">
              已卸载 Mod：{lastModUninstall.namespace}
              <br />
              目标：{lastModUninstall.target_root}
            </p>
          ) : null}
          {lastInstalledMods ? (
            <section className="installed-mods-panel" aria-label="installed mods">
              <h2>已安装 Mod</h2>
              <p className="preflight-text">根目录：{lastInstalledMods.root_path}</p>
              {lastInstalledMods.discovered.length > 0 ? (
                <ul className="installed-mods-list">
                  {lastInstalledMods.discovered.map((entry) => (
                    <li key={entry.manifest.namespace}>
                      <div className="mod-row-main">
                        <label className="mod-toggle">
                          <input
                            type="checkbox"
                            checked={
                              modEnablement.find(
                                (selection) =>
                                  selection.namespace === entry.manifest.namespace,
                              )?.enabled ?? true
                            }
                            onChange={(event) =>
                              setModEnabled(
                                entry.manifest.namespace,
                                event.currentTarget.checked,
                                DEFAULT_MOD_INSTALL_ROOT,
                              )
                            }
                            disabled={loading}
                            aria-label={`启用 ${entry.manifest.namespace}`}
                          />
                          <strong>{entry.manifest.name}</strong>
                        </label>
                        <button
                          type="button"
                          className="mod-uninstall-button"
                          onClick={() =>
                            planModUninstall(
                              DEFAULT_MOD_INSTALL_ROOT,
                              entry.manifest.namespace,
                            )
                          }
                          disabled={loading}
                          aria-label={`卸载 ${entry.manifest.namespace}`}
                        >
                          <Trash2 size={15} /> 卸载
                        </button>
                      </div>
                      <span>
                        {entry.manifest.namespace}@{entry.manifest.version}
                      </span>
                      <small>{entry.root_path}</small>
                    </li>
                  ))}
                </ul>
              ) : (
                <p className="empty-text">未发现已安装 Mod。</p>
              )}
              {lastModUninstallPlan ? (
                <div className="mod-uninstall-plan" aria-label="mod uninstall plan">
                  <h3>卸载预检</h3>
                  <p className="preflight-error-text">
                    待卸载：{lastModUninstallPlan.namespace}
                    <br />
                    目标：{lastModUninstallPlan.target_root}
                    <br />
                    临时目录：{lastModUninstallPlan.staging_root}
                  </p>
                  <ul className="mod-preflight-issues">
                    {lastModUninstallPlan.actions.map((action, index) => (
                      <li key={`${action.kind}:${index}`}>
                        <strong>{modInstallActionLabels[action.kind]}</strong>
                        <span>{formatModInstallActionTarget(action)}</span>
                      </li>
                    ))}
                  </ul>
                  <button
                    type="button"
                    className="mod-danger-button"
                    onClick={() =>
                      uninstallInstalledMod(
                        lastModUninstallPlan.install_root,
                        lastModUninstallPlan.namespace,
                      )
                    }
                    disabled={loading}
                  >
                    <Trash2 size={15} /> 确认卸载
                  </button>
                </div>
              ) : null}
              {lastModEnablementPlan ? (
                <div
                  className="mod-enablement-plan"
                  aria-label="mod enablement plan"
                >
                  <strong>启用顺序</strong>
                  {lastModEnablementPlan.enabled.length > 0 ? (
                    <ol>
                      {lastModEnablementPlan.enabled.map((manifest) => (
                        <li key={manifest.namespace}>{manifest.namespace}</li>
                      ))}
                    </ol>
                  ) : (
                    <p className="empty-text">无启用 Mod。</p>
                  )}
                  {lastModEnablementPlan.disabled.length > 0 ? (
                    <p>
                      禁用：
                      {lastModEnablementPlan.disabled
                        .map((entry) => entry.manifest.namespace)
                        .join("、")}
                    </p>
                  ) : null}
                </div>
              ) : null}
              {lastInstalledMods.errors.length > 0 ? (
                <ul className="mod-preflight-issues">
                  {lastInstalledMods.errors.map((issue) => (
                    <li key={`${issue.kind}:${issue.path}:${issue.message}`}>
                      <strong>错误</strong>
                      <span>{issue.message}</span>
                      <small>{issue.path}</small>
                    </li>
                  ))}
                </ul>
              ) : null}
            </section>
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
