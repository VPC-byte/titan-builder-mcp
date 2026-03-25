use rmcp::schemars;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SendRawTransactionParams {
    /// Signed raw transaction hex string. Required.
    pub signed_tx: String,
}
