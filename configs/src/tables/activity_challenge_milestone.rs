use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct ActivityChallengeMilestone {
    pub id: i32,
    pub activity_id: i32,
    pub challenge_point: i32,
    pub reward: TablePair<i32>,
}
