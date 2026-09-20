use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct GachaReward {
    pub id: i32,
    #[serde(rename = "draw_pool_type")]
    pub pool_type: i32,
    #[serde(rename = "type")]
    pub reward_type: i32,
    pub quality: i32,
    #[serde(rename = "get_num")]
    pub amount: i32,
    #[serde(rename = "if_convert", default)]
    pub converts_duplicates: i32,
    pub convert: Option<TablePair<i32>>,
}
