use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct RogueWeeklyReward {
    pub id: i32,
    #[serde(rename = "point_need")]
    pub required_points: i32,
    #[serde(rename = "point_reward")]
    pub reward: TablePair<i32>,
}
