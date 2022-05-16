use serde::Serialize;
use serde::Deserialize;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct MpvCommandResponse {
    pub error: MpvCommandStatus,
    pub data: Option<serde_json::Value>,
    pub request_id: u64,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MpvCommandStatus {
    Success,
    Error,

    #[serde(rename = "invalid parameter")]
    InvalidParameter
}
