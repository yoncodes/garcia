use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct BattlePass {
    pub id: i32,
    pub start_time: String,
    pub end_time: String,
    pub level_max: i32,
    pub exp_per_level: i32,
    pub exp_week_limit: i32,
    #[serde(default)]
    pub advanced_reward: Vec<TablePair<i32>>,
    #[serde(default)]
    pub premium_reward: Vec<TablePair<i32>>,
}
