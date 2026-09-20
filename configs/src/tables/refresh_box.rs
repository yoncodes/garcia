use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct RefreshBox {
    pub id: i32,
    pub limit_def: i32,
    pub total_refresh_num: i32,
    #[serde(default)]
    pub city_weight: Vec<TablePair<i32>>,
    #[serde(default)]
    pub box_num_max: Vec<TablePair<i32>>,
    #[serde(default)]
    pub box_num_min: Vec<TablePair<i32>>,
}
