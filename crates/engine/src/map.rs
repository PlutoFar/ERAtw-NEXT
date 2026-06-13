//! 地图模型：双模式地图（字符画 / SVG）共享的同一份数据。
//!
//! M0 使用引擎内置的自有示例数据（幻想乡题材、自有布局），
//! 不复制 `eratw-content` 的地图。后续由内容包提供真实数据。

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: &str = "map-model/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapModel {
    pub schema_version: String,
    pub default_area_id: String,
    pub grid: Grid,
    pub areas: Vec<Area>,
    pub legend: Vec<LegendEntry>,
    pub nodes: Vec<MapNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Grid {
    pub columns: u32,
    pub rows: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Area {
    pub id: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegendEntry {
    pub key: String,
    pub label: String,
    pub glyph: String,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapNode {
    pub id: String,
    pub area_id: String,
    pub label: String,
    pub kind: String,
    pub glyph: String,
    pub x: u32,
    pub y: u32,
    pub terrain: String,
    pub move_minutes: u32,
    pub note: String,
    pub links: Vec<String>,
    pub occupants: Vec<Occupant>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Occupant {
    pub id: String,
    pub label: String,
    pub activity: String,
}

fn s(value: &str) -> String {
    value.to_string()
}

fn occ(id: &str, label: &str, activity: &str) -> Occupant {
    Occupant {
        id: s(id),
        label: s(label),
        activity: s(activity),
    }
}

#[allow(clippy::too_many_arguments)]
fn node(
    id: &str,
    area_id: &str,
    label: &str,
    kind: &str,
    glyph: &str,
    x: u32,
    y: u32,
    terrain: &str,
    move_minutes: u32,
    note: &str,
    links: &[&str],
    occupants: Vec<Occupant>,
) -> MapNode {
    MapNode {
        id: s(id),
        area_id: s(area_id),
        label: s(label),
        kind: s(kind),
        glyph: s(glyph),
        x,
        y,
        terrain: s(terrain),
        move_minutes,
        note: s(note),
        links: links.iter().map(|l| s(l)).collect(),
        occupants,
    }
}

/// 返回内置示例地图模型。
pub fn map_overview() -> MapModel {
    MapModel {
        schema_version: s(SCHEMA_VERSION),
        default_area_id: s("village"),
        grid: Grid {
            columns: 48,
            rows: 24,
        },
        areas: vec![
            Area {
                id: s("village"),
                label: s("人里"),
                description: s("以中央广场为核心的人类聚落，四向街道连接各处店铺与公共设施。"),
            },
            Area {
                id: s("temple"),
                label: s("命莲寺周边"),
                description: s("山门内的寺院区域，含本堂、墓地、灵泉与塔。"),
            },
        ],
        legend: vec![
            LegendEntry {
                key: s("staying"),
                label: s("逗留中"),
                glyph: s("△"),
                color: s("#d65a5a"),
            },
            LegendEntry {
                key: s("working"),
                label: s("工作中"),
                glyph: s("●"),
                color: s("#e08a3c"),
            },
            LegendEntry {
                key: s("sleeping"),
                label: s("睡眠中"),
                glyph: s("z"),
                color: s("#4f9bd9"),
            },
            LegendEntry {
                key: s("passing"),
                label: s("路人"),
                glyph: s("·"),
                color: s("#8b8f96"),
            },
            LegendEntry {
                key: s("free"),
                label: s("自由行动"),
                glyph: s("☆"),
                color: s("#e0c341"),
            },
        ],
        nodes: vec![
            // ===== 人里 =====
            node(
                "plaza",
                "village",
                "中央广场",
                "public",
                "◎",
                22,
                12,
                "石砖广场",
                0,
                "四条主街在此交汇，是人里的中心。",
                &[
                    "gate_south",
                    "market",
                    "teahouse",
                    "clinic",
                    "inn",
                    "well",
                    "shrine_path",
                    "bookstore",
                ],
                vec![
                    occ("villager_a", "里人甲", "passing"),
                    occ("villager_b", "里人乙", "free"),
                ],
            ),
            node(
                "gate_south",
                "village",
                "南门",
                "gate",
                "門",
                22,
                22,
                "木制大门",
                8,
                "通往村外桥与南方道路的关口。",
                &["plaza"],
                vec![occ("guard", "门卫", "working")],
            ),
            node(
                "market",
                "village",
                "集市",
                "shop",
                "市",
                9,
                9,
                "露天市集",
                6,
                "广场西北的露天市集，清晨最热闹。",
                &["plaza", "blacksmith"],
                vec![
                    occ("merchant", "杂货商", "working"),
                    occ("kid", "顽童", "passing"),
                ],
            ),
            node(
                "blacksmith",
                "village",
                "锻冶屋",
                "shop",
                "鍛",
                9,
                3,
                "石砌作坊",
                9,
                "集市北侧的锻冶作坊，常年炉火不熄。",
                &["market"],
                vec![occ("smith", "锻冶师", "working")],
            ),
            node(
                "teahouse",
                "village",
                "茶馆",
                "shop",
                "茶",
                34,
                9,
                "二层木楼",
                6,
                "广场东北的茶馆，午后是闲谈之所。",
                &["plaza", "school", "bathhouse"],
                vec![
                    occ("hostess", "看板娘", "working"),
                    occ("regular", "常客", "staying"),
                ],
            ),
            node(
                "school",
                "village",
                "寺子屋",
                "public",
                "学",
                34,
                3,
                "讲堂院落",
                9,
                "孩子们读书识字的地方。",
                &["teahouse"],
                vec![occ("teacher", "先生", "working")],
            ),
            node(
                "clinic",
                "village",
                "诊所",
                "public",
                "医",
                37,
                15,
                "白墙医馆",
                8,
                "村东的诊所，兼营药材。",
                &["plaza", "bathhouse", "bookstore"],
                vec![occ("doctor", "医师", "working")],
            ),
            node(
                "bathhouse",
                "village",
                "钱汤",
                "public",
                "汤",
                41,
                11,
                "蒸汽浴堂",
                10,
                "村东最东的钱汤，傍晚人多。",
                &["teahouse", "clinic"],
                vec![occ("bather", "泡汤客", "staying")],
            ),
            node(
                "inn",
                "village",
                "旅笼屋",
                "home",
                "宿",
                7,
                16,
                "客栈",
                7,
                "广场西侧的客栈，接待外来旅人。",
                &["plaza"],
                vec![occ("traveler", "旅人", "sleeping")],
            ),
            node(
                "well",
                "village",
                "古井",
                "landmark",
                "井",
                16,
                15,
                "石井",
                4,
                "广场西南的古井，村人取水处。",
                &["plaza"],
                vec![],
            ),
            node(
                "shrine_path",
                "village",
                "参道入口",
                "landmark",
                "鳥",
                22,
                6,
                "石板参道",
                5,
                "广场正北的参道入口，通向命莲寺方向。",
                &["plaza"],
                vec![occ("pilgrim", "香客", "passing")],
            ),
            node(
                "bookstore",
                "village",
                "书肆",
                "shop",
                "书",
                28,
                18,
                "旧书店",
                7,
                "广场东南的旧书店，藏书繁杂。",
                &["plaza", "clinic"],
                vec![occ("clerk", "店主", "working")],
            ),
            // ===== 命莲寺周边 =====
            node(
                "temple_gate",
                "temple",
                "山门",
                "gate",
                "門",
                24,
                20,
                "山门",
                12,
                "命莲寺的山门，进入寺院区域的入口。",
                &["main_hall"],
                vec![occ("monk_a", "扫地僧", "working")],
            ),
            node(
                "main_hall",
                "temple",
                "本堂",
                "shrine",
                "堂",
                24,
                8,
                "大殿",
                4,
                "寺院本堂，香火所在。",
                &["temple_gate", "graveyard", "pagoda", "hermitage"],
                vec![occ("monk_b", "住持", "staying")],
            ),
            node(
                "graveyard",
                "temple",
                "墓地",
                "nature",
                "墓",
                9,
                12,
                "墓园",
                4,
                "本堂西侧的墓地，松柏环绕。",
                &["main_hall", "spring"],
                vec![],
            ),
            node(
                "pagoda",
                "temple",
                "五重塔",
                "landmark",
                "塔",
                39,
                10,
                "高塔",
                6,
                "本堂东侧的五重塔，可远眺。",
                &["main_hall"],
                vec![occ("sweeper", "塔守", "free")],
            ),
            node(
                "hermitage",
                "temple",
                "庵",
                "home",
                "庵",
                24,
                3,
                "草庵",
                7,
                "本堂北侧的小庵，僧人起居处。",
                &["main_hall"],
                vec![occ("hermit", "隐者", "sleeping")],
            ),
            node(
                "spring",
                "temple",
                "灵泉",
                "nature",
                "泉",
                11,
                4,
                "清泉",
                16,
                "墓地西北的清泉，传说能净心。",
                &["graveyard"],
                vec![],
            ),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn model_has_stable_identity() {
        let model = map_overview();
        assert_eq!(model.schema_version, SCHEMA_VERSION);
        assert!(model.areas.iter().any(|a| a.id == model.default_area_id));
    }

    #[test]
    fn nodes_reference_valid_areas() {
        let model = map_overview();
        let areas: HashSet<&str> = model.areas.iter().map(|a| a.id.as_str()).collect();
        for node in &model.nodes {
            assert!(
                areas.contains(node.area_id.as_str()),
                "bad area: {}",
                node.id
            );
        }
    }

    #[test]
    fn links_resolve_to_existing_nodes() {
        let model = map_overview();
        let ids: HashSet<&str> = model.nodes.iter().map(|n| n.id.as_str()).collect();
        for node in &model.nodes {
            for link in &node.links {
                assert!(
                    ids.contains(link.as_str()),
                    "{} -> missing {}",
                    node.id,
                    link
                );
            }
        }
    }

    #[test]
    fn occupant_activities_match_legend() {
        let model = map_overview();
        let keys: HashSet<&str> = model.legend.iter().map(|l| l.key.as_str()).collect();
        for node in &model.nodes {
            for occ in &node.occupants {
                assert!(
                    keys.contains(occ.activity.as_str()),
                    "bad activity: {}",
                    occ.activity
                );
            }
        }
    }

    #[test]
    fn coordinates_are_within_grid() {
        let model = map_overview();
        for node in &model.nodes {
            assert!(node.x < model.grid.columns, "{} x out of range", node.id);
            assert!(node.y < model.grid.rows, "{} y out of range", node.id);
        }
    }
}
