use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct EventNpc {
    #[serde(rename = "ID")]
    pub id: i32,
    pub choose_effect: Vec<i32>,
}
