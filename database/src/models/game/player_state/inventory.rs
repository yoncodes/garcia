#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemRecord {
    pub user_item_id: i64,
    pub item_id: i32,
    pub amount: i32,
    pub remain_sec: i32,
    pub quality: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemAcquiredRecord {
    pub item_id: i32,
    pub amount: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemSpentRecord {
    pub item_id: i32,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartnerRecord {
    pub brek: i32,
    pub exp: i32,
    pub group_id: i32,
    pub id: i64,
    pub locked: i32,
    pub lv: i32,
    pub partner_id: i32,
    pub quality: i32,
    pub reson_lv: i32,
    pub skill_lv: i32,
    pub creat_at: i64,
}
