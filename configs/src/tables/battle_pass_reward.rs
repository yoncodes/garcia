use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct BattlePassReward {
    pub id: i32,
    pub base_reward: TablePair<i32>,
    #[serde(default)]
    pub extra_reward: Vec<TablePair<i32>>,
    #[serde(default)]
    pub is_step_level: i32,
}
