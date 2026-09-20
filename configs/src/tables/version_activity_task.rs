use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct VersionActivityTask {
    pub id: i32,
    pub group_id: i32,
    #[serde(default)]
    pub finish_limit: Vec<TablePair<String>>,
    #[serde(default)]
    pub reward: Vec<TablePair<i32>>,
}
