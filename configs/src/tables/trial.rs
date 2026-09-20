use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct Trial {
    pub id: i32,
    #[serde(default)]
    #[serde(rename = "open_limit_def")]
    pub unlock_conditions: Vec<TablePair<String>>,
    pub finish_task_id: i32,
    #[serde(default)]
    pub reward: Vec<TablePair<i32>>,
    #[serde(default)]
    pub initial_city: i32,
    #[serde(default)]
    pub initial_point: String,
    #[serde(default)]
    pub complete_point: String,
}
