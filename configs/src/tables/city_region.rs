use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CityRegion {
    pub id: i32,
    #[serde(default)]
    pub reputation_exp: i32,
    #[serde(default)]
    pub reputation_money: i32,
    #[serde(default)]
    pub money_daily_limit: i32,
}
