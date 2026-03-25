use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

use super::types::{JsonRpcRequest, JsonRpcResponse};

#[derive(Debug, Clone)]
pub struct TitanRpcClient {
    client: Client,
    pub rpc_url: String,
}

impl TitanRpcClient {
    pub fn new(rpc_url: String, timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("failed to build HTTP client");
        Self { client, rpc_url }
    }

    pub async fn call<T: Serialize>(
        &self,
        url: &str,
        method: &str,
        params: T,
    ) -> Result<Value, String> {
        let request = JsonRpcRequest::new(method, params);

        let response = self
            .client
            .post(url)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let rpc_response: JsonRpcResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(error) = rpc_response.error {
            return Err(error.to_string());
        }

        Ok(rpc_response.result.unwrap_or(Value::Null))
    }
}
