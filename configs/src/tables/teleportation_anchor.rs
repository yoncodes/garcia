use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TeleportationAnchor {
    pub id: String,
    pub city_id: i32,
    #[serde(default)]
    pub is_visible: i32,
    #[serde(default)]
    pub is_touch: i32,
}
