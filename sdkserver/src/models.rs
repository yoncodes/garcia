use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct QuickSdkLoginRequest {
    pub uid: String,
    pub token: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct QuickSdkLoginResponse {
    pub code: u8,
    pub guid: u64,
    pub node_id: String,
    pub token: String,
    pub bl_reg: bool,
}

#[derive(Debug, Serialize)]
pub struct OpenNoticesResponse {
    pub code: u8,
    pub notices: Vec<()>,
}
