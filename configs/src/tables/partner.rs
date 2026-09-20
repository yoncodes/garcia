use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PartnerDefinition {
    pub id: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub english_name: String,
    pub quality: i32,
    pub group_id: i32,
    pub skill_cost_group: i32,
}
