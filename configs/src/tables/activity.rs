use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct Activity {
    pub id: i32,
    #[serde(rename = "type")]
    pub activity_type: i32,
    pub time_type: i32,
    #[serde(default)]
    pub open_time: String,
    #[serde(default)]
    pub close_time: String,
    #[serde(default)]
    pub limit_type: Vec<TablePair<String>>,
    pub order: i32,
}
