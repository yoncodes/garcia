use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct WorldLevel {
    #[serde(rename = "lv")]
    pub level: i32,
    pub task_id: i32,
}
