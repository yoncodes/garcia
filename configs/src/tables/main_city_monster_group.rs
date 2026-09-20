use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MainCityMonsterGroup {
    pub group_id: String,
    pub city_id: i32,
}
