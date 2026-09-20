use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct MaidDefinition {
    pub id: i32,
    pub character_id: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub english_name: String,
    pub element: i32,
    #[serde(default)]
    pub maid_quality: i32,
    pub quality: i32,
    #[serde(rename = "return_when_dup")]
    pub duplicate_reward: TablePair<i32>,
    pub quality_break: TablePair<i32>,
    #[serde(default)]
    pub talent_cost_group: Vec<i32>,
    #[serde(default)]
    pub in_team: i32,
    #[serde(default)]
    pub maid_group: i32,
}
