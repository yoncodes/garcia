use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct TeamEquipmentDefinition {
    pub id: i32,
    #[serde(rename = "type")]
    pub equipment_type: i32,
    pub quality: i32,
    #[serde(rename = "main_word_ID")]
    pub main_word_id: i32,
}
