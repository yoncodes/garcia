#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionProgressRecord {
    pub region_id: i32,
    pub level: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionRecord {
    pub cid: i32,
    pub in_time: i64,
    pub reward: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectionSuitRewardRecord {
    pub suit_id: i32,
    pub step: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectionPlaceRecord {
    pub collection_id: i32,
    pub platform_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TpMapRecord {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayingPortRecord {
    pub port_index_id: i32,
    pub port_info: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoldCoinRecord {
    pub city_id: i32,
    pub coin_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionResourceRecord {
    pub collection_id: String,
    pub remaining: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterPointRecord {
    pub day: i32,
    pub point_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionCoinDailyRecord {
    pub day: i32,
    pub coin_id: i32,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewardBoxRecord {
    pub id: String,
    pub status: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AchievementRecord {
    pub id: i32,
    pub took_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonsterManualRecord {
    pub gameplay_id: i32,
    pub enemy_hash: u32,
}
