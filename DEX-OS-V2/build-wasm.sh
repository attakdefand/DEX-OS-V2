#!/bin/bash

# Build script for DEX-OS WASM module

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null
then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WASM module
echo "Building WASM module..."
wasm-pack build dex-wasm --target web --out-dir ../pkg

echo "WASM module built successfully!"