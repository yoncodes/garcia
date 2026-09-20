#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractRecord {
    pub object_id: String,
    pub interactive: bool,
    pub status: i32,
    pub count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeRecord {
    pub id: String,
    pub stars: i32,
    pub finished: bool,
    pub claimed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumRecord {
    pub id: i32,
    pub status: i32,
    pub sort: i32,
}
