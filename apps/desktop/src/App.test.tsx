import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { App } from "./App";
import {
  buildAsciiMapModel,
  groupLocationLegendLocations,
} from "./components/traditional/viewModel";
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

const openMovementMap = async () => {
  fireEvent.click(
    within(screen.getByLabelText("quick actions")).getByRole("button", {
      name: /移动/,
    }),
  );
  return screen.findByLabelText("movement map");
};

const openWorldMap = async () => {
  fireEvent.click(
    within(screen.getByLabelText("quick actions")).getByRole("button", {
      name: /地图/,
    }),
  );
  return screen.findByLabelText("world map");
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
    expect(screen.getByLabelText("status screen")).toBeInTheDocument();
    expect(screen.getByLabelText("current scene")).toBeInTheDocument();
    expect(screen.getByLabelText("focused character")).toBeInTheDocument();
    expect(screen.getByLabelText("location occupants")).toBeInTheDocument();
    expect(
      within(screen.getByLabelText("quick actions")).getByRole("button", { name: /地图/ }),
    ).toBeInTheDocument();
    expect(screen.queryByText(/Act_COM/)).not.toBeInTheDocument();
    expect(screen.queryByLabelText("era text map")).not.toBeInTheDocument();
  });

  it("renders ASCII map text as fixed cells with separate overlay hotspots", async () => {
    await enterGame();
    await openMovementMap();

    const map = screen.getByLabelText("era text map");
    expect(map).toHaveClass("ascii-map-grid");
    expect(within(map).queryByRole("button")).not.toBeInTheDocument();
    expect(map.textContent).toContain("广场");
    expect(map.textContent).toContain("稗田邸");
    expect(map.textContent).toContain("阿求私室");
    expect(map.textContent).toContain("慧音房间");
    expect(map.textContent).toContain("咖啡馆");
    expect(map.textContent).toContain("包场浴场");
    expect(map.textContent).toContain("望楼");
    expect(map.textContent).not.toContain("広场");
    expect(map.textContent).not.toContain("櫓");
    expect(map.textContent).not.toContain("橹");
    expect(map.textContent).not.toContain("咖啡館");
    expect(map.textContent).not.toContain("貸切浴場");
    expect(map.textContent).not.toContain("房間");
    expect(map.textContent).not.toContain("现在位置");
    expect(map.textContent).not.toContain("颜色说明");
    expect(map.textContent).not.toContain("提示");
    expect(map.querySelectorAll(".ascii-map-cell").length).toBeGreaterThan(0);
    expect(map.querySelectorAll(".cell-wall").length).toBeGreaterThan(0);
    expect(map.querySelectorAll(".cell-road").length).toBeGreaterThan(0);
    expect(map.querySelectorAll(".cell-building-label").length).toBeGreaterThan(0);
    expect(map.querySelectorAll(".cell-forest").length).toBeGreaterThan(0);
    expect(Number(map.getAttribute("data-column-count"))).toBeGreaterThan(110);
    expect(Number(map.getAttribute("data-row-count"))).toBeGreaterThan(50);

    const hotspots = screen.getByLabelText("text map hotspots");
    expect(within(hotspots).getByRole("button", { name: "人里的门" })).toHaveAttribute(
      "data-location-id",
      "school_gate",
    );
    expect(within(hotspots).getByRole("button", { name: "南大街" })).toHaveAttribute(
      "data-location-id",
      "club_room",
    );
    expect(within(hotspots).getByRole("button", { name: "瞭望楼" })).toHaveAttribute(
      "data-location-id",
      "legacy.sato.220",
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
    expect(model.hotspots.map((hotspot) => hotspot.locationId)).toContain("club_room");
    expect(model.hotspots.length).toBeGreaterThan(20);
    expect(model.rowCount).toBeGreaterThan(50);
    expect(model.maxColumns).toBeGreaterThan(110);
    expect(model.lines.join("\n")).not.toContain("现在位置");
    expect(model.lines.join("\n")).not.toContain("颜色说明");
  });

  it("opens the map independently and supports wheel zoom", async () => {
    await enterGame();
    await openWorldMap();

    expect(screen.getByLabelText("map screen")).toBeInTheDocument();
    const viewport = document.querySelector(".ascii-map-viewport");
    expect(viewport).toHaveAttribute("data-zoom", "1.00");

    fireEvent.wheel(viewport!, { deltaY: -120 });

    await waitFor(() => {
      expect(document.querySelector(".ascii-map-viewport")).toHaveAttribute(
        "data-zoom",
        "1.08",
      );
    });
  });

  it("groups the location legend by map area instead of rendering one flat list", async () => {
    await enterGame();
    await openMovementMap();

    const legend = screen.getByLabelText("location legend");
    expect(within(legend).getByText("人里")).toBeInTheDocument();
    expect(within(legend).getByText("长屋")).toBeInTheDocument();
    expect(within(legend).getAllByText("鲵吞亭").length).toBeGreaterThan(0);
    expect(screen.queryByText("切区")).not.toBeInTheDocument();

    const world = createDemoWorld();
    const groups = groupLocationLegendLocations(world.locations);
    expect(groups.map((group) => group.title)).toEqual(
      expect.arrayContaining(["街区 / 出入口", "商店 / 设施", "长屋 / 住居"]),
    );
  });

  it("opens location details from a hotspot and moves by double click", async () => {
    await enterGame();
    await openMovementMap();

    const plazaHotspot = screen.getByRole("button", { name: "广场" });
    fireEvent.mouseEnter(plazaHotspot);
    expect(within(screen.getByRole("tooltip")).getByText("广场")).toBeInTheDocument();

    fireEvent.click(plazaHotspot);
    const destination = screen.getByLabelText("selected destination");
    expect(within(destination).getByRole("heading", { name: "广场" })).toBeInTheDocument();

    fireEvent.doubleClick(plazaHotspot);
    await waitFor(() => {
      expect(within(screen.getByLabelText("game hud")).getByText("广场")).toBeInTheDocument();
    });
    expect(screen.queryByLabelText("movement map")).not.toBeInTheDocument();
  });

  it("opens a right-click location menu and moves through it", async () => {
    await enterGame();
    await openMovementMap();

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

    await openMovementMap();
    fireEvent.click(screen.getByRole("button", { name: "广场" }));
    fireEvent.click(
      within(screen.getByLabelText("selected destination")).getByRole("button", {
        name: /确定移动/,
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
    await openMovementMap();
    fireEvent.click(screen.getByRole("button", { name: "人里的门" }));
    fireEvent.click(
      within(screen.getByLabelText("selected destination")).getByRole("button", {
        name: /确定移动/,
      }),
    );
    await waitFor(() => {
      expect(within(screen.getByLabelText("game hud")).getByText("人里的门")).toBeInTheDocument();
    });

    fireEvent.keyDown(window, { key: "Escape" });
    const pauseMenu = await screen.findByLabelText("pause menu");
    fireEvent.click(within(pauseMenu).getByRole("button", { name: "读取" }));
    const loadPanel = screen.getByLabelText("save load panel");
    expect(within(loadPanel).queryByRole("button", { name: /保存/ })).not.toBeInTheDocument();
    fireEvent.click(within(loadPanel).getByRole("button", { name: "预检读取" }));
    await waitFor(() => {
      expect(screen.getByText(/读档预检：可读取/)).toBeInTheDocument();
      expect(within(loadPanel).getByRole("button", { name: "确认读取" })).toBeInTheDocument();
    });

    fireEvent.click(within(loadPanel).getByRole("button", { name: "确认读取" }));
    await waitFor(() => {
      expect(within(screen.getByLabelText("game hud")).getByText("广场")).toBeInTheDocument();
    });
  }, 10_000);

  it("opens title load as a standalone load screen", async () => {
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "读取" }));
    const loadScreen = await screen.findByLabelText("load screen");
    const loadPanel = within(loadScreen).getByLabelText("save load panel");

    expect(within(loadPanel).getByRole("heading", { name: "读取存档" })).toBeInTheDocument();
    expect(within(loadPanel).getByRole("button", { name: "预检读取" })).toBeInTheDocument();
    expect(within(loadPanel).queryByRole("button", { name: /保存/ })).not.toBeInTheDocument();
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
      expect(screen.getByText(/好感度:S 6/)).toBeInTheDocument();
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
