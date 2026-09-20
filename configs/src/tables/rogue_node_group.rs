use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RogueNodeGroup {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "node_id")]
    pub nodes: Vec<i32>,
    #[serde(rename = "node_group_port")]
    pub port_id: i32,
}
