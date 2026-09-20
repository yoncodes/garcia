use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct EquipmentParameters {
    pub quality: i32,
    #[serde(rename = "lv_max")]
    pub max_level: i32,
    #[serde(rename = "use_exp")]
    pub base_exp: i32,
    pub sell: Vec<TablePair<i32>>,
    #[serde(rename = "rate_count")]
    pub breakpoint_word_rolls: i32,
    #[serde(rename = "word_extra_count")]
    pub initial_word_rolls: i32,
    #[serde(rename = "word_extra_cost")]
    pub extra_word_costs: Vec<TablePair<i32>>,
}

impl EquipmentParameters {
    /// Maximum number of substat rolls at the relic's level cap. Repeated
    /// effects still consume a roll and are rendered as `+1` by the client.
    pub fn max_word_rolls(&self) -> i32 {
        self.breakpoint_word_rolls
            .saturating_add(self.initial_word_rolls)
            .max(0)
    }
}
