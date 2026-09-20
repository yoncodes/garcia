use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct BattlePassTask {
    pub id: i32,
    #[serde(rename = "type")]
    pub task_type: i32,
    pub chain: i32,
    pub chain_order: i32,
    pub finish_limit: Vec<TablePair<String>>,
    pub exp: TablePair<i32>,
    pub bp_id: i32,
}
