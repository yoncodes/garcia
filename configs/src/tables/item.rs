use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct ItemDefinition {
    pub id: i32,
    #[serde(default)]
    pub english_name: String,
    pub quality: i32,
    #[serde(rename = "type", default)]
    pub item_type: i32,
    #[serde(default)]
    pub sub_type: i32,
    #[serde(default)]
    pub effect: Option<TablePair<i32>>,
}
