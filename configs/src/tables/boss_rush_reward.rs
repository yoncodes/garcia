use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct BossRushReward {
    pub id: i32,
    pub bossrush_id: i32,
    pub boss_point: i64,
    pub reward: Vec<TablePair<i32>>,
}
