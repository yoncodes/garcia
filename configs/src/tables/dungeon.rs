use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Dungeon {
    pub dungeon_id: i32,
    pub dungeon_type: i32,
}
