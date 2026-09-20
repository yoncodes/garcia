use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct ActivityBossMilestone {
    pub id: i32,
    pub boss_point: i64,
    pub reward: TablePair<i32>,
}
