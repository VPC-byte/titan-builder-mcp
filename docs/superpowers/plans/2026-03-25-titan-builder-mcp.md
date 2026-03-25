# titan-builder-mcp Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust MCP server that proxies Titan Builder's MEV API as 5 MCP tools, distributed via crates.io and GitHub Releases.

**Architecture:** Modular Rust project with `tools/` (one file per MCP tool) and `rpc/` (HTTP JSON-RPC client) separation. The server runs on stdio transport using the `rmcp` crate. Each tool constructs a JSON-RPC request body, sends it to Titan Builder's RPC endpoint via `reqwest`, and returns the response as an MCP tool result.

**Tech Stack:** Rust, rmcp 1.2 (MCP server + macros + stdio transport), reqwest (HTTP), serde/serde_json (serialization), tokio (async runtime)

**Spec:** `docs/superpowers/specs/2026-03-25-titan-builder-mcp-design.md`
**API Reference:** `/data/service/titan-builder-mcp-skill/titan.md`

---

## File Map

| File | Responsibility |
|---|---|
| `Cargo.toml` | Package metadata, dependencies |
| `src/main.rs` | Entry point: init config → construct server → start stdio transport |
| `src/config.rs` | Read `TITAN_RPC_URL` and `TITAN_TIMEOUT_MS` from env vars with defaults |
| `src/rpc/mod.rs` | Re-export `client` and `types` modules |
| `src/rpc/types.rs` | `JsonRpcRequest<T>` and `JsonRpcResponse` structs |
| `src/rpc/client.rs` | `TitanRpcClient` — wraps `reqwest::Client`, provides `call(url, method, params) -> Result<Value>` |
| `src/tools/mod.rs` | `TitanMcpServer` struct with `ToolRouter`, `#[tool_handler]` impl for `ServerHandler` |
| `src/tools/send_bundle.rs` | `SendBundleParams` struct + handler function |
| `src/tools/cancel_bundle.rs` | `CancelBundleParams` struct + handler function |
| `src/tools/get_bundle_stats.rs` | `GetBundleStatsParams` struct + handler function |
| `src/tools/send_raw_tx.rs` | `SendRawTransactionParams` struct + handler function |
| `src/tools/send_blobs.rs` | `SendBlobsParams` struct + handler function |
| `.github/workflows/release.yml` | CI: cross-compile for 5 targets, publish GitHub Release |

---

## Task 1: Project Scaffolding (Cargo.toml + main.rs skeleton)

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "titan-builder-mcp"
version = "0.1.0"
edition = "2021"
description = "MCP server for Titan Builder — Ethereum block builder"
license = "MIT"
repository = "https://github.com/VPC-byte/titan-builder-mcp"
keywords = ["mcp", "mev", "ethereum", "titan-builder"]
categories = ["command-line-utilities", "web-programming"]

