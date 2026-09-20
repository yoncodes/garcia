use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct TeamCoreRecipe {
    pub id: i32,
    pub target: Vec<TablePair<i32>>,
    pub cost_item: Vec<TablePair<i32>>,
}
