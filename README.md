# DEX-OS V1

A high-performance decentralized exchange core engine built with Rust, WebAssembly, and modern database technologies.

## Project Structure

- `dex-core/` - Core DEX engine logic (orderbook, AMM, etc.)
- `dex-wasm/` - WebAssembly bindings for browser integration
- `dex-db/` - Database layer for persistence
- `dex-api/` - HTTP API layer for external interactions

## Features

- High-performance orderbook matching engine
- Automated Market Maker (AMM) with constant product formula
- WebAssembly support for browser-based trading interfaces
- Database persistence layer with SQLx
- RESTful API for external integrations
- Designed for scalability and low-latency trading

## Prerequisites

- Rust toolchain (latest stable)
- wasm-pack (for WASM builds)
- PostgreSQL (for database functionality)
- Git (for version control and repository management)
- Node.js (for Codex AI assistance)

## Building

### Core Engine

```bash
cargo build
```

### WebAssembly Module

```bash
# On Unix-like systems:
./build-wasm.sh

# On Windows:
build-wasm.bat
```

### Running the API Server

```bash
cargo run -p dex-api
```

The API server will start on http://localhost:3030

### Authentication helpers

The API now exposes token issuance flows so the web UI (and CLI) can mint JWTs without copying secrets around:

- Configure `JWT_ISSUER`, `JWT_TTL_SECONDS` (default `900`), `JWT_MAX_TTL_SECONDS` (default `3600`), and `TRADER_SECRETS` (comma-separated `trader:secret` pairs) in your environment or `.env`.
- Wallet signatures use `/auth/challenge` + `/auth/token/wallet` with a per-address nonce. Tune the expiry via `WALLET_CHALLENGE_TTL_SECONDS` (default `300`).
- The CLI helper issues tokens locally: `cargo run -p dex-api --bin issue_token -- --trader-id alice --ttl-seconds 600`.

### Market data streams

- Retrieve depth snapshots via `GET /orderbook/depth?levels=10`.
- Subscribe to real-time updates using the WebSocket feed at `/ws/depth?levels=10` (the UI connects automatically and falls back to manual refresh when needed).

## Reference data maintenance

- After modifying `.reference/testing_web3_full.csv`, normalize the canonical identifiers via `cargo run -p reference-tools -- normalize web3` before committing.
- Sanity-check any `.reference` CSVs (detection, governance, protection, resilience, security, web3) with `cargo run -p reference-tools -- validate all` or limit the scope (e.g., `-- validate security`) to catch slug/count drift locally.

### Codex AI Assistant

This project includes Codex AI assistant integration for rapid development:

```bash
# On Windows:
codex.bat "Generate a new trading pair struct"

# On Unix-like systems:
chmod +x codex.sh
./codex.sh "Create a function to calculate trading fees"
```

For more detailed instructions, see [RUNNING-CODEX-IN-WSL.MD](RUNNING-CODEX-IN-WSL.MD).

## Architecture

The DEX-OS follows a modular architecture:

1. **Core Engine** (`dex-core`): Contains the business logic for orderbook management, matching, and AMM functionality.
2. **WebAssembly Interface** (`dex-wasm`): Provides WASM bindings for browser-based trading interfaces.
3. **Database Layer** (`dex-db`): Handles data persistence using SQLx with support for PostgreSQL.
4. **API Layer** (`dex-api`): Exposes RESTful endpoints for external integrations.

## Components

Based on the DEX-OS-V1.csv specification, this implementation includes:

- Orderbook with BTreeMap-based storage
- AMM with constant product formula (x*y=k)
- Price-time priority matching
- WASM interface for web integration
- Database persistence layer

## Git Repository Initialization

To initialize this project as a Git repository and push it to GitHub, you can use the provided scripts:

### On Windows:
```cmd
init-and-push-to-github.bat
```

### On Unix-like systems:
```bash
chmod +x init-and-push-to-github.sh
./init-and-push-to-github.sh
```

For detailed instructions on installing Git, see [GIT-INSTALLATION-GUIDE.md](GIT-INSTALLATION-GUIDE.md).

## License

This project is licensed under the MIT License.
