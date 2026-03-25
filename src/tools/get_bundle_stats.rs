use rmcp::schemars;
use serde::Serialize;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBundleStatsParams {
    /// The hash of the bundle to query. Required.
    pub bundle_hash: String,
}
