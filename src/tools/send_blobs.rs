use rmcp::schemars;
use serde::Serialize;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendBlobsParams {
    /// Array of blob transactions. One transaction per blob permutation. Send all permutations (1-blob tx, 2-blob tx, etc.) for optimal inclusion.
    pub txs: Vec<String>,

    /// Hex-encoded block number of the last block in which the transactions should be included. Optional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_block_number: Option<String>,
}
