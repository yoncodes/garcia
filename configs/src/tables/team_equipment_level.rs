use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct TeamEquipmentLevel {
    #[serde(rename = "lv")]
    pub level: i32,
    pub exp: i32,
}
