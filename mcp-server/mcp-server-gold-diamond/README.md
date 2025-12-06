# MCP Server Gold Diamond Protection Tests

This MCP (Model Context Protocol) server provides tools for working with the Gold Diamond protection tests from the DEX-OS-V2 project.

## Features

- Search protection tests by keyword
- Get summary statistics of all protection tests
- List available protection test files
- Generate implementation plans for protection features based on test cases

## Available Tools

### `protection.search_tests`
Search protection tests by keyword across all gold diamond protection test files.

### `protection.get_summary`
Get a summary of all gold diamond protection tests including file counts and test counts.

### `protection.list_test_files`
List all available protection test files in the gold layer.

### `protection.implement_feature`
Generate an implementation plan for a protection feature based on a specific test case.

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