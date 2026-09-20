use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct VersionChallenge {
    pub id: i32,
    #[serde(default)]
    pub pre_dungeon_id: i32,
    pub dungeon_diff: i32,
    pub activity_id: i32,
    pub open_time: String,
    pub close_time: String,
    pub star_limit_1: TablePair<i32>,
    pub star_limit_2: TablePair<i32>,
    pub star_limit_3: TablePair<i32>,
}
