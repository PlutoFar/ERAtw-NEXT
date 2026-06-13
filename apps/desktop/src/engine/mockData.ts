// 浏览器 / 开发 / 测试环境下的引擎返回镜像数据。
// 与 crates/engine 中的内置数据保持一致；真实环境由 Tauri command 返回。

import type { MapModel, SystemStatus } from "../types";

export const mockSystemStatus: SystemStatus = {
  schemaVersion: "system-status/v1",
  app: {
    name: "ERAtw-NEXT",
    stage: "M0",
    tagline: "ERAtw 现代化引擎与桌面应用，不是旧运行时打包。",
  },
  engine: { name: "eratw_next_engine", version: "0.1.0-m0" },
  build: { profile: "debug", gitDescribe: null, timestamp: null },
  paths: [
    {
      id: "content_source",
      label: "内容源",
      value: "D:\\AICODE\\eratw-content",
      kind: "read_only",
      note: "外部只读源，永不复制进引擎仓库。",
    },
    {
      id: "playable_reference",
      label: "可游玩对照",
      value: "D:\\AICODE\\eratw",
      kind: "reference",
      note: "仅供人工参考，引擎不读取。",
    },
    {
      id: "modern",
      label: "ERAtw-modern",
      value: "D:\\AICODE\\ERAtw-modern",
      kind: "excluded",
      note: "无关项目，不作为输入或迁移来源。",
    },
    {
      id: "native_foundation",
      label: "ERAtw-native-foundation",
      value: "D:\\AICODE\\ERAtw-native-foundation",
      kind: "excluded",
      note: "无关项目，不作为输入或迁移来源。",
    },
  ],
  capabilities: [
    {
      id: "system_status",
      label: "系统状态查询",
      status: "available",
      description: "system_get_status 已可用并被 schema 校验。",
    },
    {
      id: "map_overview",
      label: "地图总览（双模式）",
      status: "available",
      description: "map_get_overview 提供字符画 / SVG 共享的地图模型。",
    },
    {
      id: "content_audit",
      label: "只读内容审计",
      status: "planned",
      description: "M1 实现，扫描 eratw-content，默认不自动执行。",
    },
    {
      id: "erb_runtime",
      label: "ERB 子集解释器",
      status: "disabled",
      description: "默认禁用，不执行任何外部 ERB 或脚本。",
    },
  ],
  currentMilestone: "M0",
  milestones: [
    {
      id: "M0",
      title: "现代工程骨架",
      status: "in_progress",
      summary: "Rust + Tauri + React/MUI 可启动基线，地图功能提前接入。",
    },
    {
      id: "M1",
      title: "只读内容审计",
      status: "planned",
      summary: "安全扫描 eratw-content，输出规模/编码/资源引用报告。",
    },
    {
      id: "M2",
      title: "内容契约与转换草案",
      status: "planned",
      summary: "定义新内容 schema 并生成可校验的草案内容包。",
    },
  ],
};

