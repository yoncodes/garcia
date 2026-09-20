use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct RewardBundle {
    pub id: i32,
    #[serde(default)]
    pub reward: Vec<TablePair<i32>>,
}
