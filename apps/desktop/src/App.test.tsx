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
});
