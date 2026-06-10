import { describe, expect, it } from "vitest";
import { normalizeSimplifiedChineseText } from "./displayText";

describe("display text normalization", () => {
  it("normalizes legacy map labels to Simplified Chinese display text", () => {
    expect(
      normalizeSimplifiedChineseText(
        "人里的門 / 東大街 / 鈴奈庵 / 長屋 / 鯢呑亭 / 銭湯 / 広场 / 咖啡館 / 貸切浴場 / 甘味処 / 八橋的房間 / 蓮子的房間",
      ),
    ).toBe(
      "人里的门 / 东大街 / 铃奈庵 / 长屋 / 鲵吞亭 / 钱汤 / 广场 / 咖啡馆 / 包场浴场 / 甘味处 / 八桥的房间 / 莲子的房间",
    );
  });

  it("does not require changing ids or unrelated ascii text", () => {
    expect(normalizeSimplifiedChineseText("legacy.sato.201 core.demo")).toBe(
      "legacy.sato.201 core.demo",
    );
  });
});
