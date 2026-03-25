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
    schemars, tool, tool_router, tool_handler,
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

    // Tools will be added in subsequent tasks
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
