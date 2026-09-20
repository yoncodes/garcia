#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalRecord {
    pub region: i32,
    pub local: String,
    pub local2: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FavorRecord {
    pub character_id: i32,
    pub level: i32,
    pub exp: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmsRecord {
    pub group_id: i32,
    pub read: bool,
    pub selected: Vec<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchiveUnlockRecord {
    pub archive_id: i32,
    pub unlocked_at: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileUnlockRecord {
    pub profile_type: i32,
    pub profile_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatRecord {
    pub id: i32,
    pub status: i32,
}
