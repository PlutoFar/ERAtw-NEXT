import { useState } from "react";
import {
  Boxes,
  MessageSquareText,
  RotateCcw,
  Save,
  Trash2,
} from "lucide-react";
import type { ModInstallActionReport } from "../../types";
import { SAVE_SLOTS, type ShellServices } from "./shellTypes";

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

interface PanelServicesProps {
  services: ShellServices;
}

export const SaveLoadPanel = ({ services }: PanelServicesProps) => (
  <section className="menu-panel" aria-label="save load panel">
    <h2>存取档</h2>
    <div className="save-slot-panel" aria-label="save slots">
      {SAVE_SLOTS.map((slotId, index) => (
        <button
          key={slotId}
          type="button"
          className={services.selectedSlotId === slotId ? "active-slot" : undefined}
          onClick={() => services.setSelectedSlotId(slotId)}
          disabled={services.loading}
          aria-pressed={services.selectedSlotId === slotId}
        >
          槽位 {index + 1}
        </button>
      ))}
    </div>
    <p className="slot-text">当前槽位：{services.selectedSlotId}</p>
    <div className="menu-action-grid">
      <button
        type="button"
        onClick={() => services.saveSlot(services.selectedSlotId)}
        disabled={services.loading}
      >
        <Save size={16} aria-hidden="true" /> 保存
      </button>
      <button
        type="button"
        onClick={() => services.preflightLoadSlot(services.selectedSlotId)}
        disabled={services.loading}
      >
        预检读取
      </button>
      {services.lastLoadPreflight?.slot_id === services.selectedSlotId &&
      services.lastLoadPreflight.ready ? (
        <button
          type="button"
          onClick={() => services.loadSlot(services.selectedSlotId)}
          disabled={services.loading}
        >
          确认读取
        </button>
      ) : null}
      <button
        type="button"
        onClick={() => services.recoverSlot(services.selectedSlotId)}
        disabled={services.loading}
      >
        <RotateCcw size={16} aria-hidden="true" /> 恢复
      </button>
    </div>
  </section>
);

