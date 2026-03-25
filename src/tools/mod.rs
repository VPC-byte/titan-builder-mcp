pub mod send_bundle;
pub mod cancel_bundle;
pub mod get_bundle_stats;
pub mod send_raw_tx;
pub mod send_blobs;

use rmcp::{
    ServerHandler,
    ErrorData as McpError,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_router, tool_handler,
};

use crate::config::{Config, STATS_URL};
use crate::rpc::client::TitanRpcClient;

#[derive(Debug, Clone)]
pub struct TitanMcpServer {
    pub rpc_client: TitanRpcClient,
    pub config: Config,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl TitanMcpServer {
    pub fn new(config: Config) -> Self {
        let rpc_client = TitanRpcClient::new(config.rpc_url.clone(), config.timeout);
        Self {
            rpc_client,
            config,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Send an atomic bundle of signed transactions to Titan Builder via eth_sendBundle. \
        Transactions must be pre-signed. The bundle is submitted to the configured Titan Builder RPC endpoint. \
        Returns the bundle hash on success. Supports refund configuration, replacement UUIDs, and reverting/dropping tx hashes.")]
    async fn send_bundle(
        &self,
        Parameters(params): Parameters<send_bundle::SendBundleParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .rpc_client
            .call(&self.rpc_client.rpc_url, "eth_sendBundle", vec![&params])
            .await;

        match result {
            Ok(value) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string()),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "Cancel a previously submitted bundle via eth_cancelBundle. \
        Requires the replacementUuid that was set when the bundle was submitted. \
        Note: cancellation is not guaranteed if submitted within 4 seconds of the final relay submission.")]
    async fn cancel_bundle(
        &self,
        Parameters(params): Parameters<cancel_bundle::CancelBundleParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .rpc_client
            .call(&self.rpc_client.rpc_url, "eth_cancelBundle", vec![&params])
            .await;

        match result {
            Ok(value) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string()),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "Get the status and trace information for a submitted bundle via titan_getBundleStats. \
        Returns status (Received, Invalid, SimulationFail, SimulationPass, ExcludedFromBlock, IncludedInBlock, Submitted), \
        builderPayment, and builderPaymentWhenIncluded. \
        Note: bundle trace is ready approximately 5 minutes after submission. Rate limit: 50 requests/sec. \
        This endpoint always uses stats.titanbuilder.xyz regardless of TITAN_RPC_URL configuration.")]
    async fn get_bundle_stats(
        &self,
        Parameters(params): Parameters<get_bundle_stats::GetBundleStatsParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .rpc_client
            .call(STATS_URL, "titan_getBundleStats", vec![&params])
            .await;

        match result {
            Ok(value) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string()),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "Submit a signed raw transaction to Titan Builder's private RPC via eth_sendRawTransaction. \
        The transaction must be pre-signed. Sent through Titan Builder's private transaction pool.")]
    async fn send_raw_transaction(
        &self,
        Parameters(params): Parameters<send_raw_tx::SendRawTransactionParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .rpc_client
            .call(
                &self.rpc_client.rpc_url,
                "eth_sendRawTransaction",
                vec![&params.signed_tx],
            )
            .await;

        match result {
            Ok(value) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string()),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(description = "Send blob transaction permutations to Titan Builder via eth_sendBlobs. \
        Enables sending all permutations of blob transactions from a single sender (same nonce, different blob counts) \
        for optimal blob inclusion. Titan Builder will sort and select the optimal combination.")]
    async fn send_blobs(
        &self,
        Parameters(params): Parameters<send_blobs::SendBlobsParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .rpc_client
            .call(&self.rpc_client.rpc_url, "eth_sendBlobs", vec![&params])
            .await;

        match result {
            Ok(value) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string()),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }
}

#[tool_handler]
impl ServerHandler for TitanMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(
                "Titan Builder MCP Server — interact with Titan Builder's MEV infrastructure. \
                 Send bundles, cancel bundles, check bundle status, submit raw transactions, \
                 and send blob transactions."
                    .to_string(),
            )
    }
}
