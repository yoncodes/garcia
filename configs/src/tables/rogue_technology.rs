use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct RogueTechnology {
    pub id: i32,
    #[serde(rename = "pre_id", default)]
    pub prerequisites: Vec<i32>,
    pub cost: i32,
    #[serde(rename = "effect_id", default)]
    pub effects: Vec<TablePair<String>>,
}
