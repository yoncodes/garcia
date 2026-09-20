use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct RoguePort {
    pub id: i32,
    pub battle_coin: i32,
    pub buff_count: i32,
    pub gameplay_pressure: i32,
    pub battle_item: Vec<TablePair<i32>>,
    #[serde(rename = "bossbox_reward", default)]
    pub boss_box_reward: i32,
}
