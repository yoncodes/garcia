use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Collection {
    pub id: i32,
    pub suit_id: i32,
    pub reward: i32,
}
