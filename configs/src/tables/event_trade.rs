use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct EventTrade {
    #[serde(rename = "ID")]
    pub id: i32,
    pub item_regular: Vec<TablePair<i32>>,
}
