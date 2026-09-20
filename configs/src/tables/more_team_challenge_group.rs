use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct MoreTeamChallengeGroup {
    pub id: i32,
    #[serde(default)]
    pub limit_type: Vec<TablePair<String>>,
    #[serde(default)]
    pub refresh_time: String,
    #[serde(default)]
    pub close_time: String,
}
