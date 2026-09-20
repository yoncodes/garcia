use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct DailyActivityMilestone {
    pub id: i32,
    pub active: i32,
    pub active_reward: Vec<TablePair<i32>>,
    pub player_exp: TablePair<i32>,
}
