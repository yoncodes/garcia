use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct GachaWeight {
    pub gacha_id: i32,
    pub rarity: Vec<TablePair<i32>>,
}
