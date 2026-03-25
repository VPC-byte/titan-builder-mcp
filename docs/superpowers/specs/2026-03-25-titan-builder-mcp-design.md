# titan-builder-mcp Design Spec

## Overview

An MCP (Model Context Protocol) server for Titan Builder, the high-performance Ethereum block builder. The server exposes Titan Builder's MEV API as MCP tools, enabling AI coding agents (Claude Code, Cursor, etc.) to interact with Titan Builder's infrastructure directly.

**Goal:** Increase visibility in the MEV community through high-quality open-source Rust tooling around Titan Builder.

## Decisions

| Item | Decision |
|---|---|
| Language | Rust |
| Architecture | Modular (tools/ + rpc/ separation) |
| Scope | 5 tools, pure proxy, no private key handling |
| Transport | stdio only |
| Core dependencies | rmcp, reqwest, serde, tokio |
| Distribution | crates.io + GitHub Releases pre-built binaries (5 targets) |
| Error handling | Passthrough RPC errors, readable network errors, no retries |
| License | MIT |
| Repository | https://github.com/VPC-byte/titan-builder-mcp (public) |

## Project Structure

```
titan-builder-mcp/
├── Cargo.toml
├── README.md
├── LICENSE
├── .gitignore
├── .github/
│   └── workflows/
│       └── release.yml          # CI: cross-platform build + GitHub Releases
├── src/
│   ├── main.rs                  # Entry: parse config → start stdio MCP server
│   ├── config.rs                # Config: RPC URL, env var parsing
│   ├── rpc/
│   │   ├── mod.rs
│   │   ├── client.rs            # TitanRpcClient: HTTP JSON-RPC wrapper
│   │   └── types.rs             # JSON-RPC request/response types
│   └── tools/
│       ├── mod.rs               # Register all tools with MCP server
│       ├── send_bundle.rs       # eth_sendBundle
│       ├── cancel_bundle.rs     # eth_cancelBundle
│       ├── get_bundle_stats.rs  # titan_getBundleStats
│       ├── send_raw_tx.rs       # eth_sendRawTransaction
│       └── send_blobs.rs        # eth_sendBlobs
```

## Data Flow

```
Claude Code / Cursor
    ↓ stdin (MCP JSON)
titan-builder-mcp (stdio MCP server)
    ↓ parse tool call
tools/send_bundle.rs (construct JSON-RPC params)
    ↓
rpc/client.rs (reqwest POST)
    ↓ HTTPS
Titan Builder RPC (us.rpc.titanbuilder.xyz)
    ↓ JSON-RPC response
rpc/client.rs (parse response)
    ↓
tools/send_bundle.rs (format as MCP tool result)
    ↓ stdout (MCP JSON)
Claude Code / Cursor
```

## Component Responsibilities

| Component | Responsibility |
|---|---|
| `main.rs` | Read config → construct `TitanRpcClient` → create MCP `ServerHandler` → start stdio transport |
| `config.rs` | Read RPC endpoint from env var `TITAN_RPC_URL`, default `https://rpc.titanbuilder.xyz`. `titan_getBundleStats` always uses `https://stats.titanbuilder.xyz` |
| `rpc/client.rs` | `TitanRpcClient` struct holding `reqwest::Client` and RPC URL. Provides `async fn call(method, params) -> Result<Value>` |
| `rpc/types.rs` | `JsonRpcRequest` / `JsonRpcResponse` generic structs |
| `tools/*.rs` | Each file defines one tool: input params struct (with serde), calls `TitanRpcClient`, returns result |

## Tool Definitions

### 1. send_bundle (eth_sendBundle)

All param structs use `#[serde(rename_all = "camelCase")]` to match the Titan API's camelCase JSON field names.

