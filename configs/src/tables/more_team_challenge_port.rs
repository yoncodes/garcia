use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MoreTeamChallengePort {
    pub id: i32,
    pub dungeon_id: i32,
    pub port_id: i32,
    pub level_need: i32,
}
