use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MallGoodsGroup {
    pub goods_group_id: i32,
    pub mall_id: i32,
    #[serde(default)]
    pub goods_group_open: String,
    #[serde(default)]
    pub goods_group_close: String,
    #[serde(default)]
    pub refresh_type: i32,
}
