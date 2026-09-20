use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BreakableObject {
    pub id: String,
    pub drop_id: i32,
}
