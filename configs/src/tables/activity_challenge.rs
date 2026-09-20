use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct ActivityChallenge {
    pub id: i32,
    pub activity_id: i32,
    pub times: i32,
    pub limit_open: Vec<TablePair<String>>,
    pub challenge_id: String,
    pub reward: Vec<TablePair<i32>>,
    pub jump_id: i32,
}
