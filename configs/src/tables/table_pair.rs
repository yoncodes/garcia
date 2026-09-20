use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TablePair<T> {
    #[serde(rename = "k")]
    pub key: i32,
    #[serde(rename = "v")]
    pub value: T,
}
