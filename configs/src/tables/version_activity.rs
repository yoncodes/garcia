use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct VersionActivity {
    pub id: i32,
    pub open_time: String,
    pub close_time: String,
    #[serde(default)]
    pub limit: Vec<TablePair<String>>,
}
