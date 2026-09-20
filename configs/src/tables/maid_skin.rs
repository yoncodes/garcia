use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MaidSkin {
    pub id: i32,
    pub maid_id: i32,
}
