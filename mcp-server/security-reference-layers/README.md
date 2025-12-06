# Security Reference Layers MCP Server

This MCP (Model Context Protocol) server provides access to the DEX-OS-V2 security reference layers, allowing you to search and analyze security tests and vulnerability data.

## Features

- Search security tests by keyword across all reference layers
- Get summaries of security reference layers
- List all available security test files
- Access to over 60,000 Web3 vulnerabilities organized in 7 major groups

## Tools Available

### `security.search_tests`
Search security tests by keyword.

**Parameters:**
- `query` (string, required): Search query
- `layer` (string, optional): Layer to search in (e.g., "gold")

### `security.get_layer_summary`
Get summary of all security reference layers.

### `security.list_test_files`
List all available security test files.

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

## Documentation

- [Integration Guide](INTEGRATION_GUIDE.md) - How the reference layers are integrated and act in their intended role
- [Architecture](ARCHITECTURE.md) - Technical architecture and component diagram
- [Demo Script](demo-usage.js) - Simple demonstration of available tools

## Example Queries

Search for reentrancy vulnerabilities:
```json
{
  "method": "tools/call",
  "params": {
    "name": "security.search_tests",
    "arguments": {
      "query": "reentrancy"
    }
  }
}
```

Get a summary of all layers:
```json
{
  "method": "tools/call",
  "params": {
    "name": "security.get_layer_summary",
    "arguments": {}
  }
}
```