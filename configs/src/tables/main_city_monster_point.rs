use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MainCityMonsterPoint {
    pub id: String,
    pub group_id: String,
    pub monster_id: String,
}
