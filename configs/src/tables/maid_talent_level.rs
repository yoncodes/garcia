use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct MaidTalentLevel {
    pub id: String,
    pub quality: i32,
    pub position: i32,
    #[serde(rename = "lv")]
    pub level: i32,
    #[serde(default)]
    pub unlock_limit: Option<TablePair<i32>>,
    #[serde(default)]
    pub cost: Vec<TablePair<i32>>,
    #[serde(default)]
    pub cost_money: i32,
}
