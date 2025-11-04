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

This project is licensed under the MIT License.# DEX-OS-V1
