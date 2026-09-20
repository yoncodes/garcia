use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterFavor {
    pub id: i32,
    pub favor_step: i32,
    pub character_id: i32,
}
