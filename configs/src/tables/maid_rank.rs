use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct MaidRank {
    pub maid_id: i32,
    #[serde(rename = "lv")]
    pub rank: i32,
    #[serde(default)]
    pub cost: Vec<TablePair<i32>>,
    #[serde(default)]
    pub award: Vec<TablePair<i32>>,
    #[serde(rename = "world_lv")]
    pub required_world_level: i32,
}
