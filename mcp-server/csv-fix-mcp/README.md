# Code Fix MCP Server

This MCP (Model Context Protocol) server is designed to detect and fix errors in multiple codebases, with a focus on:

1. **DEX-OS-V2.csv file** - Fixing formatting issues and inconsistencies
2. **Rust codebases** - Compiling, formatting, and fixing errors
3. **Solidity codebases** - Building and fixing smart contract errors

## Features

### CSV Error Handling
1. **Error Detection**: Identifies common CSV formatting issues
2. **Structure Validation**: Validates CSV structure against expected format
3. **Error Correction**: Automatically fixes detected errors
4. **Backup Creation**: Creates backups before making changes

### Rust Code Error Handling
1. **Compilation Checking**: Runs `cargo check` to find compiler errors
2. **Code Formatting**: Runs `cargo fmt` to format Rust code
3. **File Operations**: Read, write, and patch Rust files

### Solidity Code Error Handling
1. **Smart Contract Building**: Uses `forge build` or `solc` to compile Solidity contracts
2. **Error Detection**: Parses compilation errors for fixing
3. **File Operations**: Read, write, and patch Solidity files

## Installation

1. Navigate to the csv-fix-mcp directory:
   ```bash
   cd d:\DEX-OS-V2\mcp-server\csv-fix-mcp
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

## Usage

### Starting the Server

```bash
node server.js
```

### Using the Tools

The server provides multiple tools for different purposes:

#### CSV Tools
1. **analyze_csv_errors**: Detects errors in the CSV file
2. **validate_csv_structure**: Validates the CSV structure
3. **fix_csv_errors**: Fixes detected errors in the CSV file

#### Rust Tools
1. **rust_check_project**: Runs `cargo check` in a Rust project
2. **rust_fmt_project**: Runs `cargo fmt` to format Rust code

#### Solidity Tools
1. **solidity_build_project**: Compiles Solidity projects with Forge or solc

#### File Utilities
1. **read_code_file**: Reads any text file
2. **write_code_file**: Writes content to a file
3. **apply_patch**: Applies search-and-replace patches to files

### Example Client Usage

```javascript
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";

const transport = new StdioClientTransport({
  command: 'node',
  args: ['./server.js']
});

const client = new Client({
  name: "code-fix-client",
  version: "1.0.0",
});

await client.connect(transport);

// Analyze CSV errors
const csvResult = await client.request("tools/call", {
  name: 'analyze_csv_errors',
  arguments: {
    csv_path: '../DEX-OS-V2/DEX-OS-V2.csv'
  }
});

// Check Rust project for errors
const rustResult = await client.request("tools/call", {
  name: 'rust_check_project',
  arguments: {
    project_path: '../../dex-core'
  }
});

// Apply a patch to fix an error
const patchResult = await client.request("tools/call", {
  name: 'apply_patch',
  arguments: {
    path: '../../dex-core/src/main.rs',
    patches: [
      {
        old_text: "let x = missing_semicolon",
        new_text: "let x = missing_semicolon;"
      }
    ]
  }
});
```

## Common Issues Detected

### CSV Issues
1. **Duplicate [IMPLEMENTED] markers** - Multiple instances of `[IMPLEMENTED]` in the same cell
2. **Duplicate Security tags** - Repeated security layer annotations
3. **Column count mismatches** - Inconsistent number of columns per row
4. **Structural issues** - Headers not matching expected format

### Rust Issues
1. **Compilation errors** - Syntax errors, type mismatches, missing imports
2. **Formatting issues** - Code that doesn't follow Rust formatting standards

### Solidity Issues
1. **Compilation errors** - Syntax errors, type mismatches, missing functions
2. **Contract issues** - Problems with Solidity smart contracts

## Error Fixing Capabilities

### CSV Fixing
1. Removes duplicate `[IMPLEMENTED]` markers, keeping only one instance
2. Eliminates duplicate security tags
3. Fixes column count inconsistencies
4. Preserves original data while correcting formatting issues
5. Creates backup files before making changes

### Code Fixing
1. Identifies compiler errors and warnings
2. Suggests fixes for common Rust and Solidity errors
3. Applies patches to fix syntax and logic errors
4. Formats code according to language standards

## Integration

To integrate with your existing MCP setup, add this to your configuration:

```json
{
  "mcpServers": {
    "code-fix-mcp": {
      "command": "node",
      "args": ["d:/DEX-OS-V2/mcp-server/csv-fix-mcp/server.js"]
    }
  }
}
```