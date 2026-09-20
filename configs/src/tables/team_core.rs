use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct TeamCore {
    pub id: i32,
    pub quality: i32,
}
