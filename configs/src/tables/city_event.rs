use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CityEvent {
    pub interact_id: String,
    #[serde(default)]
    pub interact_num: i32,
    #[serde(default)]
    pub reward_id: i32,
}
