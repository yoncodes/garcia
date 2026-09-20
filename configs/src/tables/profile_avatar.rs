use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ProfileAvatar {
    pub id: i32,
    #[serde(default)]
    pub maid_id: i32,
}
