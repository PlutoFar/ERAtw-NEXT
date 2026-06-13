import { ThemeProvider } from "@mui/material/styles";
import { fireEvent, render, screen, within } from "@testing-library/react";
import type { ReactElement } from "react";
import { describe, expect, it } from "vitest";
import { mockMapModel } from "../engine/mockData";
import { SettingsProvider } from "../settings/SettingsContext";
import { theme } from "../theme";
import { MapView } from "./MapView";

function renderMap(ui: ReactElement, mode: "ascii" | "svg" = "svg") {
  return render(
    <ThemeProvider theme={theme}>
      <SettingsProvider initialMapRenderMode={mode}>{ui}</SettingsProvider>
    </ThemeProvider>,
  );
}

describe("MapView 双模式地图", () => {
  it("SVG 模式默认渲染并显示图例", () => {
    renderMap(<MapView model={mockMapModel} />, "svg");
    expect(screen.getByRole("img", { name: "地图节点与连线" })).toBeInTheDocument();
    expect(screen.getByText("逗留中")).toBeInTheDocument();
  });

  it("字符画模式渲染网格", () => {
    renderMap(<MapView model={mockMapModel} />, "ascii");
    expect(screen.getByRole("grid", { name: "字符画地图" })).toBeInTheDocument();
  });

  it("切换显示方式时保留选中地点", () => {
    renderMap(<MapView model={mockMapModel} />, "svg");

    // 选中“茶馆”
    fireEvent.click(screen.getAllByText("茶馆")[0]);
    const sidebar = screen.getByLabelText("地图侧栏");
    expect(within(sidebar).getByRole("heading", { name: /茶馆/ })).toBeInTheDocument();

    // 切到字符画
    fireEvent.click(screen.getByRole("button", { name: "字符画地图" }));
    expect(screen.getByRole("grid", { name: "字符画地图" })).toBeInTheDocument();

    // 选中状态保留
    expect(within(sidebar).getByRole("heading", { name: /茶馆/ })).toBeInTheDocument();
  });

  it("切换区域 Tab 切到命莲寺周边", () => {
    renderMap(<MapView model={mockMapModel} />, "svg");
    fireEvent.click(screen.getByRole("tab", { name: "命莲寺周边" }));
    expect(screen.getAllByText("本堂").length).toBeGreaterThan(0);
  });
});
