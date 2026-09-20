use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct Archive {
    pub id: i32,
    #[serde(rename = "type")]
    pub archive_type: i32,
    pub detail_id: i32,
    #[serde(rename = "limit_def", default)]
    pub conditions: Vec<TablePair<String>>,
}
