use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct AchievementChain {
    pub id: i32,
    #[serde(default)]
    pub achieve_finish_limit: Vec<TablePair<String>>,
}
