use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct ActivityLevelUp {
    pub id: i32,
    #[serde(rename = "lv")]
    pub required_level: i32,
    pub reward: Vec<TablePair<i32>>,
}
