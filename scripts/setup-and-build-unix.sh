#!/usr/bin/env bash
set -euo pipefail

echo "[dex-os] Setup and build (Unix)"

if ! command -v rustup >/dev/null 2>&1; then
  echo "Installing rustup..."
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  source "$HOME/.cargo/env"
fi

rustup update stable
rustup default stable

# Optional: install wasm-pack if available via package manager
if command -v apt-get >/dev/null 2>&1; then
  sudo apt-get update
  sudo apt-get install -y build-essential pkg-config libssl-dev git curl
fi

if command -v brew >/dev/null 2>&1; then
  brew update
  brew install git
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$ROOT_DIR/DEX-OS-V2"
echo "[dex-os] Cleaning and building workspace..."
cargo clean
cargo build --workspace

echo "[dex-os] Building core only (optional)..."
cargo build -p dex-core || true
echo "[dex-os] Testing core only (optional)..."
cargo test -p dex-core || true

echo "[dex-os] Done."

