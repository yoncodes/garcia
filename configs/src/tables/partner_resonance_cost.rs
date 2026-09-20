use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct PartnerResonanceCost {
    #[serde(rename = "reson_lv")]
    pub resonance_level: i32,
    #[serde(default)]
    pub cost: Option<TablePair<i32>>,
}
