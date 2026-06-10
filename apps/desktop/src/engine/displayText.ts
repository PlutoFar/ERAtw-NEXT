const phraseReplacements: Array<[string, string]> = [
  ["人里的門", "人里的门"],
  ["東大街", "东大街"],
  ["鈴奈庵", "铃奈庵"],
  ["小鈴私室", "小铃私室"],
  ["長屋前", "长屋前"],
  ["瞭望樓", "瞭望楼"],
  ["鯢呑亭", "鲵吞亭"],
  ["鯢呑", "鲵吞"],
  ["銭湯", "钱汤"],
  ["貸切浴場", "包场浴场"],
  ["甘味処", "甘味处"],
  ["櫓", "瞭望楼"],
  ["橹", "瞭望楼"],
  ["広场", "广场"],
  ["房間", "房间"],
];

const characterReplacements: Record<string, string> = {
  館: "馆",
  東: "东",
  鈴: "铃",
  長: "长",
  門: "门",
  龍: "龙",
  広: "广",
  鯢: "鲵",
  呑: "吞",
  銭: "钱",
  湯: "汤",
  樓: "楼",
  橋: "桥",
  間: "间",
  蓮: "莲",
  貸: "贷",
  場: "场",
  処: "处",
};

export const normalizeSimplifiedChineseText = (text: string): string => {
  let normalized = text;
  for (const [source, replacement] of phraseReplacements) {
    normalized = normalized.replaceAll(source, replacement);
  }
  return normalized.replace(
    /[館東鈴長門龍広鯢呑銭湯樓橋間蓮貸場処]/g,
    (character) => characterReplacements[character] ?? character,
  );
};

export const displayText = (text: string | null | undefined, fallback = "") =>
  normalizeSimplifiedChineseText(text ?? fallback);
