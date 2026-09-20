use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct ExtraEquipmentWord {
    pub id: i32,
    #[serde(rename = "pos")]
    pub slot: i32,
    pub effect: TablePair<f32>,
    #[serde(rename = "word_limit_max")]
    pub duplicate_limit: i32,
    #[serde(rename = "word_wheel")]
    pub wheel_position: i32,
}
