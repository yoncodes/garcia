use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CollectionResource {
    pub collection_id: String,
    pub city_id: i32,
    pub collection_num: i32,
    #[serde(default)]
    pub drop_id: Option<i32>,
}
