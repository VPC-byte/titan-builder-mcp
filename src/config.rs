use std::env;
use std::time::Duration;

const DEFAULT_RPC_URL: &str = "https://rpc.titanbuilder.xyz";
const DEFAULT_TIMEOUT_MS: u64 = 10_000;
pub const STATS_URL: &str = "https://stats.titanbuilder.xyz";

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub timeout: Duration,
}

impl Config {
    pub fn from_env() -> Self {
        let rpc_url = env::var("TITAN_RPC_URL").unwrap_or_else(|_| DEFAULT_RPC_URL.to_string());
        let timeout_ms: u64 = env::var("TITAN_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_TIMEOUT_MS);

        Self {
            rpc_url,
            timeout: Duration::from_millis(timeout_ms),
        }
    }
}
