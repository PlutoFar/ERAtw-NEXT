import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { App } from "./App";
import { buildAsciiMapModel } from "./components/traditional/viewModel";
import { createBrowserMockEngineClient } from "./engine/client";
import { createDemoWorld } from "./engine/demoWorld";
import { DEFAULT_MOD_INSTALL_ROOT, useEngine } from "./engine/useEngine";

const resetEngineStore = () => {
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
};

const enterGame = async () => {
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "开始" }));
  return screen.findByLabelText("game screen");
};

describe("App", () => {
  beforeEach(resetEngineStore);

  it("starts on the title screen and enters the game shell", async () => {
    render(<App />);

    expect(await screen.findByLabelText("title screen")).toBeInTheDocument();
    expect(screen.queryByLabelText("era text map")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "开始" }));

    expect(await screen.findByLabelText("game screen")).toBeInTheDocument();
    expect(screen.getByLabelText("game hud")).toBeInTheDocument();
    expect(screen.getByLabelText("quick actions")).toBeInTheDocument();
  });

  it("renders ASCII map text as fixed cells with separate overlay hotspots", async () => {
    await enterGame();

    const map = screen.getByLabelText("era text map");
    expect(map).toHaveClass("ascii-map-grid");
    expect(within(map).queryByRole("button")).not.toBeInTheDocument();
    expect(map.textContent).toContain("广场");
    expect(map.textContent).not.toContain("広场");
    expect(map.querySelectorAll(".ascii-map-cell").length).toBeGreaterThan(0);
    expect(map).toHaveAttribute("data-column-count");

    const hotspots = screen.getByLabelText("text map hotspots");
    expect(within(hotspots).getByRole("button", { name: "人里的门" })).toHaveAttribute(
      "data-location-id",
      "school_gate",
    );
  });

  it("keeps normalized map rows aligned and preserves location ids", () => {
    const world = createDemoWorld();
    const area = world.text_maps[0].areas[0];
    const model = buildAsciiMapModel(area);
    const sourceLengths = area.rows.map((row) =>
      row.runs.reduce((total, run) => total + Array.from(run.text).length, 0),
    );

    expect(model.lines.map((line) => Array.from(line).length)).toEqual(sourceLengths);
    expect(model.hotspots.find((hotspot) => hotspot.locationId === "school_gate"))
      .toMatchObject({ label: "人里的门", locationId: "school_gate" });
    expect(model.gridRows[0]).toHaveLength(sourceLengths[0]);
  });

  it("opens location details from a hotspot and moves by double click", async () => {
    await enterGame();

    const plazaHotspot = screen.getByRole("button", { name: "广场" });
    fireEvent.mouseEnter(plazaHotspot);
    expect(within(screen.getByRole("tooltip")).getByText("广场")).toBeInTheDocument();

    fireEvent.click(plazaHotspot);
    const drawer = screen.getByLabelText("location details");
    expect(within(drawer).getByText("广场")).toBeInTheDocument();

    fireEvent.doubleClick(plazaHotspot);
    await waitFor(() => {
      expect(within(screen.getByLabelText("game hud")).getByText("广场")).toBeInTheDocument();
    });
  });

  it("opens a right-click location menu and moves through it", async () => {
    await enterGame();

    fireEvent.contextMenu(screen.getByRole("button", { name: "广场" }), {
      clientX: 90,
      clientY: 120,
    });

    const menu = screen.getByRole("menu", { name: /广场 操作菜单/ });
    expect(within(menu).getByRole("menuitem", { name: /查看人物/ })).toBeInTheDocument();
    fireEvent.click(within(menu).getByRole("menuitem", { name: /移动到这里/ }));

    await waitFor(() => {
      expect(within(screen.getByLabelText("game hud")).getByText("广场")).toBeInTheDocument();
    });
  });

  it("toggles the pause menu with Escape and saves from the menu", async () => {
    await enterGame();

    fireEvent.keyDown(window, { key: "Escape" });
    const pauseMenu = await screen.findByLabelText("pause menu");
    expect(within(pauseMenu).getByRole("button", { name: /继续游戏/ })).toBeInTheDocument();
    expect(pauseMenu).toHaveClass("pause-overlay");

    fireEvent.click(within(screen.getByLabelText("save load panel")).getByRole("button", {
      name: /保存/,
    }));

    await waitFor(() => {
      expect(screen.getByText(/browser-memory:\/\/slot_1.json/)).toBeInTheDocument();
    });

    fireEvent.click(within(pauseMenu).getByRole("button", { name: /继续游戏/ }));
    expect(screen.queryByLabelText("pause menu")).not.toBeInTheDocument();
  });

  it("saves and loads the selected slot independently from the pause menu", async () => {
    await enterGame();

    fireEvent.click(screen.getByRole("button", { name: "广场" }));
    fireEvent.click(
      within(screen.getByLabelText("location details")).getByRole("button", {
        name: /移动/,
      }),
    );
    await waitFor(() => {
      expect(within(screen.getByLabelText("game hud")).getByText("广场")).toBeInTheDocument();
    });

    fireEvent.keyDown(window, { key: "Escape" });
    const saveLoadPanel = await screen.findByLabelText("save load panel");
    fireEvent.click(within(saveLoadPanel).getByRole("button", { name: "槽位 2" }));
    expect(within(saveLoadPanel).getByText("当前槽位")).toBeInTheDocument();
    expect(within(saveLoadPanel).getAllByText("slot_2").length).toBeGreaterThan(0);
    fireEvent.click(within(saveLoadPanel).getByRole("button", { name: /保存/ }));
    await waitFor(() => {
      expect(screen.getByText(/browser-memory:\/\/slot_2.json/)).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: /继续游戏/ }));
    fireEvent.click(screen.getByRole("button", { name: "人里的门" }));
    fireEvent.click(
      within(screen.getByLabelText("location details")).getByRole("button", {
        name: /移动/,
      }),
    );
    await waitFor(() => {
      expect(within(screen.getByLabelText("game hud")).getByText("人里的门")).toBeInTheDocument();
    });

    fireEvent.keyDown(window, { key: "Escape" });
    const loadPanel = await screen.findByLabelText("save load panel");
    fireEvent.click(within(loadPanel).getByRole("button", { name: "预检读取" }));
    await waitFor(() => {
      expect(screen.getByText(/读档预检：可读取/)).toBeInTheDocument();
      expect(within(loadPanel).getByRole("button", { name: "确认读取" })).toBeInTheDocument();
    });

    fireEvent.click(within(loadPanel).getByRole("button", { name: "确认读取" }));
    await waitFor(() => {
      expect(within(screen.getByLabelText("game hud")).getByText("广场")).toBeInTheDocument();
    });
  });

  it("shows current-location characters and a stable portrait fallback", async () => {
    await enterGame();

    fireEvent.click(screen.getByRole("button", { name: "打开人物面板" }));
    const dock = screen.getByLabelText("current location characters");
    expect(within(dock).getAllByText("示例角色").length).toBeGreaterThan(0);
    expect(within(dock).getByLabelText("character portrait")).toBeInTheDocument();
    expect(within(dock).getByText("core.demo.heroine.neutral")).toBeInTheDocument();
  });

  it("opens dialogue as a layer and dispatches choices", async () => {
    await enterGame();

    fireEvent.click(
      within(screen.getByLabelText("quick actions")).getByRole("button", { name: /对话/ }),
    );

    const layer = await screen.findByLabelText("dialogue layer");
    expect(within(layer).getByText("询问新引擎")).toBeInTheDocument();
    fireEvent.click(within(layer).getByRole("button", { name: "询问新引擎" }));

    await waitFor(() => {
      expect(within(layer).getByText(/不执行旧 ERB/)).toBeInTheDocument();
    });
  });

  it("shows dialogue choices only when relationship conditions pass", async () => {
    await enterGame();

    fireEvent.click(
      within(screen.getByLabelText("quick actions")).getByRole("button", { name: /对话/ }),
    );
    const layer = await screen.findByLabelText("dialogue layer");
    expect(within(layer).queryByText("谈谈信任")).not.toBeInTheDocument();

    const actionBar = screen.getByLabelText("quick actions");
    fireEvent.click(within(actionBar).getByRole("button", { name: /交流/ }));
    await waitFor(() => {
      expect(screen.getAllByText("6").length).toBeGreaterThan(0);
    });
    fireEvent.click(within(actionBar).getByRole("button", { name: /交流/ }));

    await waitFor(() => {
      expect(within(layer).getByText("谈谈信任")).toBeInTheDocument();
    });
  });

  it("uses the title Mod entry for preflight, install, enablement, and uninstall", async () => {
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /Mod/ }));
    const modPanel = screen.getByLabelText("mod panel");
    fireEvent.click(within(modPanel).getByRole("button", { name: /Mod 预检/ }));

    await waitFor(() => {
      expect(screen.getByLabelText("mod package preflight")).toBeInTheDocument();
      expect(screen.getByText(/可安装：example\.minimal_character/)).toBeInTheDocument();
      expect(screen.getByText(/resource sha256 is missing/)).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "安装 Mod 包" }));
    const toggle = await screen.findByRole("checkbox", {
      name: "启用 example.minimal_character",
    });
    expect(toggle).toBeChecked();
    fireEvent.click(toggle);

    await waitFor(() => {
      expect(toggle).not.toBeChecked();
      expect(within(screen.getByLabelText("mod enablement plan")).getByText(/无启用 Mod/))
        .toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "卸载 example.minimal_character" }));
    const uninstallPlan = await screen.findByLabelText("mod uninstall plan");
    fireEvent.click(within(uninstallPlan).getByRole("button", { name: /确认卸载/ }));

    await waitFor(() => {
      expect(screen.getByLabelText("mod uninstall result")).toBeInTheDocument();
      expect(within(screen.getByLabelText("installed mods")).getByText("未发现已安装 Mod。"))
        .toBeInTheDocument();
    });
  });

  it("opens title load as a dedicated screen", async () => {
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "读取" }));

    expect(screen.getByLabelText("load screen")).toBeInTheDocument();
    expect(screen.queryByLabelText("title screen")).not.toBeInTheDocument();
    expect(screen.getByLabelText("save load panel")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "返回标题菜单" })).toBeInTheDocument();
  });

  it("restores persisted mod enablement for installed mods", async () => {
    await useEngine.getState().client.saveModEnablement(DEFAULT_MOD_INSTALL_ROOT, [
      { namespace: "example.minimal_character", enabled: false },
    ]);
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /Mod/ }));
    fireEvent.click(within(screen.getByLabelText("mod panel")).getByRole("button", {
      name: /Mod 预检/,
    }));
    fireEvent.click(await screen.findByRole("button", { name: "安装 Mod 包" }));

    const toggle = await screen.findByRole("checkbox", {
      name: "启用 example.minimal_character",
    });
    expect(toggle).not.toBeChecked();
    expect(within(screen.getByLabelText("mod enablement plan")).getByText(/禁用/))
      .toBeInTheDocument();
  });
});
