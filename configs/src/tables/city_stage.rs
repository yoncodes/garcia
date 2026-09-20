use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CityStage {
    pub city_id: i32,
    #[serde(default)]
    pub region: i32,
}
