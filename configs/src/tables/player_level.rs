use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PlayerLevel {
    #[serde(rename = "lv")]
    pub level: i32,
    #[serde(default)]
    pub exp: i32,
    #[serde(rename = "phy", default)]
    pub stamina: i32,
    #[serde(rename = "phy_max", default)]
    pub max_stamina: i32,
    #[serde(rename = "world_lv", default)]
    pub required_world_level: i32,
}
