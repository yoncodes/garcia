use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct TeamEquipmentParameters {
    pub id: i32,
    pub exp: i32,
    #[serde(rename = "lv_max")]
    pub max_level: i32,
    #[serde(rename = "core_pos")]
    pub core_slots: i32,
}
