use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct EventNpcChoice {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(default)]
    pub choose_condition: Vec<TablePair<i32>>,
}
