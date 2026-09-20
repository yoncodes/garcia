use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct DailyTaskDefinition {
    pub id: i32,
    #[serde(default)]
    pub group: i32,
    #[serde(default)]
    pub group_selet: i32,
    pub finish_limit: Vec<TablePair<String>>,
    pub reward: i32,
}
