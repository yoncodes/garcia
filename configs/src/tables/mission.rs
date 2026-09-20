use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct Mission {
    pub id: i32,
    #[serde(rename = "type")]
    pub mission_type: i32,
    pub finish_limit: Vec<TablePair<String>>,
    #[serde(rename = "bouns")]
    pub rewards: Vec<TablePair<i32>>,
}
