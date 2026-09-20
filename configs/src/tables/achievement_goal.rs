use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AchievementGoal {
    pub id: i32,
    pub total: i32,
}
