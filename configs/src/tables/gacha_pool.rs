use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct GachaPool {
    pub id: i32,
    #[serde(rename = "show_sort")]
    pub display_order: i32,
    pub group_id: i32,
    #[serde(rename = "draw_pool_type")]
    pub pool_type: i32,
    pub time_type: i32,
    #[serde(default)]
    pub open_time: String,
    #[serde(default)]
    pub duration: i32,
    #[serde(rename = "cost_single")]
    pub single_draw_cost: TablePair<i32>,
    pub discount: i32,
    #[serde(default)]
    pub first_get: i32,
    pub reward_id: i32,
    #[serde(rename = "min_10")]
    pub guaranteed_draw_interval: i32,
    pub ten_reward_id: i32,
    #[serde(rename = "up_maid", default)]
    pub featured_maids: Vec<i32>,
    #[serde(rename = "up_weight", default)]
    pub featured_weights: Vec<i32>,
    #[serde(rename = "up_detail", default)]
    pub featured_details: Vec<i32>,
    #[serde(rename = "open_limit_type", default)]
    pub open_conditions: Vec<TablePair<String>>,
    #[serde(rename = "close_limit_type", default)]
    pub close_conditions: Vec<TablePair<String>>,
    pub extra_reward_id: i32,
    pub mall_id: i32,
}
