use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct PartnerSkill {
    pub skill_group: i32,
    pub level: i32,
    #[serde(default)]
    pub item_cost: Vec<TablePair<i32>>,
}
