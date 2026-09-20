use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct CollectionSuit {
    pub id: i32,
    #[serde(default)]
    pub step: Vec<TablePair<i32>>,
    #[serde(default)]
    pub step_reward: Vec<TablePair<i32>>,
}
