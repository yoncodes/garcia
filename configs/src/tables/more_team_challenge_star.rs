use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct MoreTeamChallengeStar {
    pub id: i32,
    pub group_id: i32,
    pub dungeon_id: i32,
    pub star_limit: i32,
    pub reward: Vec<TablePair<i32>>,
}
