# MCP Server Gold Diamond Web3 Testing

This MCP (Model Context Protocol) server provides tools for working with the Gold Diamond Web3 testing suite from the DEX-OS-V2 project.

## Features

- Search Web3 tests by keyword
- Get summary statistics of all Web3 tests
- List available Web3 test files
- Generate implementation plans for Web3 tests based on test cases

## Available Tools

### `web3.search_tests`
Search Web3 tests by keyword across all gold diamond Web3 test files.

### `web3.get_summary`
Get a summary of all gold diamond Web3 tests including file counts and test counts.

### `web3.list_test_files`
List all available Web3 test files in the gold layer.

### `web3.implement_test`
Generate an implementation plan for a Web3 test based on a specific test case.

## Setup

1. Install dependencies:
   ```bash
   npm install
   ```

2. Build the server:
   ```bash
   npm run build
   ```

3. Start the server:
   ```bash
   npm start
   ```

## Usage

The server communicates over stdio and is designed to be used with MCP-compatible clients.