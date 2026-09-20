#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity7DayClaimRecord {
    pub id: i32,
    pub took_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BattlePassRecord {
    pub id: i32,
    pub paid_status: i32,
    pub level: i32,
    pub exp: i32,
    pub exp_week: i32,
    pub week: i32,
    pub rewards: Vec<(i32, i32)>,
    pub tasks: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MallPurchaseRecord {
    pub goods_id: i32,
    pub bought: i32,
    pub period: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RechargePurchaseRecord {
    pub recharge_id: i32,
    pub purchase_count: i32,
    pub last_bought_at: i32,
    pub expires_at: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BossRushRecord {
    pub id: i32,
    pub ports: Vec<BossRushPortRecord>,
    pub rewards: Vec<i32>,
    pub ranking_rewards: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BossRushPortRecord {
    pub pid: i32,
    pub role_ids: Vec<i32>,
    pub damage: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActivityBossRecord {
    pub aid: i32,
    pub day: i32,
    pub damage: i64,
    pub daily_damage: i64,
    pub role_ids: Vec<i32>,
    pub rewards: Vec<i32>,
    pub daily_rewards: Vec<(i32, i32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionChallengeRecord {
    pub id: i32,
    pub star1: bool,
    pub star2: bool,
    pub star3: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoreTeamChallengeRecord {
    pub id: i32,
    pub cost_time: i32,
    pub star1: bool,
    pub star2: bool,
    pub star3: bool,
    pub cycle: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RiftRecord {
    pub id: i32,
    pub current_port_id: i32,
    pub best_buff_score: i32,
    pub best_score: i32,
    pub best_stage: i32,
    pub completion_count: i32,
    pub run_buff_score: i32,
    pub run_score: i32,
    pub run_duration: i32,
    pub active: bool,
    pub roles: Vec<i32>,
    pub tasks: Vec<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RougeProgressRecord {
    pub id: i32,
    pub pass: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GachaRecord {
    pub gacha_id: i32,
    pub all_count: i32,
    pub gacha_count_10: i32,
    pub reward_cnt: i32,
    pub reward_num: i32,
    pub taken_new_reward: bool,
    pub had_take_reward: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GachaLogRecord {
    pub gacha_id: i32,
    pub reward: i32,
    pub created_at: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckinRecord {
    pub aid: i32,
    pub check_days: i32,
    pub last_check_day: i32,
    pub claimed_days: Vec<i32>,
}
