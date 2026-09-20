use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct SevenDayActivityMilestone {
    pub id: i32,
    #[serde(default)]
    pub limit_type: Vec<TablePair<String>>,
    #[serde(default)]
    pub reward: Vec<TablePair<i32>>,
}
