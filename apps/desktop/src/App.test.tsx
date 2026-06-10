import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { App } from "./App";
import { createBrowserMockEngineClient } from "./engine/client";
import { DEFAULT_MOD_INSTALL_ROOT, useEngine } from "./engine/useEngine";

describe("App", () => {
  beforeEach(() => {
    window.localStorage.clear();
    useEngine.setState({
      client: createBrowserMockEngineClient(),
      world: null,
      loading: false,
      error: null,
      lastSave: null,
      lastLoadPreflight: null,
      lastRecovery: null,
      lastModPackagePreflight: null,
      lastModInstall: null,
      lastModUninstallPlan: null,
      lastModUninstall: null,
      lastInstalledMods: null,
      modEnablement: [],
      lastModEnablementPlan: null,
    });
  });

  it("renders the M0 shell and traditional view", async () => {
    render(<App />);

    expect(await screen.findByText("ERAtw-NEXT")).toBeInTheDocument();
    expect(screen.getByLabelText("era text map")).toBeInTheDocument();
    expect(screen.getAllByText("示例角色").length).toBeGreaterThan(0);
    expect(screen.getAllByText("人里的门").length).toBeGreaterThan(0);
    expect(screen.queryByText(/人里的門/)).not.toBeInTheDocument();
  });

  it("loads the modern Pixi map only after selecting modern mode", async () => {
    render(<App />);

    expect(await screen.findByLabelText("era text map")).toBeInTheDocument();
    expect(screen.queryByLabelText("modern map canvas")).not.toBeInTheDocument();

    fireEvent.mouseDown(screen.getByRole("tab", { name: "现代" }), {
      button: 0,
      ctrlKey: false,
    });

    expect(await screen.findByLabelText("modern map canvas")).toBeInTheDocument();
  });

  it("dispatches dialogue command through the engine store", async () => {
    render(<App />);

    const dialogueButton = await screen.findByRole("button", { name: "对话" });
    fireEvent.click(dialogueButton);

    await waitFor(() => {
      expect(screen.getByText("询问新引擎")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "询问新引擎" }));

    await waitFor(() => {
      expect(screen.getByText(/不执行旧 ERB/)).toBeInTheDocument();
    });
  });

  it("saves through the engine store", async () => {
    render(<App />);

    expect(await screen.findByText("当前槽位：slot_1")).toBeInTheDocument();
    const saveButton = await screen.findByRole("button", { name: "保存" });
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(screen.getByText(/browser-memory:\/\/slot_1.json/)).toBeInTheDocument();
    });
  });

  it("saves and loads the selected slot independently", async () => {
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "槽位 2" }));
    expect(screen.getByText("当前槽位：slot_2")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "查看 广场" }));
    fireEvent.click(
      within(screen.getByLabelText("location details")).getByRole("button", {
        name: /移动到这里/,
      }),
    );
    await waitFor(() => {
      expect(screen.getAllByText("广场").length).toBeGreaterThan(0);
    });

    fireEvent.click(screen.getByRole("button", { name: "保存" }));
    await waitFor(() => {
      expect(screen.getByText(/browser-memory:\/\/slot_2.json/)).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "查看 人里的门" }));
    fireEvent.click(
      within(screen.getByLabelText("location details")).getByRole("button", {
        name: /移动到这里/,
      }),
    );
    await waitFor(() => {
      expect(screen.getAllByText("人里的门").length).toBeGreaterThan(0);
    });

    fireEvent.click(screen.getByRole("button", { name: "预检读取" }));
    await waitFor(() => {
      expect(screen.getByText(/读档预检：可读取/)).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "确认读取" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "确认读取" }));
    await waitFor(() => {
      expect(screen.getAllByText("广场").length).toBeGreaterThan(0);
    });
  });

  it("recovers the current slot from the latest backup", async () => {
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "保存" }));
    await waitFor(() => {
      expect(screen.getByText(/browser-memory:\/\/slot_1.json/)).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: /休息/ }));
    await waitFor(() => {
      expect(screen.getAllByText(/08:30/).length).toBeGreaterThan(0);
    });

    fireEvent.click(screen.getByRole("button", { name: "保存" }));
    await waitFor(() => {
      expect(screen.getByText(/slot_1\.json\./)).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: /休息/ }));
    await waitFor(() => {
      expect(screen.getAllByText(/09:00/).length).toBeGreaterThan(0);
    });

    fireEvent.click(screen.getByRole("button", { name: /恢复/ }));

    await waitFor(() => {
      expect(screen.getByText(/已恢复：browser-memory:\/\/slot_1.json/)).toBeInTheDocument();
      expect(screen.getAllByText(/08:00/).length).toBeGreaterThan(0);
      expect(screen.queryAllByText(/09:00/)).toHaveLength(0);
    });
  });

  it("installs the sample content package through the engine store", async () => {
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /示例包/ }));

    await waitFor(() => {
      expect(screen.getByText(/内容包 sample:sample.event_pack 已加载/)).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: /休息/ }));

    await waitFor(() => {
      expect(screen.getByText(/随内容包新增的角色/)).toBeInTheDocument();
    });
  });

  it("shows hover location details for text-map targets", async () => {
    render(<App />);

    fireEvent.mouseEnter(await screen.findByRole("button", { name: "人里的门" }));

    const tooltip = screen.getByRole("tooltip");
    expect(within(tooltip).getByText("人里的门")).toBeInTheDocument();
    expect(within(tooltip).getByText("当前位置")).toBeInTheDocument();
    expect(within(tooltip).getByText(/示例角色/)).toBeInTheDocument();
  });

  it("opens a right-click location menu and moves through it", async () => {
    render(<App />);

    fireEvent.contextMenu(await screen.findByRole("button", { name: "查看 广场" }), {
      clientX: 90,
      clientY: 120,
    });

    const menu = screen.getByRole("menu", { name: /广场 操作菜单/ });
    fireEvent.click(within(menu).getByRole("menuitem", { name: /移动到这里/ }));

    await waitFor(() => {
      expect(within(screen.getByLabelText("world status")).getByText("广场")).toBeInTheDocument();
    });
  });

  it("shows current-location characters and a portrait fallback", async () => {
    render(<App />);

    const panel = await screen.findByLabelText("current location characters");
    expect(within(panel).getAllByText("示例角色").length).toBeGreaterThan(0);
    expect(within(panel).getByLabelText("character portrait")).toBeInTheDocument();
    expect(within(panel).getByText("core.demo.heroine.neutral")).toBeInTheDocument();
  });

  it("preflights a mod package and shows resource warnings", async () => {
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /Mod 预检/ }));

    await waitFor(() => {
      expect(screen.getByLabelText("mod package preflight")).toBeInTheDocument();
      expect(screen.getByText(/可安装：example\.minimal_character/)).toBeInTheDocument();
      expect(screen.getByText(/resource sha256 is missing/)).toBeInTheDocument();
      expect(
        screen.getByText(
          "packages/example.minimal_character-0.1.0/content/assets/readme.txt",
        ),
      ).toBeInTheDocument();
    });
  });

  it("installs a preflighted mod package", async () => {
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /Mod 预检/ }));
    const installButton = await screen.findByRole("button", {
      name: "安装 Mod 包",
    });
    fireEvent.click(installButton);

    await waitFor(() => {
      const installResult = screen.getByLabelText("mod install result");
      expect(
        within(installResult).getByText(/已安装 Mod：example\.minimal_character/),
      ).toBeInTheDocument();
      expect(
        within(installResult).getByText(
          /目标：mods\/installed\/example\.minimal_character/,
        ),
      ).toBeInTheDocument();
      expect(screen.getByLabelText("installed mods")).toBeInTheDocument();
      expect(screen.getByText("最小角色 Mod")).toBeInTheDocument();
      expect(
        screen.getByText("example.minimal_character@0.1.0"),
      ).toBeInTheDocument();
      expect(screen.getByLabelText("mod enablement plan")).toBeInTheDocument();
      expect(screen.getByText("启用顺序")).toBeInTheDocument();
    });
  });

  it("refreshes the installed mod list", async () => {
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /已装 Mod/ }));

    await waitFor(() => {
      const panel = screen.getByLabelText("installed mods");
      expect(within(panel).getByText("根目录：mods/installed")).toBeInTheDocument();
      expect(within(panel).getByText("未发现已安装 Mod。")).toBeInTheDocument();
    });
  });

  it("disables an installed mod in the enablement plan", async () => {
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /Mod 预检/ }));
    fireEvent.click(await screen.findByRole("button", { name: "安装 Mod 包" }));

    const toggle = await screen.findByRole("checkbox", {
      name: "启用 example.minimal_character",
    });
    expect(toggle).toBeChecked();
    fireEvent.click(toggle);

    await waitFor(() => {
      expect(toggle).not.toBeChecked();
      const plan = screen.getByLabelText("mod enablement plan");
      expect(within(plan).getByText(/无启用 Mod/)).toBeInTheDocument();
      expect(
        within(plan).getByText(/禁用：example\.minimal_character/),
      ).toBeInTheDocument();
    });
  });

  it("restores persisted mod enablement for installed mods", async () => {
    await useEngine.getState().client.saveModEnablement(DEFAULT_MOD_INSTALL_ROOT, [
      { namespace: "example.minimal_character", enabled: false },
    ]);
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /Mod 预检/ }));
    fireEvent.click(await screen.findByRole("button", { name: "安装 Mod 包" }));

    const toggle = await screen.findByRole("checkbox", {
      name: "启用 example.minimal_character",
    });
    const plan = screen.getByLabelText("mod enablement plan");

    expect(toggle).not.toBeChecked();
    expect(within(plan).getByText(/禁用：example\.minimal_character/)).toBeInTheDocument();
  });

  it("uninstalls an installed mod and refreshes the installed list", async () => {
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /Mod 预检/ }));
    fireEvent.click(await screen.findByRole("button", { name: "安装 Mod 包" }));

    const uninstallButton = await screen.findByRole("button", {
      name: "卸载 example.minimal_character",
    });
    fireEvent.click(uninstallButton);

    const uninstallPlan = await screen.findByLabelText("mod uninstall plan");
    expect(within(uninstallPlan).getByText("卸载预检")).toBeInTheDocument();
    expect(
      within(uninstallPlan).getByText(/目标：mods\/installed\/example\.minimal_character/),
    ).toBeInTheDocument();
    expect(screen.queryByLabelText("mod uninstall result")).not.toBeInTheDocument();

    fireEvent.click(within(uninstallPlan).getByRole("button", { name: /确认卸载/ }));

    await waitFor(() => {
      const uninstallResult = screen.getByLabelText("mod uninstall result");
      expect(
        within(uninstallResult).getByText(/已卸载 Mod：example\.minimal_character/),
      ).toBeInTheDocument();
      const panel = screen.getByLabelText("installed mods");
      expect(within(panel).getByText("未发现已安装 Mod。")).toBeInTheDocument();
      expect(
        screen.queryByText("example.minimal_character@0.1.0"),
      ).not.toBeInTheDocument();
    });
  });

  it("dispatches relationship command through the engine store", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getAllByText("好感").length).toBeGreaterThan(0);
    });
    expect(screen.getAllByText("信赖").length).toBeGreaterThan(0);

    const communicateButton = screen.getByRole("button", { name: /交流/ });
    fireEvent.click(communicateButton);

    await waitFor(() => {
      expect(screen.getAllByText("6").length).toBeGreaterThan(0);
      expect(screen.getAllByText("1").length).toBeGreaterThan(0);
    });
  });

  it("shows dialogue choices only when conditions pass", async () => {
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "对话" }));

    await waitFor(() => {
      expect(screen.getByText("询问新引擎")).toBeInTheDocument();
    });
    expect(screen.queryByText("谈谈信任")).not.toBeInTheDocument();

    const communicateButton = screen.getByRole("button", { name: /交流/ });
    fireEvent.click(communicateButton);
    await waitFor(() => {
      expect(screen.getAllByText("6").length).toBeGreaterThan(0);
    });

    fireEvent.click(communicateButton);

    await waitFor(() => {
      expect(screen.getByText("谈谈信任")).toBeInTheDocument();
    });
  });
});
