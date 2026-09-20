use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct MaidLevel {
    #[serde(rename = "lv")]
    pub level: i32,
    pub exp: i32,
    #[serde(rename = "lv_limit", default)]
    pub required_rank: i32,
}
