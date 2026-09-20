use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterMessageGroup {
    pub id: i32,
    #[serde(default)]
    pub limit_type: Vec<TablePair<String>>,
    pub character_id: i32,
    pub show_group_id: i32,
    #[serde(default)]
    pub pop_up: i32,
}
