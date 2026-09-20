use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct BossRushRankingReward {
    pub id: i32,
    pub bossrush_id: i32,
    pub upper_limit: TablePair<i32>,
    pub lower_limit: TablePair<i32>,
    pub reward: Vec<TablePair<i32>>,
}
