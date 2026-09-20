use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct Recharge {
    pub id: i32,
    #[serde(rename = "type")]
    pub kind: i32,
    pub price: i32,
    #[serde(default)]
    pub goods: Option<TablePair<i32>>,
    #[serde(default)]
    pub first_goods: Option<TablePair<i32>>,
    #[serde(default)]
    pub other_goods: Option<TablePair<i32>>,
}
