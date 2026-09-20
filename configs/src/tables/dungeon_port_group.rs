use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DungeonPortGroup {
    pub port_group_id: i32,
    pub dungeon_id: i32,
}
