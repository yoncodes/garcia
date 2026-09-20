use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct RogueMode {
    #[serde(rename = "ID")]
    pub id: i32,
    pub group_id: i32,
    pub order: i32,
    #[serde(rename = "level_need")]
    pub required_level: i32,
    #[serde(rename = "first_node_event")]
    pub initial_event_id: i32,
    #[serde(rename = "first_node_port")]
    pub initial_port_id: i32,
    #[serde(rename = "node_group")]
    pub node_groups: Vec<i32>,
    #[serde(rename = "first_drop")]
    pub first_clear_rewards: Vec<TablePair<i32>>,
    pub point_reward: i32,
    pub tech_reward: i32,
    #[serde(rename = "coin_ex_rate")]
    pub coin_exchange_rate: i32,
    #[serde(rename = "open_limit_def", default)]
    pub unlock_conditions: Vec<TablePair<String>>,
}
