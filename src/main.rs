mod config;
mod rpc;
mod tools;

use rmcp::{ServiceExt, transport::stdio};
use tools::TitanMcpServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::Config::from_env();
    let server = TitanMcpServer::new(config);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
