use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct SevenDayActivity {
    pub id: i32,
    pub group: i32,
    #[serde(default)]
    pub finish_limit: Vec<TablePair<String>>,
    #[serde(default)]
    pub reward: Vec<TablePair<i32>>,
    #[serde(default)]
    pub jump_id: i32,
}
