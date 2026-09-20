use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct RogueRules {
    pub pressure_limit: i32,
    pub rest_effect: Vec<TablePair<String>>,
    pub coinbox_default: i32,
    pub coin_ex_basic_rate: i32,
    pub first_node_coin: i32,
    pub buff_tag_formula: Vec<TablePair<i32>>,
    pub buff_giveup_coin: i32,
    pub buff_reroll_coin: i32,
    pub rouge_item_limit: i32,
    pub bossbox_item_cost: TablePair<i32>,
    pub bossbox_phy_cost: i32,
}
