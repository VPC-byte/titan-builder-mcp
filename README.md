# titan-builder-mcp

[![CI](https://github.com/VPC-byte/titan-builder-mcp/actions/workflows/release.yml/badge.svg)](https://github.com/VPC-byte/titan-builder-mcp/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Rust MCP server for [Titan Builder](https://titanbuilder.xyz) — bring Ethereum block builder capabilities to AI coding agents.**

The first MCP (Model Context Protocol) server for an Ethereum block builder. Enables AI agents like Claude Code and Cursor to interact with Titan Builder's MEV infrastructure directly — send bundles, debug failures, submit transactions, and more.

## Highlights

- **5 MCP tools** mapping to Titan Builder's complete API surface
- **Enhanced Bundle Tracing** — AI-ready diagnostic analysis on top of `titan_getBundleStats`
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

## Tools

| Tool | Titan API | Description |
|---|---|---|
| `send_bundle` | `eth_sendBundle` | Send an atomic bundle of signed transactions |
| `cancel_bundle` | `eth_cancelBundle` | Cancel a bundle by replacement UUID |
| `get_bundle_stats` | `titan_getBundleStats` | Query bundle status with diagnostic analysis |
| `send_raw_transaction` | `eth_sendRawTransaction` | Submit a signed transaction via private mempool |
| `send_blobs` | `eth_sendBlobs` | Send blob transaction permutations for optimal inclusion |

## Enhanced Bundle Tracing

The `get_bundle_stats` tool goes beyond raw API responses. It returns the original JSON-RPC result plus a structured diagnostic analysis:

- **Root cause identification** — explains why a bundle was or wasn't included
- **Actionable suggestions** — specific steps to fix the issue
- **Payment analysis** — flags zero builder payment issues
- **Revert detection** — extracts reverting transaction hashes from error messages

This complements Titan Builder's [Bundle Tracing](https://docs.titanbuilder.xyz) feature with AI-ready diagnostics.

## Bundle Status Reference

| Status | Meaning |
|---|---|
| `Received` | Bundle received but arrived too late for the pool |
| `Invalid` | Malformed bundle (bad RLP, wrong block number, mined nonces, wrong chain ID) |
| `SimulationFail` | Transaction reverted or builder payment ≤ 0 during top-of-block simulation |
| `SimulationPass` | Passed simulation but submitted too late for inclusion |
| `ExcludedFromBlock` | Valid but not selected — usually insufficient bribe |
| `IncludedInBlock` | In a candidate block but another algorithm produced a more valuable block |
| `Submitted` | Included in a block submitted to a relay |

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
