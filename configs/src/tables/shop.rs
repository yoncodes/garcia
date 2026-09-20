use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Shop {
    pub id: i32,
    pub shop_type: i32,
    pub refresh_type: i32,
    #[serde(default)]
    pub refresh_time: String,
    pub item_num: i32,
}
