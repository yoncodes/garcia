use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct MainStoryPort {
    pub id: i32,
    pub port_id: i32,
    #[serde(default)]
    pub maid_exp: Option<TablePair<i32>>,
}
