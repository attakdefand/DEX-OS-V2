#!/bin/bash

# DEX-OS Development Tools Script

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[STATUS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    if ! command_exists rustc; then
        print_error "Rust is not installed. Please install Rust from https://rustup.rs/"
        return 1
    fi
    
    if ! command_exists cargo; then
        print_error "Cargo is not installed. Please install Rust from https://rustup.rs/"
        return 1
    fi
    
    if ! command_exists wasm-pack; then
        print_warning "wasm-pack is not installed. Installing..."
        cargo install wasm-pack
    fi

    if ! command_exists cargo-audit; then
        print_warning "cargo-audit is not installed. Installing..."
        cargo install cargo-audit --locked
    fi

    if ! command_exists cargo-deny; then
        print_warning "cargo-deny is not installed. Installing..."
        cargo install cargo-deny --locked
    fi

    if ! command_exists cargo-nextest; then
        print_warning "cargo-nextest is not installed. Installing..."
        cargo install cargo-nextest --locked
    fi
    
    print_status "All prerequisites are met!"
}

# Function to build the project
build_project() {
    print_status "Building the project..."
    # Use WSL for building as required
    wsl -d Ubuntu-24.04 -e bash -c "source ~/.cargo/env && cd /mnt/c/Users/USER/Documents/DEX-OS-V1 && cargo build"
    if [ $? -eq 0 ]; then
        print_status "Project built successfully!"
    else
        print_error "Failed to build the project"
        return 1
    fi
}

# Function to run tests
run_tests() {
    print_status "Running tests..."
    # Use WSL for testing as required
    wsl -d Ubuntu-24.04 -e bash -c "source ~/.cargo/env && cd /mnt/c/Users/USER/Documents/DEX-OS-V1 && cargo test"
    if [ $? -eq 0 ]; then
        print_status "All tests passed!"
    else
        print_error "Some tests failed"
        return 1
    fi
}

# Function to build WASM
build_wasm() {
    print_status "Building WASM module..."
    # Use WSL for WASM build as required
    wsl -d Ubuntu-24.04 -e bash -c "source ~/.cargo/env && cd /mnt/c/Users/USER/Documents/DEX-OS-V1 && wasm-pack build dex-wasm --target web --out-dir ../pkg"
    if [ $? -eq 0 ]; then
        print_status "WASM module built successfully!"
    else
        print_error "Failed to build WASM module"
        return 1
    fi
}

# Function to run the API server
run_api() {
    print_status "Starting API server..."
    # Use WSL for running API as required
    wsl -d Ubuntu-24.04 -e bash -c "source ~/.cargo/env && cd /mnt/c/Users/USER/Documents/DEX-OS-V1 && cargo run -p dex-api"
}

# Function to check code formatting
check_format() {
    print_status "Checking code formatting..."
    # Use WSL for formatting check as required
    wsl -d Ubuntu-24.04 -e bash -c "source ~/.cargo/env && cd /mnt/c/Users/USER/Documents/DEX-OS-V1 && cargo fmt -- --check"
    if [ $? -eq 0 ]; then
        print_status "Code is properly formatted!"
    else
        print_error "Code formatting issues found"
        return 1
    fi
}

# Function to run clippy for linting
run_clippy() {
    print_status "Running Clippy linter..."
    # Use WSL for clippy as required
    wsl -d Ubuntu-24.04 -e bash -c "source ~/.cargo/env && cd /mnt/c/Users/USER/Documents/DEX-OS-V1 && cargo clippy -- -D warnings"
    if [ $? -eq 0 ]; then
        print_status "No Clippy warnings found!"
    else
        print_error "Clippy found issues"
        return 1
    fi
}

# Function to run cargo-audit
run_audit() {
    print_status "Running cargo-audit (security advisories)..."
    cargo audit
    if [ $? -eq 0 ]; then
        print_status "No known security advisories found!"
    else
        print_error "Security advisories detected"
        return 1
    fi
}

# Function to run cargo-deny
run_deny() {
    print_status "Running cargo-deny (dependency policies)..."
    cargo deny check
    if [ $? -eq 0 ]; then
        print_status "Dependency policies satisfied!"
    else
        print_error "cargo-deny reported issues"
        return 1
    fi
}

# Function to run tests with cargo-nextest
run_nextest() {
    print_status "Running tests with cargo-nextest..."
    cargo nextest run
    if [ $? -eq 0 ]; then
        print_status "Nextest suite passed!"
    else
        print_error "Nextest reported failures"
        return 1
    fi
}

# Function to run all checks
run_all() {
    print_status "Running all checks..."
    check_prerequisites || return 1
    build_project || return 1
    run_tests || return 1
    check_format || return 1
    run_clippy || return 1
    run_audit || return 1
    run_deny || return 1
    run_nextest || return 1
    print_status "All checks completed successfully!"
}

# Main menu
show_menu() {
    echo "=================================="
    echo "    DEX-OS Development Tools"
    echo "=================================="
    echo "1. Check prerequisites"
    echo "2. Build project"
    echo "3. Run tests"
    echo "4. Build WASM module"
    echo "5. Run API server"
    echo "6. Check code formatting"
    echo "7. Run Clippy linter"
    echo "8. Run cargo-audit (security advisories)"
    echo "9. Run cargo-deny (dependency policies)"
    echo "10. Run tests with cargo-nextest"
    echo "11. Run all checks (build, test, format, clippy, audit, deny, nextest)"
    echo "0. Exit"
    echo "=================================="
}

# Main loop
while true; do
    show_menu
    read -p "Enter your choice: " choice
    
    case $choice in
        1)
            check_prerequisites
            ;;
        2)
            build_project
            ;;
        3)
            run_tests
            ;;
        4)
            build_wasm
            ;;
        5)
            run_api
            ;;
        6)
            check_format
            ;;
        7)
            run_clippy
            ;;
        8)
            run_audit
            ;;
        9)
            run_deny
            ;;
        10)
            run_nextest
            ;;
        11)
            run_all
            ;;
        0)
            print_status "Goodbye!"
            exit 0
            ;;
        *)
            print_error "Invalid choice. Please try again."
            ;;
    esac
    
    echo ""
    read -p "Press Enter to continue..."
    clear
done