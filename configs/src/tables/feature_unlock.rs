use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct FeatureUnlock {
    pub id: i32,
    #[serde(rename = "on_off", default)]
    pub enabled: i32,
    #[serde(rename = "limit_def", default)]
    pub conditions: Vec<TablePair<String>>,
}
