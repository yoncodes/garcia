#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BossRushOverride {
    pub event_id: i32,
    pub started_at: i32,
    pub expires_at: i32,
    pub instant_rewards: bool,
}
