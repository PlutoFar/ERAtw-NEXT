import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { mockContentPackageIndex } from "../engine/mockData";
import { ContentPackageScreen } from "./ContentPackageScreen";

describe("ContentPackageScreen", () => {
  it("loads an absolute package path", async () => {
    const user = userEvent.setup();
    const onLoad = vi.fn().mockResolvedValue(undefined);
    render(
      <ContentPackageScreen
        content={null}
        onLoad={onLoad}
        onChooseDirectory={async () => null}
      />,
    );

    await user.type(screen.getByLabelText("内容包目录"), "D:\\content\\package");
    await user.click(screen.getByRole("button", { name: "加载内容包" }));

    expect(onLoad).toHaveBeenCalledWith("D:\\content\\package");
  });

  it("renders package counts and location index", async () => {
    const user = userEvent.setup();
    render(
      <ContentPackageScreen
        content={mockContentPackageIndex}
        onLoad={vi.fn().mockResolvedValue(undefined)}
        onChooseDirectory={async () => null}
      />,
    );

    expect(screen.getByText("Playable Demo")).toBeInTheDocument();
    expect(screen.getByText("角色 1")).toBeInTheDocument();
    expect(screen.getByText("对话场景 0")).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "地点" }));
    expect(screen.getByText("居所")).toBeInTheDocument();
    expect(screen.getByText("广场")).toBeInTheDocument();
  });
});
