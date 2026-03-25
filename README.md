# titan-builder-mcp

An MCP (Model Context Protocol) server for [Titan Builder](https://titanbuilder.xyz) — the high-performance Ethereum block builder.

This MCP server enables AI agents and LLM-powered tools to interact with Titan Builder's MEV infrastructure directly, including sending bundles, canceling bundles, tracing bundle status, and submitting raw transactions.

## Features

- **`eth_sendBundle`** — Send atomic bundles with full parameter support (reverting/dropping tx hashes, replacement UUID, refund, min timestamp)
- **`eth_cancelBundle`** — Cancel a previously submitted bundle by replacement UUID
- **`titan_getBundleStats`** — Trace and debug bundle inclusion status (Received, Invalid, SimulationFail, SimulationPass, ExcludedFromBlock, IncludedInBlock, Submitted)
- **`eth_sendRawTransaction`** — Submit signed raw transactions via Titan Builder's private RPC
- **`eth_sendBlobs`** — Send blob transaction permutations for optimal blob inclusion

## Quick Start

### Installation

```bash
npm install titan-builder-mcp
```

### Usage with Claude Code

Add to your Claude Code MCP config (`~/.claude/settings.json`):

```json
{
  "mcpServers": {
    "titan-builder": {
      "command": "npx",
      "args": ["titan-builder-mcp"],
      "env": {
        "TITAN_RPC_URL": "https://us.rpc.titanbuilder.xyz"
      }
    }
  }
}
```

### Usage with Cursor / other MCP clients

```json
{
  "mcpServers": {
    "titan-builder": {
      "command": "npx",
      "args": ["titan-builder-mcp"],
      "env": {
        "TITAN_RPC_URL": "https://us.rpc.titanbuilder.xyz"
      }
    }
  }
}
```

## Configuration

| Environment Variable | Description | Default |
|---|---|---|
| `TITAN_RPC_URL` | Titan Builder RPC endpoint | `https://rpc.titanbuilder.xyz` |

### Regional Endpoints

| Region | URL |
|---|---|
| Global (geo-routed) | `https://rpc.titanbuilder.xyz` |
| Europe | `https://eu.rpc.titanbuilder.xyz` |
| United States | `https://us.rpc.titanbuilder.xyz` |
| Asia | `https://ap.rpc.titanbuilder.xyz` |
| Testnet (Hoodi) | `https://rpc-hoodi.titanbuilder.xyz` |

## Tools

### `send_bundle`

Send an atomic bundle of transactions.

**Parameters:**
- `txs` (required) — Array of signed transaction hex strings
- `blockNumber` — Target block number (hex). Defaults to current block
- `revertingTxHashes` — Tx hashes allowed to revert or be discarded
- `droppingTxHashes` — Tx hashes allowed to be discarded (but may not revert)
- `replacementUuid` — UUID for bundle replacement/cancellation
- `refundPercent` — Percentage (0-99) of ETH reward to refund
- `refundRecipient` — Address to receive the refund
- `minTimestamp` — Minimum slot timestamp (unix epoch seconds)

### `cancel_bundle`

Cancel a previously submitted bundle.

**Parameters:**
- `replacementUuid` (required) — The UUID set when the bundle was submitted

### `get_bundle_stats`

Get the status and trace information for a submitted bundle.

**Parameters:**
- `bundleHash` (required) — The hash of the bundle to query

### `send_raw_transaction`

Submit a signed raw transaction.

**Parameters:**
- `signedTx` (required) — Signed raw transaction hex string

### `send_blobs`

Send blob transaction permutations for optimal inclusion.

**Parameters:**
- `txs` (required) — Array of blob transactions (one per blob permutation)
- `maxBlockNumber` — Last block number for inclusion (hex)

## Bundle Status Reference

| Status | Description |
|---|---|
| `Received` | Bundle received but arrived too late for the pool |
| `Invalid` | Invalid bundle (bad RLP, wrong block number, mined nonces, wrong Chain ID) |
| `SimulationFail` | Transaction reverted or builder payment <= 0 |
| `SimulationPass` | Passed simulation but sent too late for inclusion |
| `ExcludedFromBlock` | Valid but not selected (usually insufficient bribe) |
| `IncludedInBlock` | Included in a block candidate but not the winning block |
| `Submitted` | Included in a block submitted to a relay |

## License

MIT
