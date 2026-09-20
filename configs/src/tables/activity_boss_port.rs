use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ActivityBossPort {
    pub id: i32,
    pub port_id: i32,
    pub level_need: i32,
}
