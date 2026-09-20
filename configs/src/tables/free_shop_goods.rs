use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct FreeShopGoods {
    pub id: i32,
    pub belong_shop: i32,
    pub goods: TablePair<i32>,
    pub refresh_type: i32,
    #[serde(default)]
    pub buy_limit: i32,
    pub price: TablePair<i32>,
}
