use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RewardBox {
    pub id: String,
    pub drop_id: i32,
}
