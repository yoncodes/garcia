use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct MaidTalent {
    pub id: i32,
    pub maid_id: i32,
    pub position: i32,
}
