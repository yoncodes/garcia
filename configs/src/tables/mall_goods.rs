use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct MallGoods {
    pub id: i32,
    pub goods_group_id: i32,
    pub order: i32,
    pub goods: TablePair<i32>,
    #[serde(default)]
    pub buy_limit: i32,
    #[serde(default)]
    pub buy_single_limit: i32,
    pub price_type: i32,
    #[serde(default)]
    pub price: Option<TablePair<i32>>,
}
