use rmcp::schemars;
use serde::Serialize;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelBundleParams {
    /// The replacement UUID that was set when the bundle was submitted. Required.
    pub replacement_uuid: String,
}
