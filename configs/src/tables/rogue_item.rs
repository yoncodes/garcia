use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct RogueItem {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "item_price")]
    pub price: i32,
    #[serde(rename = "item_effect")]
    pub effects: Vec<TablePair<String>>,
    #[serde(default)]
    pub auto_use: i32,
}
