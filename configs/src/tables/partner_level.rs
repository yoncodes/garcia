use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PartnerLevel {
    pub quality: i32,
    #[serde(rename = "lv")]
    pub level: i32,
    #[serde(rename = "lv_exp")]
    pub required_exp: i32,
    #[serde(rename = "break_point", default)]
    pub breakpoint: i32,
}
