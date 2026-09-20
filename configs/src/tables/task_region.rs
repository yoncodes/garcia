use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct TaskRegion {
    pub task_group_id: i32,
    #[serde(rename = "type")]
    pub task_type: i32,
    pub region_id: i32,
    #[serde(default)]
    pub limit_type: Vec<TablePair<String>>,
    pub weight: i32,
    pub count: TablePair<i32>,
}
