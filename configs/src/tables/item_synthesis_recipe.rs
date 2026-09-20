use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct ItemSynthesisRecipe {
    pub id: i32,
    pub target: i32,
    #[serde(rename = "limit_def", default)]
    pub conditions: Vec<TablePair<String>>,
    #[serde(default)]
    pub cost_item: Vec<TablePair<i32>>,
}
