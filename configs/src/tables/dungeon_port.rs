use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct DungeonPort {
    pub id: i32,
    pub port_id: i32,
    pub level_need: i32,
    pub port_group_id: i32,
    #[serde(default)]
    #[serde(rename = "open_limit_def")]
    pub unlock_conditions: Vec<TablePair<String>>,
    #[serde(default)]
    pub cost: Vec<TablePair<i32>>,
    #[serde(default)]
    #[serde(rename = "main_drop_show")]
    pub displayed_rewards: Vec<TablePair<i32>>,
    #[serde(default)]
    #[serde(rename = "drop_show")]
    pub guaranteed_rewards: Vec<TablePair<i32>>,
    #[serde(default)]
    #[serde(rename = "drop_show2")]
    pub possible_rewards: Vec<TablePair<i32>>,
    #[serde(default)]
    pub maid_exp: Option<TablePair<i32>>,
    #[serde(default)]
    pub favor: i32,
    #[serde(default)]
    #[serde(rename = "max_drop")]
    pub maximum_drops: Vec<TablePair<i32>>,
    #[serde(default)]
    #[serde(rename = "drop_package")]
    pub drop_packages: Vec<TablePair<i32>>,
    #[serde(default)]
    pub boss_final_drop: i32,
    #[serde(default)]
    pub n_times: i32,
    #[serde(default)]
    pub sweep: i32,
    #[serde(default)]
    pub auto: i32,
    #[serde(default)]
    #[serde(rename = "recommend_element")]
    pub recommended_elements: Vec<i32>,
    #[serde(default)]
    pub countdown_type: i32,
    #[serde(default)]
    #[serde(rename = "rkb_time")]
    pub soft_fury_enter_time: i32,
    #[serde(default)]
    #[serde(rename = "rkb_nexttime")]
    pub soft_fury_add_layer_time: i32,
}
