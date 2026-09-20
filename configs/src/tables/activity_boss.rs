use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ActivityBoss {
    pub id: i32,
    pub port_id: i32,
    pub damage_ratio: f64,
    pub jump_id: i32,
}
