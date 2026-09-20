use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RogueBuff {
    #[serde(rename = "ID")]
    pub id: i32,
    pub quality: i32,
    #[serde(rename = "buff_tag")]
    pub tag: i32,
}