export const ModPanel = ({ services }: PanelServicesProps) => (
  <section className="menu-panel" aria-label="mod panel">
    <h2>Mod</h2>
    <div className="menu-action-grid">
      <button
        type="button"
        onClick={services.installSamplePackage}
        disabled={services.loading}
      >
        <MessageSquareText size={16} aria-hidden="true" /> 示例包
      </button>
      <button
        type="button"
        onClick={() =>
          services.preflightModPackageInstall(
            services.modPackageRoot,
            services.modInstallRoot,
          )
        }
        disabled={services.loading}
      >
        <MessageSquareText size={16} aria-hidden="true" /> Mod 预检
      </button>
      <button
        type="button"
        onClick={() => services.refreshInstalledMods(services.modInstallRoot)}
        disabled={services.loading}
      >
        <Boxes size={16} aria-hidden="true" /> 已装 Mod
      </button>
    </div>

    {services.lastModPackagePreflight ? (
      <section className="mod-preflight-panel" aria-label="mod package preflight">
        <h3>Mod 包预检</h3>
        <p
          className={
            services.lastModPackagePreflight.ready
              ? "preflight-text"
              : "preflight-error-text"
          }
        >
          {services.lastModPackagePreflight.ready ? "可安装" : "已阻止"}
          {services.lastModPackagePreflight.manifest
            ? `：${services.lastModPackagePreflight.manifest.namespace}`
            : ""}
          <br />
          包：{services.lastModPackagePreflight.source_root}
          <br />
          目标：{services.lastModPackagePreflight.target_root ?? "未生成"}
        </p>
        {services.lastModPackagePreflight.issues.length > 0 ? (
          <ul className="mod-preflight-issues">
            {services.lastModPackagePreflight.issues.map((issue) => (
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
        {services.lastModPackagePreflight.ready ? (
          <button
            type="button"
            className="mod-install-button"
            onClick={() =>
              services.installModPackage(
                services.lastModPackagePreflight!.source_root,
                services.lastModPackagePreflight!.install_root,
              )
            }
            disabled={services.loading}
          >
            安装 Mod 包
          </button>
        ) : null}
      </section>
    ) : null}

    {services.lastModInstall ? (
      <p className="mod-install-text" aria-label="mod install result">
        已安装 Mod：{services.lastModInstall.manifest.namespace}
        <br />
        目标：{services.lastModInstall.target_root}
      </p>
    ) : null}

    {services.lastModUninstall ? (
      <p className="mod-install-text" aria-label="mod uninstall result">
        已卸载 Mod：{services.lastModUninstall.namespace}
        <br />
        目标：{services.lastModUninstall.target_root}
      </p>
    ) : null}

    {services.lastInstalledMods ? (
      <section className="installed-mods-panel" aria-label="installed mods">
        <h3>已安装 Mod</h3>
        <p className="preflight-text">根目录：{services.lastInstalledMods.root_path}</p>
        {services.lastInstalledMods.discovered.length > 0 ? (
          <ul className="installed-mods-list">
            {services.lastInstalledMods.discovered.map((entry) => (
              <li key={entry.manifest.namespace}>
                <div className="mod-row-main">
                  <label className="mod-toggle">
                    <input
                      type="checkbox"
                      checked={
                        services.modEnablement.find(
                          (selection) =>
                            selection.namespace === entry.manifest.namespace,
                        )?.enabled ?? true
                      }
                      onChange={(event) =>
                        services.setModEnabled(
                          entry.manifest.namespace,
                          event.currentTarget.checked,
                          services.modInstallRoot,
                        )
                      }
                      disabled={services.loading}
                      aria-label={`启用 ${entry.manifest.namespace}`}
                    />
                    <strong>{entry.manifest.name}</strong>
                  </label>
                  <button
                    type="button"
                    className="mod-uninstall-button"
                    onClick={() =>
                      services.planModUninstall(
                        services.modInstallRoot,
                        entry.manifest.namespace,
                      )
                    }
                    disabled={services.loading}
                    aria-label={`卸载 ${entry.manifest.namespace}`}
                  >
                    <Trash2 size={15} aria-hidden="true" /> 卸载
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

        {services.lastModUninstallPlan ? (
          <div className="mod-uninstall-plan" aria-label="mod uninstall plan">
            <h3>卸载预检</h3>
            <p className="preflight-error-text">
              待卸载：{services.lastModUninstallPlan.namespace}
              <br />
              目标：{services.lastModUninstallPlan.target_root}
              <br />
              临时目录：{services.lastModUninstallPlan.staging_root}
            </p>
            <ul className="mod-preflight-issues">
              {services.lastModUninstallPlan.actions.map((action, index) => (
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
                services.uninstallInstalledMod(
                  services.lastModUninstallPlan!.install_root,
                  services.lastModUninstallPlan!.namespace,
                )
              }
              disabled={services.loading}
            >
              <Trash2 size={15} aria-hidden="true" /> 确认卸载
            </button>
          </div>
        ) : null}

        {services.lastModEnablementPlan ? (
          <div className="mod-enablement-plan" aria-label="mod enablement plan">
            <strong>启用顺序</strong>
            {services.lastModEnablementPlan.enabled.length > 0 ? (
              <ol>
                {services.lastModEnablementPlan.enabled.map((manifest) => (
                  <li key={manifest.namespace}>{manifest.namespace}</li>
                ))}
              </ol>
            ) : (
              <p className="empty-text">无启用 Mod。</p>
            )}
            {services.lastModEnablementPlan.disabled.length > 0 ? (
              <p>
                禁用：
                {services.lastModEnablementPlan.disabled
                  .map((entry) => entry.manifest.namespace)
                  .join("、")}
              </p>
            ) : null}
          </div>
        ) : null}

        {services.lastInstalledMods.errors.length > 0 ? (
          <ul className="mod-preflight-issues">
            {services.lastInstalledMods.errors.map((issue) => (
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
  </section>
);

export const SettingsPanel = () => {
  const [scanlines, setScanlines] = useState(true);
  const [textGlow, setTextGlow] = useState(false);
  const [uiScale, setUiScale] = useState(100);

  return (
    <section className="menu-panel" aria-label="settings panel">
      <h2>设置</h2>
      <label className="settings-row">
        <span>扫描线</span>
        <input
          type="checkbox"
          checked={scanlines}
          onChange={(event) => setScanlines(event.currentTarget.checked)}
        />
      </label>
      <label className="settings-row">
        <span>文字辉光</span>
        <input
          type="checkbox"
          checked={textGlow}
          onChange={(event) => setTextGlow(event.currentTarget.checked)}
        />
      </label>
      <label className="settings-row">
        <span>界面比例</span>
        <input
          type="range"
          min="90"
          max="120"
          step="5"
          value={uiScale}
          onChange={(event) => setUiScale(Number(event.currentTarget.value))}
        />
        <output>{uiScale}%</output>
      </label>
    </section>
  );
};

export const StatusMessages = ({ services }: PanelServicesProps) => (
  <section className="status-messages" aria-label="system messages">
    {services.error ? <p className="error-text">{services.error}</p> : null}
    {services.lastSave ? (
      <p className="save-text">
        已保存：{services.lastSave.path}
        {services.lastSave.backup_path ? (
          <>
            <br />
            上次存档已备份：{services.lastSave.backup_path}
          </>
        ) : null}
      </p>
    ) : null}
    {services.lastLoadPreflight?.slot_id === services.selectedSlotId ? (
      <p
        className={
          services.lastLoadPreflight.ready ? "preflight-text" : "preflight-error-text"
        }
      >
        读档预检：{services.lastLoadPreflight.ready ? "可读取" : "已阻止"}
        <br />
        路径：{services.lastLoadPreflight.path}
        {services.lastLoadPreflight.validation.missing_required_mods.length > 0 ? (
          <>
            <br />
            缺少 Mod：
            {services.lastLoadPreflight.validation.missing_required_mods
              .map((dependency) => `${dependency.namespace}@${dependency.version}`)
              .join("、")}
          </>
        ) : null}
        {services.lastLoadPreflight.validation.incompatible_schema !== null ? (
          <>
            <br />
            不兼容 schema：{services.lastLoadPreflight.validation.incompatible_schema}
          </>
        ) : null}
        {services.lastLoadPreflight.validation.engine_version_mismatch ? (
          <>
            <br />
            引擎版本不同：{services.lastLoadPreflight.save.engine_version}
          </>
        ) : null}
      </p>
    ) : null}
    {services.lastRecovery ? (
      <p className="recovery-text">
        已恢复：{services.lastRecovery.path}
        <br />
        来源：{services.lastRecovery.recovered_from}
        {services.lastRecovery.failed_primary_backup_path ? (
          <>
            <br />
            失败主档已备份：{services.lastRecovery.failed_primary_backup_path}
          </>
        ) : null}
      </p>
    ) : null}
  </section>
);
