use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct Achievement {
    pub id: i32,
    pub achieve_type: i32,
    #[serde(default)]
    pub hide: i32,
    #[serde(default)]
    pub achieve_award: Vec<TablePair<i32>>,
}
