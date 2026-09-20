use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct ActivityBossDailyMilestone {
    pub id: i32,
    pub daily_point: i64,
    pub reward: TablePair<i32>,
}
