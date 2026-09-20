use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct SkillStone {
    pub id: i32,
    pub quality: i32,
    #[serde(default)]
    pub under_maid: i32,
}
