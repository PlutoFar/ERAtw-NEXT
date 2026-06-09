import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { App } from "./App";
import { useEngine } from "./engine/useEngine";

describe("App", () => {
  beforeEach(() => {
    useEngine.setState({
      world: null,
      loading: false,
      error: null,
      lastSave: null,
    });
  });

  it("renders the M0 shell and traditional view", async () => {
    render(<App />);

    expect(await screen.findByText("ERAtw-NEXT")).toBeInTheDocument();
    expect(screen.getByLabelText("ASCII map")).toBeInTheDocument();
    expect(screen.getByText("示例角色")).toBeInTheDocument();
  });

  it("loads the modern Pixi map only after selecting modern mode", async () => {
    render(<App />);

    expect(await screen.findByLabelText("ASCII map")).toBeInTheDocument();
    expect(screen.queryByLabelText("modern map canvas")).not.toBeInTheDocument();

    fireEvent.mouseDown(screen.getByRole("tab", { name: "现代" }), {
      button: 0,
      ctrlKey: false,
    });

    expect(await screen.findByLabelText("modern map canvas")).toBeInTheDocument();
  });

  it("dispatches dialogue command through the engine store", async () => {
    render(<App />);

    const dialogueButton = await screen.findByRole("button", { name: /对话/ });
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

    const saveButton = await screen.findByRole("button", { name: "保存" });
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(screen.getByText(/browser-memory:\/\/slot_1.json/)).toBeInTheDocument();
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

  it("dispatches relationship command through the engine store", async () => {
    render(<App />);

    expect(await screen.findByText("好感")).toBeInTheDocument();
    expect(screen.getByText("信赖")).toBeInTheDocument();

    const communicateButton = screen.getByRole("button", { name: /交流/ });
    fireEvent.click(communicateButton);

    await waitFor(() => {
      expect(screen.getByText("6")).toBeInTheDocument();
      expect(screen.getByText("1")).toBeInTheDocument();
    });
  });

  it("shows dialogue choices only when conditions pass", async () => {
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /对话/ }));

    await waitFor(() => {
      expect(screen.getByText("询问新引擎")).toBeInTheDocument();
    });
    expect(screen.queryByText("谈谈信任")).not.toBeInTheDocument();

    const communicateButton = screen.getByRole("button", { name: /交流/ });
    fireEvent.click(communicateButton);
    fireEvent.click(communicateButton);

    await waitFor(() => {
      expect(screen.getByText("谈谈信任")).toBeInTheDocument();
    });
  });
});
