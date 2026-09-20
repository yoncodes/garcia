use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ProfileCosmetic {
    pub id: i32,
    #[serde(default)]
    pub maid_id: Option<i32>,
}
