use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterFavorParameters {
    pub id: i32,
    pub exp: i32,
}
