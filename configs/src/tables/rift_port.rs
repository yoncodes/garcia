use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RiftPort {
    pub id: i32,
    pub group: i32,
    pub port_id: i32,
    pub base_level: i32,
    pub step: i32,
    pub end_time: i32,
    pub jump_para1: i32,
    pub jump_para2: i32,
    pub jump_max: i32,
    pub extra_score_time: i32,
    pub extra_score: i32,
}