[dependencies]
rmcp = { version = "1.2", features = ["server", "macros", "transport-io"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

- [ ] **Step 2: Create minimal src/main.rs**

```rust
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
```

Note: This won't compile yet — we'll add each module in subsequent tasks. This establishes the top-level wiring.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml src/main.rs
git commit -m "feat: scaffold project with Cargo.toml and main.rs entry point"
```

---

## Task 2: Config Module

**Files:**
- Create: `src/config.rs`

- [ ] **Step 1: Implement Config**

```rust
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
```

- [ ] **Step 2: Commit**

```bash
git add src/config.rs
git commit -m "feat: add config module with env var parsing"
```

---

## Task 3: RPC Types

**Files:**
- Create: `src/rpc/mod.rs`
- Create: `src/rpc/types.rs`

- [ ] **Step 1: Create src/rpc/mod.rs**

```rust
pub mod client;
pub mod types;
```

- [ ] **Step 2: Create src/rpc/types.rs**

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct JsonRpcRequest<T: Serialize> {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: String,
    pub params: T,
}

impl<T: Serialize> JsonRpcRequest<T> {
    pub fn new(method: impl Into<String>, params: T) -> Self {
        Self {
            jsonrpc: "2.0",
            id: 1,
            method: method.into(),
            params,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcResponse {
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcError {
    pub code: Option<i64>,
    pub message: Option<String>,
}

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.code, &self.message) {
            (Some(code), Some(msg)) => write!(f, "RPC error {}: {}", code, msg),
            (None, Some(msg)) => write!(f, "RPC error: {}", msg),
            (Some(code), None) => write!(f, "RPC error code: {}", code),
            (None, None) => write!(f, "Unknown RPC error"),
        }
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add src/rpc/mod.rs src/rpc/types.rs
git commit -m "feat: add JSON-RPC request/response types"
```

---

## Task 4: RPC Client

**Files:**
- Create: `src/rpc/client.rs`

- [ ] **Step 1: Implement TitanRpcClient**

```rust
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
```

- [ ] **Step 2: Commit**

```bash
git add src/rpc/client.rs
git commit -m "feat: add TitanRpcClient HTTP JSON-RPC wrapper"
```

---

## Task 5: Tools Module Skeleton (TitanMcpServer)

**Files:**
- Create: `src/tools/mod.rs`

- [ ] **Step 1: Create the server struct with ToolRouter**

This file defines `TitanMcpServer`, registers all tools via `#[tool_router]`, and implements `ServerHandler` via `#[tool_handler]`. We start with an empty `#[tool_router]` impl and add tools in subsequent tasks.

```rust
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

    // Tools will be added here in Tasks 6-10
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
```

- [ ] **Step 2: Create placeholder files for each tool module**

Create empty files so the module declarations compile:
- `src/tools/send_bundle.rs` — empty
- `src/tools/cancel_bundle.rs` — empty
- `src/tools/get_bundle_stats.rs` — empty
- `src/tools/send_raw_tx.rs` — empty
- `src/tools/send_blobs.rs` — empty

- [ ] **Step 3: Verify build compiles**

```bash
cd /data/service/titan-builder-mcp && cargo build
```

Expected: compiles successfully (no tools registered yet, but the server skeleton works).

- [ ] **Step 4: Commit**

```bash
git add src/tools/
git commit -m "feat: add TitanMcpServer skeleton with ToolRouter and ServerHandler"
```

---

## Task 6: send_bundle Tool

**Files:**
- Modify: `src/tools/send_bundle.rs`
- Modify: `src/tools/mod.rs` (add `#[tool]` method)

- [ ] **Step 1: Define SendBundleParams in send_bundle.rs**

```rust
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
```

- [ ] **Step 2: Add send_bundle tool method to TitanMcpServer in mod.rs**

Add inside the `#[tool_router] impl TitanMcpServer` block (all required imports are already in the top-level `use` block from Task 5):

```rust
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
```

- [ ] **Step 3: Verify build compiles**

```bash
cd /data/service/titan-builder-mcp && cargo build
```

- [ ] **Step 4: Commit**

```bash
git add src/tools/send_bundle.rs src/tools/mod.rs
git commit -m "feat: add send_bundle tool (eth_sendBundle)"
```

---

## Task 7: cancel_bundle Tool

**Files:**
- Modify: `src/tools/cancel_bundle.rs`
- Modify: `src/tools/mod.rs` (add `#[tool]` method)

- [ ] **Step 1: Define CancelBundleParams in cancel_bundle.rs**

```rust
use rmcp::schemars;
use serde::Serialize;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelBundleParams {
    /// The replacement UUID that was set when the bundle was submitted. Required.
    pub replacement_uuid: String,
}
```

- [ ] **Step 2: Add cancel_bundle tool method to TitanMcpServer in mod.rs**

Add inside the `#[tool_router] impl TitanMcpServer` block:

```rust
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
```

- [ ] **Step 3: Verify build compiles**

```bash
cd /data/service/titan-builder-mcp && cargo build
```

- [ ] **Step 4: Commit**

```bash
git add src/tools/cancel_bundle.rs src/tools/mod.rs
git commit -m "feat: add cancel_bundle tool (eth_cancelBundle)"
```

---

## Task 8: get_bundle_stats Tool

**Files:**
- Modify: `src/tools/get_bundle_stats.rs`
- Modify: `src/tools/mod.rs` (add `#[tool]` method)

- [ ] **Step 1: Define GetBundleStatsParams in get_bundle_stats.rs**

```rust
use rmcp::schemars;
use serde::Serialize;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBundleStatsParams {
    /// The hash of the bundle to query. Required.
    pub bundle_hash: String,
}
```

- [ ] **Step 2: Add get_bundle_stats tool method to TitanMcpServer in mod.rs**

Note: this tool always uses `STATS_URL` (imported in Task 5), not the configured RPC URL.

```rust
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
```

- [ ] **Step 3: Verify build compiles**

```bash
cd /data/service/titan-builder-mcp && cargo build
```

- [ ] **Step 4: Commit**

```bash
git add src/tools/get_bundle_stats.rs src/tools/mod.rs
git commit -m "feat: add get_bundle_stats tool (titan_getBundleStats)"
```

---

## Task 9: send_raw_transaction Tool

**Files:**
- Modify: `src/tools/send_raw_tx.rs`
- Modify: `src/tools/mod.rs` (add `#[tool]` method)

- [ ] **Step 1: Define SendRawTransactionParams in send_raw_tx.rs**

```rust
use rmcp::schemars;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SendRawTransactionParams {
    /// Signed raw transaction hex string. Required.
    pub signed_tx: String,
}
```

Note: `eth_sendRawTransaction` takes a flat array `["0x..."]` not an object, so we don't derive `Serialize` on this struct. The tool method will construct params manually.

- [ ] **Step 2: Add send_raw_transaction tool method to TitanMcpServer in mod.rs**

```rust
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
```

- [ ] **Step 3: Verify build compiles**

```bash
cd /data/service/titan-builder-mcp && cargo build
```

- [ ] **Step 4: Commit**

```bash
git add src/tools/send_raw_tx.rs src/tools/mod.rs
git commit -m "feat: add send_raw_transaction tool (eth_sendRawTransaction)"
```

---

## Task 10: send_blobs Tool

**Files:**
- Modify: `src/tools/send_blobs.rs`
- Modify: `src/tools/mod.rs` (add `#[tool]` method)

- [ ] **Step 1: Define SendBlobsParams in send_blobs.rs**

```rust
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
```

- [ ] **Step 2: Add send_blobs tool method to TitanMcpServer in mod.rs**

```rust
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
```

- [ ] **Step 3: Verify full build compiles**

```bash
cd /data/service/titan-builder-mcp && cargo build
```

Expected: compiles with all 5 tools registered.

- [ ] **Step 4: Commit**

```bash
git add src/tools/send_blobs.rs src/tools/mod.rs
git commit -m "feat: add send_blobs tool (eth_sendBlobs)"
```

---

## Task 11: GitHub Actions Release CI

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Create the release workflow**

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            archive: tar.gz
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
            archive: tar.gz
          - target: x86_64-apple-darwin
            os: macos-latest
            archive: tar.gz
          - target: aarch64-apple-darwin
            os: macos-latest
            archive: tar.gz
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            archive: zip

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross (Linux ARM64)
        if: matrix.target == 'aarch64-unknown-linux-gnu'
        run: cargo install cross --git https://github.com/cross-rs/cross

      - name: Build (cross)
        if: matrix.target == 'aarch64-unknown-linux-gnu'
        run: cross build --release --target ${{ matrix.target }}

      - name: Build (native)
        if: matrix.target != 'aarch64-unknown-linux-gnu'
        run: cargo build --release --target ${{ matrix.target }}

      - name: Package (tar.gz)
        if: matrix.archive == 'tar.gz'
        run: |
          cd target/${{ matrix.target }}/release
          tar czf ../../../titan-builder-mcp-${{ matrix.target }}.tar.gz titan-builder-mcp
          cd ../../..

      - name: Package (zip)
        if: matrix.archive == 'zip'
        shell: bash
        run: |
          cd target/${{ matrix.target }}/release
          7z a ../../../titan-builder-mcp-${{ matrix.target }}.zip titan-builder-mcp.exe
          cd ../../..

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: titan-builder-mcp-${{ matrix.target }}
          path: titan-builder-mcp-${{ matrix.target }}.*

  release:
    name: Create Release
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          merge-multiple: true

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: titan-builder-mcp-*
          generate_release_notes: true
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add cross-platform release workflow for GitHub Releases"
```

---

## Task 12: Final Build Verification + Push

- [ ] **Step 1: Full cargo build + check**

```bash
cd /data/service/titan-builder-mcp && cargo build
cd /data/service/titan-builder-mcp && cargo clippy -- -D warnings
```

Fix any warnings or errors.

- [ ] **Step 2: Push all commits to GitHub**

```bash
cd /data/service/titan-builder-mcp && git push origin main
```

- [ ] **Step 3: Verify on GitHub**

Run `gh repo view VPC-byte/titan-builder-mcp --web` or verify the commits are visible.
