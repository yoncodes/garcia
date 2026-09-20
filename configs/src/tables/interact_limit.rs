use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct InteractLimit {
    pub interact_id: String,
    #[serde(default)]
    pub interact_num: i32,
    #[serde(default)]
    pub interact_state: i32,
    #[serde(default)]
    pub interactob_limit: Vec<TablePair<String>>,
}
