use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct EquipmentDefinition {
    pub id: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub english_name: String,
    #[serde(rename = "pos")]
    pub slot: i32,
    pub quality: i32,
    pub group_id: i32,
}
