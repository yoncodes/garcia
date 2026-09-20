use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BossRush {
    pub id: i32,
    pub order: i32,
    pub open_time: String,
    pub close_time: String,
    pub port_id: Vec<i32>,
    pub reward_preview: Vec<i32>,
}
