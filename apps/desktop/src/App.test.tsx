import { ThemeProvider } from "@mui/material/styles";
import { render, screen, waitFor } from "@testing-library/react";
import type { ReactElement } from "react";
import { describe, expect, it } from "vitest";
import App from "./App";
import type { EngineClient } from "./engine/client";
import { mockMapModel, mockSystemStatus } from "./engine/mockData";
import { SettingsProvider } from "./settings/SettingsContext";
import { theme } from "./theme";

function renderApp(ui: ReactElement) {
  return render(
    <ThemeProvider theme={theme}>
      <SettingsProvider>{ui}</SettingsProvider>
    </ThemeProvider>,
  );
}

const readyClient: EngineClient = {
  getSystemStatus: async () => mockSystemStatus,
  getMapOverview: async () => mockMapModel,
};

describe("App 三态", () => {
  it("初始显示加载状态", () => {
    const pendingClient: EngineClient = {
      getSystemStatus: () => new Promise(() => {}),
      getMapOverview: () => new Promise(() => {}),
    };
    renderApp(<App client={pendingClient} />);
    expect(screen.getByText("正在读取引擎状态…")).toBeInTheDocument();
  });

  it("加载成功后渲染系统状态首屏", async () => {
    renderApp(<App client={readyClient} />);
    expect(await screen.findByText("引擎能力")).toBeInTheDocument();
    expect(screen.getByText("内容边界")).toBeInTheDocument();
  });

  it("加载失败显示错误并可重试", async () => {
    const failClient: EngineClient = {
      getSystemStatus: async () => {
        throw { code: "SYSTEM_STATUS_UNAVAILABLE", message: "引擎挂了", details: {} };
      },
      getMapOverview: async () => mockMapModel,
    };
    renderApp(<App client={failClient} />);
    await waitFor(() =>
      expect(screen.getByText(/引擎状态不可用/)).toBeInTheDocument(),
    );
    expect(screen.getByRole("button", { name: "重试" })).toBeInTheDocument();
  });
});
