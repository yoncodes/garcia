use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Rift {
    pub id: i32,
    pub open_time: String,
    pub close_time: String,
    pub rank_group: i32,
    pub reward_group: i32,
    pub buff_group: i32,
    pub gameplay_port_group: i32,
    pub time_battle: i32,
    pub time_rest: i32,
}
