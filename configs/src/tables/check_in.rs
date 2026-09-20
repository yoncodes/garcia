use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct CheckIn {
    pub id: i32,
    pub activity_id: i32,
    pub num: i32,
    #[serde(default)]
    pub turn: i32,
    pub reward: TablePair<i32>,
}
