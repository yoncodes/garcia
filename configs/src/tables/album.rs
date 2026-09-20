use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct Album {
    pub id: i32,
    #[serde(rename = "limit_def", default)]
    pub conditions: Vec<TablePair<String>>,
}
