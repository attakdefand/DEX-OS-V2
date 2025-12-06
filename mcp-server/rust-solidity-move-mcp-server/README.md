# CodeFix OS MCP Server

An MCP (Model Context Protocol) server that analyzes, fixes, and debugs code in Rust, Solidity, and Move programming languages.

## Features

- **Multi-language Support**: Works with Rust, Solidity, and Move
- **Error Analysis**: Identifies syntax, semantic, and runtime errors
- **Code Fixing**: Generates patches for common programming errors
- **Debugging**: Provides insights into runtime issues
- **Security Auditing**: Identifies potential security vulnerabilities

## Available Tools

### Rust Tools
- `rust.analyze_errors` - Parse rustc errors & classify them
- `rust.fix_errors` - Generate correct code patches
- `rust.debug` - Detect runtime issues like panics, deadlocks, memory leaks
- `rust.refactor` - Rewrite code for safety/performance

### Solidity Tools
- `sol.analyze` - Parse solc/forge errors
- `sol.fix` - Patch Solidity code
- `sol.debug` - Foundry test debug
- `sol.audit` - Security improvements

### Move Tools
- `move.analyze` - Parse Move compiler/test errors
- `move.fix` - Patch Move modules
- `move.debug` - Trace abort/test failures

### Utility Tools
- `project.scan` - Detect languages, modules, errors
- `project.autofix` - Apply patches for all errors
- `patch.validate` - Run tests to confirm fix
- `patch.plan` - Explain the diff before applying

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

### Running the Demo

To see the server in action, run:
```bash
npm run demo
```

This will demonstrate:
1. Listing all available tools
2. Analyzing Rust code with errors
3. Analyzing Solidity code with errors
4. Analyzing Move code with errors

### Running the Error Types Demo

To see all supported error types, run:
```bash
npm run error-types-demo
```

## Integration

The server can be integrated with any MCP-compatible development environment or tool. It accepts JSON-RPC requests over stdio and responds with structured JSON data.

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "rust.analyze_errors",
    "arguments": {
      "code": "fn main() { let x = y; }"
    }
  }
}
```

## Supported Error Types

### Rust
- **Syntax errors** - Parsing and grammar issues
- **Borrow checker violations** - Mutable/immutable borrow conflicts
- **Type mismatches** - When values don't match expected types
- **Unused imports/variables** - Code elements that aren't used
- **Lifetime issues** - Problems with object lifetimes
- **Trait bound mismatches** - When traits aren't properly implemented
- **Undefined types/functions** - References to non-existent code elements

### Solidity
- **Compiler errors** - Issues that prevent compilation
- **Reentrancy vulnerabilities** - Security issues with external calls
- **Parser errors** - Syntax and grammar problems
- **Visibility issues** - Incorrect function/variable visibility
- **Undefined variables** - References to non-existent variables
- **Inheritance problems** - Issues with contract inheritance
- **Storage layout issues** - Problems with state variable storage

### Move
- **Ability constraint violations** - Issues with resource abilities
- **Resource safety issues** - Linear/resource type violations
- **Type mismatches** - When values don't match expected types
- **Abort code errors** - Issues with abort/error handling
- **Module import issues** - Problems with module imports

## Development

To run the server in development mode with live reloading:
```bash
npm run dev
```

## License

This project is licensed under the MIT License.