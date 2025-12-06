# Rust Error Debugging MCP Server

This is a specialized Model Context Protocol (MCP) server designed specifically for debugging Rust code errors. It provides comprehensive analysis, fixing, and improvement suggestions for Rust compilation errors.

## Features

- **Error Analysis**: Parses Rust compiler output and categorizes errors by type
- **Automated Fixes**: Applies fixes for common Rust errors including:
  - Borrow checker violations
  - Lifetime issues
  - Trait bound mismatches
  - Type mismatches
  - Undefined types/imports
  - Unused imports/variables
  - Syntax errors
- **Runtime Issue Detection**: Identifies potential runtime issues in Rust code
- **Best Practice Suggestions**: Provides idiomatic Rust improvements

## Prerequisites

- Node.js (v16 or higher)
- Rust toolchain (rustc, cargo)
- TypeScript

## Installation

```bash
npm install
```

## Running the Server

To run the Rust debugging server in development mode:

```bash
npm run dev-rust
```

## Using the Server

The server provides several tools for Rust error debugging:

### 1. `rust.analyze_errors`
Analyzes Rust compiler output and provides detailed error diagnosis.

Input:
```json
{
  "compilerOutput": "string"
}
```

### 2. `rust.fix_code`
Fixes Rust code based on analyzed errors.

Input:
```json
{
  "code": "string",
  "errors": "array"
}
```

### 3. `rust.debug_runtime`
Analyzes potential runtime issues in Rust code.

Input:
```json
{
  "code": "string"
}
```

### 4. `rust.full_analysis`
Performs complete analysis including error diagnosis, fixes, and improvements.

Input:
```json
{
  "code": "string",
  "compilerOutput": "string"
}
```

## Error Categories

The server recognizes and handles the following Rust error categories:

1. **Borrow Checker**: Issues related to Rust's ownership system
2. **Lifetime**: Lifetime annotation and management issues
3. **Trait Bounds**: Missing or incorrect trait implementations
4. **Type Mismatch**: Type conversion and compatibility issues
5. **Undefined**: Missing imports, types, or identifiers
6. **Unused**: Unused imports, variables, or functions
7. **Syntax**: Syntax errors and malformed code
8. **Generic Constraints**: Issues with generic type parameters

## Response Format

All responses follow a standardized format:

### ✅ Step 1 — Error Diagnosis
- Root cause identification
- Explanation of why Rust is complaining

### ✅ Step 2 — Correct Fix
- Specific changes needed
- Explanation of the fix

### ✅ Step 3 — Fixed Full Code
- Complete, working Rust code with all necessary imports

### ✅ Step 4 — Optional Enhancements
- Idiomatic improvements
- Performance tips
- Safety recommendations

## Integration

This server can be integrated with any MCP-compatible client, including:

- VS Code with MCP extensions
- Codex/Qoder IDE
- Custom debugging tools
- CI/CD pipelines

## Limitations

- Some complex errors may require manual intervention
- Automated fixes are best-effort and may not cover all edge cases
- Runtime debugging is limited to static analysis

## Contributing

Contributions are welcome! Please submit issues and pull requests to improve error detection and fixing capabilities.