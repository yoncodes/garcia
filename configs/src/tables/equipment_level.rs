use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct EquipmentLevel {
    #[serde(rename = "lv")]
    pub level: i32,
    pub exp: i32,
    #[serde(rename = "attr_up")]
    pub attribute_multiplier: f32,
    #[serde(rename = "break_point", default)]
    pub breakpoint: i32,
}
