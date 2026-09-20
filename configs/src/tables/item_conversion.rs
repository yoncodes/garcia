use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct ItemConversion {
    pub id: i32,
    pub target: i32,
    pub cost_type_number: i32,
    pub cost_item_number: i32,
    #[serde(default)]
    pub cost_item: Vec<i32>,
    #[serde(rename = "limit_def", default)]
    pub conditions: Vec<TablePair<String>>,
}
