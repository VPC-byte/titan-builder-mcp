use rmcp::schemars;
use serde::Serialize;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendBundleParams {
    /// Array of signed transaction hex strings to execute atomically. Can be empty for bundle cancellations.
    pub txs: Vec<String>,

    /// Hex-encoded block number for which this bundle is valid. Omit or set to "0x0" to default to current block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<String>,

    /// Transaction hashes that are allowed to revert or be discarded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverting_tx_hashes: Option<Vec<String>>,

    /// Transaction hashes that are allowed to be discarded but may not revert.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dropping_tx_hashes: Option<Vec<String>>,

    /// Arbitrary string for bundle replacement or cancellation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement_uuid: Option<String>,

    /// Percentage (0-99) of ETH reward to refund. If set, the builder constructs a refund tx automatically. Bundle is discarded if refund amount doesn't cover gas cost.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_percent: Option<u64>,

    /// Address to receive the ETH refund. Defaults to sender of the first transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_recipient: Option<String>,

    /// Monotonically increasing sequence number for bundles sharing the same replacementUuid. Higher sequence numbers replace lower ones.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement_seq_number: Option<u64>,

    /// Minimum slot timestamp (unix epoch seconds) for which this bundle is valid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_timestamp: Option<u64>,
}
