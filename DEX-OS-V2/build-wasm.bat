@echo off

REM Build script for DEX-OS WASM module on Windows

REM Check if wasm-pack is installed
where wasm-pack >nul 2>&1
if %errorlevel% neq 0 (
    echo Installing wasm-pack...
    curl -o wasm-pack-init.exe https://rustwasm.github.io/wasm-pack/installer/init.exe
    wasm-pack-init.exe
    del wasm-pack-init.exe
)

REM Build the WASM module
echo Building WASM module...
wasm-pack build dex-wasm --target web --out-dir ../pkg

echo WASM module built successfully!