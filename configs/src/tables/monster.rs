use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MonsterDefinition {
    pub id: String,
    pub id_crc: u32,
    #[serde(default)]
    pub drop_reward: i32,
}
