use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CityMap {
    pub id: i32,
    pub city_id: i32,
}
