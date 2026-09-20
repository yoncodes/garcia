use serde::Deserialize;

use super::TablePair;

#[derive(Debug, Clone, Deserialize)]
pub struct PartnerParameters {
    pub quality: i32,
    #[serde(rename = "partner_exp")]
    pub base_exp: i32,
    pub sell: TablePair<i32>,
}
