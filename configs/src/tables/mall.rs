use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Mall {
    pub id: i32,
    #[serde(default)]
    pub mall_open: String,
    #[serde(default)]
    pub mall_close: String,
}
