import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import {
  mockContentPackageIndex,
  mockInitialGameState,
} from "../engine/mockData";
import { GameScreen } from "./GameScreen";

const commonProps = {
  content: mockContentPackageIndex,
  onCommand: vi.fn().mockResolvedValue(undefined),
  onSave: vi.fn().mockResolvedValue({
    schemaVersion: "save-report/v1",
    path: "D:/save.json",
    packageId: "demo.playable",
    turn: 0,
    bytes: 100,
    stateHash: "sha256:test",
  }),
  onLoadSave: vi.fn().mockResolvedValue(undefined),
  onChooseSavePath: vi.fn().mockResolvedValue(null),
  onChooseLoadPath: vi.fn().mockResolvedValue(null),
};

describe("GameScreen", () => {
  it("starts a playable package", async () => {
    const user = userEvent.setup();
    const onNewGame = vi.fn().mockResolvedValue(undefined);
    render(
      <GameScreen
        {...commonProps}
        state={null}
        onNewGame={onNewGame}
      />,
    );

    await user.click(screen.getByRole("button", { name: "开始新游戏" }));
    expect(onNewGame).toHaveBeenCalledOnce();
  });

  it("dispatches deterministic wait command", async () => {
    const user = userEvent.setup();
    const onCommand = vi.fn().mockResolvedValue(undefined);
    render(
      <GameScreen
        {...commonProps}
        state={mockInitialGameState}
        onNewGame={vi.fn().mockResolvedValue(undefined)}
        onCommand={onCommand}
      />,
    );

    await user.click(screen.getByRole("button", { name: "等待 30 分钟" }));
    expect(onCommand).toHaveBeenCalledWith({ type: "wait", minutes: 30 });
  });
});
