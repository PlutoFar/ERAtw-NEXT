import { describe, expect, it } from "vitest";
import { normalizeSimplifiedChineseText } from "./displayText";

describe("display text normalization", () => {
  it("normalizes legacy map labels to Simplified Chinese display text", () => {
    expect(
      normalizeSimplifiedChineseText("人里的門 / 東大街 / 鈴奈庵 / 長屋 / 鯢呑亭 / 銭湯 / 広场"),
    ).toBe("人里的门 / 东大街 / 铃奈庵 / 长屋 / 鲵吞亭 / 钱汤 / 广场");
  });

  it("does not require changing ids or unrelated ascii text", () => {
    expect(normalizeSimplifiedChineseText("legacy.sato.201 core.demo")).toBe(
      "legacy.sato.201 core.demo",
    );
  });
});
