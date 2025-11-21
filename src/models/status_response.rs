use serde_json::Value;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct StatusResponse {
    pub status: String,
    #[serde(rename = "analyzeResult")]
    pub result: Option<Value>,
}
