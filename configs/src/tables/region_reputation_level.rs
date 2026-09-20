use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct RegionReputationLevel {
    pub id: i32,
    pub exp: i32,
}
