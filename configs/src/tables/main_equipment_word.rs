use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct MainEquipmentWord {
    pub id: i32,
    pub group_id: i32,
    pub effect: TablePair<f32>,
}
