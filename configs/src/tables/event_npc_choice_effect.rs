use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct EventNpcChoiceEffect {
    #[serde(rename = "ID")]
    pub id: i32,
    pub effect_group: Vec<TablePair<String>>,
}
