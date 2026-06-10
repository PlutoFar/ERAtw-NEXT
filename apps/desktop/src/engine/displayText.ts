const phraseReplacements: Array<[string, string]> = [
  ["人里的門", "人里的门"],
  ["東大街", "东大街"],
  ["鈴奈庵", "铃奈庵"],
  ["小鈴私室", "小铃私室"],
  ["長屋前", "长屋前"],
  ["鯢呑亭", "鲵吞亭"],
  ["鯢呑", "鲵吞"],
  ["銭湯", "钱汤"],
  ["広场", "广场"],
];

const characterReplacements: Record<string, string> = {
  東: "东",
  鈴: "铃",
  長: "长",
  門: "门",
  広: "广",
  鯢: "鲵",
  呑: "吞",
  銭: "钱",
  湯: "汤",
  櫓: "橹",
};

export const normalizeSimplifiedChineseText = (text: string): string => {
  let normalized = text;
  for (const [source, replacement] of phraseReplacements) {
    normalized = normalized.replaceAll(source, replacement);
  }
  return normalized.replace(
    /[東鈴長門広鯢呑銭湯櫓]/g,
    (character) => characterReplacements[character] ?? character,
  );
};

export const displayText = (text: string | null | undefined, fallback = "") =>
  normalizeSimplifiedChineseText(text ?? fallback);
