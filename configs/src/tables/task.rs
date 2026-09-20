use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct Task {
    pub id: i32,
    pub task_type: i32,
    pub task_group: i32,
    pub sort: i32,
    #[serde(default)]
    pub next: Vec<String>,
    #[serde(default)]
    pub show_key: Vec<TablePair<String>>,
    #[serde(default)]
    #[serde(rename = "bouns")]
    pub rewards: Vec<TablePair<i32>>,
    #[serde(default)]
    pub done_key: Vec<TablePair<String>>,
    #[serde(default)]
    pub city_id: i32,
    #[serde(default)]
    #[serde(rename = "positsion")]
    pub position: Option<TablePair<String>>,
}
