use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct PartnerAscension {
    pub group_id: i32,
    #[serde(rename = "break_lv")]
    pub rank: i32,
    #[serde(default)]
    pub cost: Vec<TablePair<i32>>,
    #[serde(rename = "world_lv")]
    pub required_world_level: i32,
}
