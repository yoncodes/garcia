use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct EventBlessing {
    #[serde(rename = "ID")]
    pub id: i32,
    pub blessing_effect: Vec<TablePair<String>>,
    pub curse_effect: Vec<TablePair<String>>,
}
