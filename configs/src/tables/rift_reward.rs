use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct RiftReward {
    pub id: i32,
    pub group: i32,
    pub finish_limit: Vec<TablePair<String>>,
    pub reward: Vec<TablePair<i32>>,
    pub sort: i32,
}
