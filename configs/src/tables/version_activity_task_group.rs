use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct VersionActivityTaskGroup {
    pub id: i32,
    pub activity_id: i32,
    pub open_time: String,
    pub close_time: String,
}
