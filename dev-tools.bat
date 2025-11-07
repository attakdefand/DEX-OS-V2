@echo off
setlocal enabledelayedexpansion

REM Add Rust to PATH
set PATH=%PATH%;%USERPROFILE%\.cargo\bin

REM DEX-OS Development Tools Script for Windows

:menu
cls
echo ==================================
echo     DEX-OS Development Tools     
echo ==================================
echo 1. Check prerequisites
echo 2. Build project
echo 3. Run tests
echo 4. Build WASM module
echo 5. Run API server
echo 6. Check code formatting
echo 7. Run Clippy linter
echo 8. Run cargo-audit (security advisories)
echo 9. Run cargo-deny (dependency policies)
echo 10. Run tests with cargo-nextest
echo 11. Run all checks (build, test, format, clippy, audit, deny, nextest)
echo 0. Exit
echo ==================================

set /p choice=Enter your choice: 

if "%choice%"=="1" goto check_prerequisites
if "%choice%"=="2" goto build_project
if "%choice%"=="3" goto run_tests
if "%choice%"=="4" goto build_wasm
if "%choice%"=="5" goto run_api
if "%choice%"=="6" goto check_format
if "%choice%"=="7" goto run_clippy
if "%choice%"=="8" goto run_audit
if "%choice%"=="9" goto run_deny
if "%choice%"=="10" goto run_nextest
if "%choice%"=="11" goto run_all
if "%choice%"=="0" goto exit
goto menu

:check_prerequisites
echo [STATUS] Checking prerequisites...
rustc --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Rust is not installed. Please install Rust from https://rustup.rs/
    goto pause
)

cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Cargo is not installed. Please install Rust from https://rustup.rs/
    goto pause
)

wasm-pack --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [WARNING] wasm-pack is not installed. Installing...
    cargo install wasm-pack
)

cargo-audit --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [WARNING] cargo-audit is not installed. Installing...
    cargo install cargo-audit --locked
)

cargo-deny --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [WARNING] cargo-deny is not installed. Installing...
    cargo install cargo-deny --locked
)

cargo-nextest --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [WARNING] cargo-nextest is not installed. Installing...
    cargo install cargo-nextest --locked
)

echo [STATUS] All prerequisites are met!
goto pause

:build_project
echo [STATUS] Building the project...
wsl -d Ubuntu-24.04 -e bash -c "source ~/.cargo/env && cd /mnt/c/Users/USER/Documents/DEX-OS-V1 && cargo -Znext-lockfile-bump build"
if %errorlevel% equ 0 (
    echo [STATUS] Project built successfully!
) else (
    echo [ERROR] Failed to build the project
)
goto pause

:run_tests
echo [STATUS] Running tests...
wsl -d Ubuntu-24.04 -e bash -c "source ~/.cargo/env && cd /mnt/c/Users/USER/Documents/DEX-OS-V1 && cargo -Znext-lockfile-bump test"
if %errorlevel% equ 0 (
    echo [STATUS] All tests passed!
) else (
    echo [ERROR] Some tests failed
)
goto pause

:build_wasm
echo [STATUS] Building WASM module...
wasm-pack build dex-wasm --target web --out-dir ../pkg
if %errorlevel% equ 0 (
    echo [STATUS] WASM module built successfully!
) else (
    echo [ERROR] Failed to build WASM module
)
goto pause

:run_api
echo [STATUS] Starting API server...
cargo run -p dex-api
goto pause

:check_format
echo [STATUS] Checking code formatting...
wsl -d Ubuntu-24.04 -e bash -c "source ~/.cargo/env && cd /mnt/c/Users/USER/Documents/DEX-OS-V1 && cargo fmt -- --check"
if %errorlevel% equ 0 (
    echo [STATUS] Code is properly formatted!
) else (
    echo [ERROR] Code formatting issues found
)
goto pause

:run_clippy
echo [STATUS] Running Clippy linter...
wsl -d Ubuntu-24.04 -e bash -c "source ~/.cargo/env && cd /mnt/c/Users/USER/Documents/DEX-OS-V1 && cargo clippy -- -D warnings"
if %errorlevel% equ 0 (
    echo [STATUS] No Clippy warnings found!
) else (
    echo [ERROR] Clippy found issues
)
goto pause

:run_audit
echo [STATUS] Running cargo-audit...
cargo audit
if %errorlevel% equ 0 (
    echo [STATUS] No known security advisories found!
) else (
    echo [ERROR] cargo-audit reported advisories
)
goto pause

:run_deny
echo [STATUS] Running cargo-deny...
cargo deny check
if %errorlevel% equ 0 (
    echo [STATUS] Dependency policies satisfied!
) else (
    echo [ERROR] cargo-deny reported issues
)
goto pause

:run_nextest
echo [STATUS] Running cargo-nextest...
cargo nextest run
if %errorlevel% equ 0 (
    echo [STATUS] Nextest suite passed!
) else (
    echo [ERROR] cargo-nextest reported failures
)
goto pause

:run_all
call :check_prerequisites
call :build_project
call :run_tests
call :check_format
call :run_clippy
call :run_audit
call :run_deny
call :run_nextest
echo [STATUS] All checks completed successfully!
goto pause

:pause
echo.
pause
goto menu

:exit
echo [STATUS] Goodbye!
exit /b
