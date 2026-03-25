# titan-builder-mcp

```
  _   _ _                   _           _ _     _
 | |_(_) |_ __ _ _ __      | |__  _   _(_) | __| | ___ _ __
 | __| | __/ _` | '_ \ ____| '_ \| | | | | |/ _` |/ _ \ '__|
 | |_| | || (_| | | | |____| |_) | |_| | | | (_| |  __/ |
  \__|_|\__\__,_|_| |_|    |_.__/ \__,_|_|_|\__,_|\___|_|
                                           _ __ ___   ___ _ __
                                          | '_ ` _ \ / __| '_ \
                                          | | | | | | (__| |_) |
                                          |_| |_| |_|\___| .__/
                                                         |_|
```

[![CI](https://github.com/VPC-byte/titan-builder-mcp/actions/workflows/release.yml/badge.svg)](https://github.com/VPC-byte/titan-builder-mcp/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Rust MCP server for [Titan Builder](https://titanbuilder.xyz) — bring Ethereum block builder capabilities to AI coding agents.**

The first MCP (Model Context Protocol) server for an Ethereum block builder. Enables AI agents like Claude Code and Cursor to interact with Titan Builder's MEV infrastructure directly — send bundles, debug failures, submit transactions, and more.

## Highlights

- **Enhanced Bundle Tracing** — AI-ready diagnostic analysis on top of `titan_getBundleStats` — [see below](#enhanced-bundle-tracing)
- **5 MCP tools** mapping to Titan Builder's complete API surface
- **Single binary** — zero runtime dependencies, compiled Rust
- **Regional endpoints** — US, EU, Asia for minimal latency

## Architecture

```
┌─────────────────┐     stdio      ┌──────────────────┐     HTTPS     ┌─────────────────────┐
│  Claude Code /   │◄──────────────►│  titan-builder-   │◄────────────►│  Titan Builder RPC   │
│  Cursor / AI     │   MCP JSON     │  mcp              │   JSON-RPC   │  us.rpc.titanbuilder │
│  Agent           │                │                    │              │  .xyz                │
└─────────────────┘                └──────────────────┘              └─────────────────────┘
```

## Quick Start

**Install:**
```bash
cargo install titan-builder-mcp
```

**Configure** (Claude Code `~/.claude/settings.json`):
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

**Use:**
Once configured, your AI agent can send bundles, check bundle status, submit transactions, and debug failures through natural language.

## Enhanced Bundle Tracing

**The core feature.** When your bundle fails, you shouldn't have to guess why.

Titan Builder provides [Bundle Tracing](https://docs.titanbuilder.xyz) via `titan_getBundleStats` — returning status codes, builder payment data, and error details. Our MCP server adds an **AI-ready diagnostic layer** on top: root cause analysis, actionable fix suggestions, and payment flags.

### Without Enhanced Bundle Tracing

Raw API response — you see *what* happened, but not *why* or *what to do*:

```json
{
  "status": "SimulationFail",
  "builderPayment": "0",
  "builderPaymentWhenIncluded": "0",
  "error": "BundleRevert. Reverting Hash: 0xa1b2c3d4e5f6...789"
}
```

### With Enhanced Bundle Tracing

Same response + structured diagnostic analysis:

```
## Raw Response
{
  "status": "SimulationFail",
  "builderPayment": "0",
  "builderPaymentWhenIncluded": "0",
  "error": "BundleRevert. Reverting Hash: 0xa1b2c3d4e5f6...789"
}

## Analysis
Status: SimulationFail
Root cause: Transaction 0xa1b2c3d4e5f6...789 reverted during top-of-block simulation.

Suggestions:
- Add 0xa1b2c3d4e5f6...789 to revertingTxHashes if this revert is acceptable
- Verify the transaction succeeds via estimateGas against current state
- Ensure builder payment (post-bundle balance - pre-bundle balance) is > 0
```

### What it analyzes

| Capability | Description |
|---|---|
| **Root cause identification** | Maps each of 7 bundle statuses to a human-readable explanation |
| **Actionable suggestions** | Specific steps to fix the issue — not generic advice |
| **Payment analysis** | Flags when `builderPayment` is 0 (your bundle pays nothing to the builder) |
| **Revert detection** | Extracts reverting transaction hashes from error messages and suggests adding them to `revertingTxHashes` |

This is pure analysis — no additional API calls. It runs entirely on the response data from `titan_getBundleStats`.

## Tools

| Tool | Titan API | Description |
|---|---|---|
| `send_bundle` | `eth_sendBundle` | Send an atomic bundle of signed transactions |
| `cancel_bundle` | `eth_cancelBundle` | Cancel a bundle by replacement UUID |
| `get_bundle_stats` | `titan_getBundleStats` | Query bundle status **with diagnostic analysis** |
| `send_raw_transaction` | `eth_sendRawTransaction` | Submit a signed transaction via private mempool |
| `send_blobs` | `eth_sendBlobs` | Send blob transaction permutations for optimal inclusion |

## Bundle Status Reference

| Status | Meaning | Diagnostic |
|---|---|---|
| `Received` | Arrived too late for the pool | Submit earlier, use regional endpoint |
| `Invalid` | Malformed bundle | Check RLP, chain ID, nonces, block number |
| `SimulationFail` | Revert or zero builder payment | Fix reverting tx or increase priority fee |
| `SimulationPass` | Passed but too late | Submit earlier in the slot |
| `ExcludedFromBlock` | Insufficient bribe (99% of cases) | Increase priority fee |
| `IncludedInBlock` | Lost to a more valuable block | Increase bribe |
| `Submitted` | Success — submitted to relay | Monitor for on-chain inclusion |

## Configuration

| Environment Variable | Description | Default |
|---|---|---|
| `TITAN_RPC_URL` | Titan Builder RPC endpoint | `https://rpc.titanbuilder.xyz` |
| `TITAN_TIMEOUT_MS` | HTTP request timeout (ms) | `10000` |

### Regional Endpoints

| Region | URL |
|---|---|
| Global (geo-routed) | `https://rpc.titanbuilder.xyz` |
| United States | `https://us.rpc.titanbuilder.xyz` |
| Europe | `https://eu.rpc.titanbuilder.xyz` |
| Asia | `https://ap.rpc.titanbuilder.xyz` |
| Testnet (Hoodi) | `https://rpc-hoodi.titanbuilder.xyz` |

## Building from Source

```bash
git clone https://github.com/VPC-byte/titan-builder-mcp.git
cd titan-builder-mcp
cargo build --release
```

## Related

- [titan-builder-skill](https://github.com/VPC-byte/titan-builder-skill) — Claude Code skill for MEV development assistance and bundle debugging

## License

[MIT](LICENSE)
