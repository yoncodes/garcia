use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct CityChallenge {
    pub city_challenge_id: String,
    #[serde(default)]
    pub challenge_type: i32,
    #[serde(default)]
    pub complete_type: i32,
    #[serde(default)]
    pub complete_key: Vec<TablePair<i32>>,
    #[serde(default)]
    pub reward: Vec<i32>,
    #[serde(default)]
    pub reward_preview_1: Vec<TablePair<i32>>,
    #[serde(default)]
    pub reward_preview_2: Vec<TablePair<i32>>,
    #[serde(default)]
    pub reward_preview_3: Vec<TablePair<i32>>,
}

impl CityChallenge {
    pub fn reward_preview(&self, tier: usize) -> Option<&[TablePair<i32>]> {
        match tier {
            0 => Some(&self.reward_preview_1),
            1 => Some(&self.reward_preview_2),
            2 => Some(&self.reward_preview_3),
            _ => None,
        }
    }
}
