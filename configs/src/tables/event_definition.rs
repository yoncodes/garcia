use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct EventDefinition {
    #[serde(rename = "ID")]
    pub id: i32,
    pub pressure_increase: i32,
}
