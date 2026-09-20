use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct MoreTeamChallenge {
    pub id: i32,
    pub group_id: i32,
    #[serde(default)]
    pub pre_dungeon_id: i32,
    pub star_limit_1: TablePair<i32>,
    pub star_limit_2: TablePair<i32>,
    pub star_limit_3: TablePair<i32>,
}
