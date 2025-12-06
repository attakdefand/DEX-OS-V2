# Using the Rust Error Debugging MCP Server

This guide explains how to use the Rust Error Debugging MCP Server to analyze and fix Rust compilation errors.

## Quick Start

1. **Install dependencies**:
   ```bash
   npm install
   ```

2. **Run the server in development mode**:
   ```bash
   npm run dev-rust
   ```

3. **Run the test client**:
   ```bash
   npm run test-rust
   ```

## Building the Server

To build the TypeScript code into JavaScript:

```bash
npm run build-rust
```

This will compile the TypeScript files and place the output in the `dist` directory.

## Available Tools

The server provides four main tools for Rust error debugging:

### 1. `rust.analyze_errors`
Parses Rust compiler output and categorizes errors.

**Usage**:
```json
{
  "method": "tools/call",
  "params": {
    "name": "rust.analyze_errors",
    "arguments": {
      "compilerOutput": "string"
    }
  }
}
```

### 2. `rust.fix_code`
Applies fixes to Rust code based on error analysis.

**Usage**:
```json
{
  "method": "tools/call",
  "params": {
    "name": "rust.fix_code",
    "arguments": {
      "code": "string",
      "errors": "array"
    }
  }
}
```

### 3. `rust.debug_runtime`
Analyzes potential runtime issues in Rust code.

**Usage**:
```json
{
  "method": "tools/call",
  "params": {
    "name": "rust.debug_runtime",
    "arguments": {
      "code": "string"
    }
  }
}
```

### 4. `rust.full_analysis`
Performs complete analysis including error diagnosis, fixes, and improvements.

**Usage**:
```json
{
  "method": "tools/call",
  "params": {
    "name": "rust.full_analysis",
    "arguments": {
      "code": "string",
      "compilerOutput": "string"
    }
  }
}
```

## Example Usage

Here's an example of how to use the server with a sample Rust program that has errors:

1. **Create a Rust file with errors** (see `example-broken-code.rs`)

2. **Compile the Rust file to get error output**:
   ```bash
   rustc example-broken-code.rs
   ```

3. **Capture the compiler output**

4. **Send the output to the server for analysis**:
   ```json
   {
     "jsonrpc": "2.0",
     "id": 1,
     "method": "tools/call",
     "params": {
       "name": "rust.analyze_errors",
       "arguments": {
         "compilerOutput": "[captured compiler output]"
       }
     }
   }
   ```

5. **Apply fixes to the code**:
   ```json
   {
     "jsonrpc": "2.0",
     "id": 2,
     "method": "tools/call",
     "params": {
       "name": "rust.fix_code",
       "arguments": {
         "code": "[original code]",
         "errors": "[errors from analysis]"
       }
     }
   }
   ```

## Response Format

All responses follow a standardized format with four sections:

1. **Error Diagnosis**: Root cause identification and explanation
2. **Correct Fix**: Specific changes needed and explanation
3. **Fixed Full Code**: Complete, working Rust code
4. **Optional Enhancements**: Best practices and improvements

## Supported Error Types

The server can analyze and fix the following Rust error categories:

- Borrow checker violations
- Lifetime issues
- Trait bound mismatches
- Type mismatches
- Undefined types/imports
- Unused imports/variables
- Syntax errors
- Generic constraints

## Integration with Editors

The server can be integrated with any MCP-compatible editor or IDE:

1. **VS Code**: Use with the MCP extension
2. **Custom Tools**: Implement the MCP protocol in your application
3. **CI/CD Pipelines**: Integrate into automated testing workflows

## Troubleshooting

If you encounter issues:

1. **Ensure dependencies are installed**: Run `npm install`
2. **Check TypeScript compilation**: Run `npm run build-rust`
3. **Verify Rust installation**: Ensure `rustc` is available in your PATH
4. **Check Node.js version**: Node.js v16 or higher is required

## Contributing

To contribute to the Rust debugging server:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request