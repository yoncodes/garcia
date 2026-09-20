use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TreasureLocation {
    pub id: i32,
    pub city_id: i32,
    #[serde(rename = "element_posion")]
    pub element_position: String,
    pub point_weight: i32,
    pub style: i32,
}