export const mockMapModel: MapModel = {
  schemaVersion: "map-model/v1",
  defaultAreaId: "village",
  grid: { columns: 48, rows: 24 },
  areas: [
    {
      id: "village",
      label: "人里",
      description: "以中央广场为核心的人类聚落，四向街道连接各处店铺与公共设施。",
    },
    {
      id: "temple",
      label: "命莲寺周边",
      description: "山门内的寺院区域，含本堂、墓地、灵泉与塔。",
    },
  ],
  legend: [
    { key: "staying", label: "逗留中", glyph: "△", color: "#d65a5a" },
    { key: "working", label: "工作中", glyph: "●", color: "#e08a3c" },
    { key: "sleeping", label: "睡眠中", glyph: "z", color: "#4f9bd9" },
    { key: "passing", label: "路人", glyph: "·", color: "#8b8f96" },
    { key: "free", label: "自由行动", glyph: "☆", color: "#e0c341" },
  ],
  nodes: [
    // ===== 人里 =====
    {
      id: "plaza", areaId: "village", label: "中央广场", kind: "public", glyph: "◎",
      x: 22, y: 12, terrain: "石砖广场", moveMinutes: 0,
      note: "四条主街在此交汇，是人里的中心。",
      links: ["gate_south", "market", "teahouse", "clinic", "inn", "well", "shrine_path", "bookstore"],
      occupants: [
        { id: "villager_a", label: "里人甲", activity: "passing" },
        { id: "villager_b", label: "里人乙", activity: "free" },
      ],
    },
    {
      id: "gate_south", areaId: "village", label: "南门", kind: "gate", glyph: "門",
      x: 22, y: 22, terrain: "木制大门", moveMinutes: 8,
      note: "通往村外桥与南方道路的关口。",
      links: ["plaza"],
      occupants: [{ id: "guard", label: "门卫", activity: "working" }],
    },
    {
      id: "market", areaId: "village", label: "集市", kind: "shop", glyph: "市",
      x: 9, y: 9, terrain: "露天市集", moveMinutes: 6,
      note: "广场西北的露天市集，清晨最热闹。",
      links: ["plaza", "blacksmith"],
      occupants: [
        { id: "merchant", label: "杂货商", activity: "working" },
        { id: "kid", label: "顽童", activity: "passing" },
      ],
    },
    {
      id: "blacksmith", areaId: "village", label: "锻冶屋", kind: "shop", glyph: "鍛",
      x: 9, y: 3, terrain: "石砌作坊", moveMinutes: 9,
      note: "集市北侧的锻冶作坊，常年炉火不熄。",
      links: ["market"],
      occupants: [{ id: "smith", label: "锻冶师", activity: "working" }],
    },
    {
      id: "teahouse", areaId: "village", label: "茶馆", kind: "shop", glyph: "茶",
      x: 34, y: 9, terrain: "二层木楼", moveMinutes: 6,
      note: "广场东北的茶馆，午后是闲谈之所。",
      links: ["plaza", "school", "bathhouse"],
      occupants: [
        { id: "hostess", label: "看板娘", activity: "working" },
        { id: "regular", label: "常客", activity: "staying" },
      ],
    },
    {
      id: "school", areaId: "village", label: "寺子屋", kind: "public", glyph: "学",
      x: 34, y: 3, terrain: "讲堂院落", moveMinutes: 9,
      note: "孩子们读书识字的地方。",
      links: ["teahouse"],
      occupants: [{ id: "teacher", label: "先生", activity: "working" }],
    },
    {
      id: "clinic", areaId: "village", label: "诊所", kind: "public", glyph: "医",
      x: 37, y: 15, terrain: "白墙医馆", moveMinutes: 8,
      note: "村东的诊所，兼营药材。",
      links: ["plaza", "bathhouse", "bookstore"],
      occupants: [{ id: "doctor", label: "医师", activity: "working" }],
    },
    {
      id: "bathhouse", areaId: "village", label: "钱汤", kind: "public", glyph: "汤",
      x: 41, y: 11, terrain: "蒸汽浴堂", moveMinutes: 10,
      note: "村东最东的钱汤，傍晚人多。",
      links: ["teahouse", "clinic"],
      occupants: [{ id: "bather", label: "泡汤客", activity: "staying" }],
    },
    {
      id: "inn", areaId: "village", label: "旅笼屋", kind: "home", glyph: "宿",
      x: 7, y: 16, terrain: "客栈", moveMinutes: 7,
      note: "广场西侧的客栈，接待外来旅人。",
      links: ["plaza"],
      occupants: [{ id: "traveler", label: "旅人", activity: "sleeping" }],
    },
    {
      id: "well", areaId: "village", label: "古井", kind: "landmark", glyph: "井",
      x: 16, y: 15, terrain: "石井", moveMinutes: 4,
      note: "广场西南的古井，村人取水处。",
      links: ["plaza"],
      occupants: [],
    },
    {
      id: "shrine_path", areaId: "village", label: "参道入口", kind: "landmark", glyph: "鳥",
      x: 22, y: 6, terrain: "石板参道", moveMinutes: 5,
      note: "广场正北的参道入口，通向命莲寺方向。",
      links: ["plaza"],
      occupants: [{ id: "pilgrim", label: "香客", activity: "passing" }],
    },
    {
      id: "bookstore", areaId: "village", label: "书肆", kind: "shop", glyph: "书",
      x: 28, y: 18, terrain: "旧书店", moveMinutes: 7,
      note: "广场东南的旧书店，藏书繁杂。",
      links: ["plaza", "clinic"],
      occupants: [{ id: "clerk", label: "店主", activity: "working" }],
    },
    // ===== 命莲寺周边 =====
    {
      id: "temple_gate", areaId: "temple", label: "山门", kind: "gate", glyph: "門",
      x: 24, y: 20, terrain: "山门", moveMinutes: 12,
      note: "命莲寺的山门，进入寺院区域的入口。",
      links: ["main_hall"],
      occupants: [{ id: "monk_a", label: "扫地僧", activity: "working" }],
    },
    {
      id: "main_hall", areaId: "temple", label: "本堂", kind: "shrine", glyph: "堂",
      x: 24, y: 8, terrain: "大殿", moveMinutes: 4,
      note: "寺院本堂，香火所在。",
      links: ["temple_gate", "graveyard", "pagoda", "hermitage"],
      occupants: [{ id: "monk_b", label: "住持", activity: "staying" }],
    },
    {
      id: "graveyard", areaId: "temple", label: "墓地", kind: "nature", glyph: "墓",
      x: 9, y: 12, terrain: "墓园", moveMinutes: 4,
      note: "本堂西侧的墓地，松柏环绕。",
      links: ["main_hall", "spring"],
      occupants: [],
    },
    {
      id: "pagoda", areaId: "temple", label: "五重塔", kind: "landmark", glyph: "塔",
      x: 39, y: 10, terrain: "高塔", moveMinutes: 6,
      note: "本堂东侧的五重塔，可远眺。",
      links: ["main_hall"],
      occupants: [{ id: "sweeper", label: "塔守", activity: "free" }],
    },
    {
      id: "hermitage", areaId: "temple", label: "庵", kind: "home", glyph: "庵",
      x: 24, y: 3, terrain: "草庵", moveMinutes: 7,
      note: "本堂北侧的小庵，僧人起居处。",
      links: ["main_hall"],
      occupants: [{ id: "hermit", label: "隐者", activity: "sleeping" }],
    },
    {
      id: "spring", areaId: "temple", label: "灵泉", kind: "nature", glyph: "泉",
      x: 11, y: 4, terrain: "清泉", moveMinutes: 16,
      note: "墓地西北的清泉，传说能净心。",
      links: ["graveyard"],
      occupants: [],
    },
  ],
};
