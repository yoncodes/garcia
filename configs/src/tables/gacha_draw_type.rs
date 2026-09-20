use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct GachaDrawType {
    pub id: i32,
    #[serde(rename = "draw_drop_coin")]
    pub bonus_currency_id: i32,
    #[serde(rename = "draw_drop_coin_number")]
    pub bonus_currency_amount: i32,
}