```rust
#[serde(rename_all = "camelCase")]
struct SendBundleParams {
    txs: Vec<String>,                          // required, signed tx hex (can be empty for cancellations)
    block_number: Option<String>,              // optional, hex block number. Omit or "0x0" → defaults to current block
    reverting_tx_hashes: Option<Vec<String>>,
    dropping_tx_hashes: Option<Vec<String>>,
    replacement_uuid: Option<String>,
    refund_percent: Option<u64>,               // 0-99
    refund_recipient: Option<String>,          // address
    replacement_seq_number: Option<u64>,
    min_timestamp: Option<u64>,
}
// Output: { bundleHash: "0x..." } or error
```

### 2. cancel_bundle (eth_cancelBundle)

```rust
struct CancelBundleParams {
    replacement_uuid: String,                  // required
}
// Output: 200 or error
```

### 3. get_bundle_stats (titan_getBundleStats)

```rust
struct GetBundleStatsParams {
    bundle_hash: String,                       // required
}
// Output: { status, builderPayment, builderPaymentWhenIncluded, error }
// Note: uses stats.titanbuilder.xyz, not the main RPC endpoint
// Note: bundle trace is ready ~5 minutes after submission. Rate limit: 50 req/sec.
```

### 4. send_raw_transaction (eth_sendRawTransaction)

```rust
struct SendRawTransactionParams {
    signed_tx: String,                         // required, signed raw tx hex
}
// Output: 200 or error
```

### 5. send_blobs (eth_sendBlobs)

```rust
struct SendBlobsParams {
    txs: Vec<String>,                          // required, blob tx list
    max_block_number: Option<String>,          // optional, hex
}
// Output: 200 or error
```

## Error Handling

- RPC `error` field: passthrough directly to MCP tool result with `is_error: true`
- Network errors / timeouts: return human-readable error message
- No retries — let the caller decide
- No error swallowing

## Configuration

| Env Var | Description | Default |
|---|---|---|
| `TITAN_RPC_URL` | Titan Builder RPC endpoint | `https://rpc.titanbuilder.xyz` |
| `TITAN_TIMEOUT_MS` | HTTP request timeout in milliseconds | `10000` (10s) |

> `TITAN_RPC_URL` does NOT affect the stats endpoint. `titan_getBundleStats` always uses `https://stats.titanbuilder.xyz`.

### Regional Endpoints

| Region | URL |
|---|---|
| Global (geo-routed) | `https://rpc.titanbuilder.xyz` |
| Europe | `https://eu.rpc.titanbuilder.xyz` |
| United States | `https://us.rpc.titanbuilder.xyz` |
| Asia | `https://ap.rpc.titanbuilder.xyz` |
| Testnet (Hoodi) | `https://rpc-hoodi.titanbuilder.xyz` |
| Bundle Stats | `https://stats.titanbuilder.xyz` (hardcoded) |

## Distribution

### crates.io

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
```

Install: `cargo install titan-builder-mcp`

### GitHub Releases Pre-built Binaries

CI workflow (`.github/workflows/release.yml`):
- **Trigger**: push tag `v*` (e.g. `v0.1.0`)
- **Build matrix**:

| Target | OS |
|---|---|
| `x86_64-unknown-linux-gnu` | Linux x64 |
| `aarch64-unknown-linux-gnu` | Linux ARM64 |
| `x86_64-apple-darwin` | macOS x64 |
| `aarch64-apple-darwin` | macOS Apple Silicon |
| `x86_64-pc-windows-msvc` | Windows x64 |

- Package as `titan-builder-mcp-{target}.tar.gz` (Windows: `.zip`)
- Auto-create GitHub Release with all binaries

### MCP Client Configuration

```json
{
  "mcpServers": {
    "titan-builder": {
      "command": "titan-builder-mcp",
      "env": {
        "TITAN_RPC_URL": "https://us.rpc.titanbuilder.xyz"
      }
    }
  }
}
```

## Non-Goals

- No private key management or transaction signing
- No HTTP/SSE transport (stdio only for v0.1)
- No auxiliary tools beyond the 5 core API endpoints
- No retry logic or rate limiting
- No logging beyond stderr (may add `tracing` in future versions)
