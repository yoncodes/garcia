use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GoldCoin {
    pub goldcoin_id: String,
    pub gold_type: i32,
    pub num: i32,
}
