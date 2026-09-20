use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct RogueNode {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "gameplay_id")]
    pub gameplay_choices: Vec<TablePair<i32>>,
    #[serde(rename = "event1")]
    pub primary_event: TablePair<i32>,
    #[serde(rename = "event2")]
    pub secondary_event: Option<TablePair<i32>>,
    #[serde(rename = "event3")]
    pub tertiary_event: Option<TablePair<i32>>,
    #[serde(rename = "pressure_increase_fix")]
    pub pressure_multiplier: f32,
    #[serde(rename = "battle_coin_fix")]
    pub coin_multiplier: f32,
}
