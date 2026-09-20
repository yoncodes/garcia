use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Tutorial {
    pub id: i32,
    pub group_id: i32,
}
