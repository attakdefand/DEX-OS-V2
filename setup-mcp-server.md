# Complete Walkthrough: Setting Up MCP Server for DEX-OS-V2

This document provides a comprehensive guide for setting up an MCP (Model Context Protocol) server to process security test cases from CSV files in the DEX-OS-V2 project.

## Overview

The MCP server acts as a bridge between security test catalogs (CSV files) and the DEX-OS codebase, automatically generating implementation plans and applying them to create security features.

## Prerequisites

1. Node.js (version 16 or higher)
2. npm (comes with Node.js)
3. TypeScript compiler
4. Git (optional, for version control)

## Directory Structure

Before starting, ensure you have the following directory structure:

```
DEX-OS-V2/
├── .reference/
│   └── layers/
│       └── gold/
│           ├── 1. protection_tests_full_with_all_metadata.csv
│           ├── 2. testing_web3_full_with_dsa_types.csv
│           ├── 3. protection_tests_full_with_dsa_types.csv
│           ├── 4. detection_response_tests_full_with_dsa_types.csv
│           ├── 5. resilience_recovery_with_dsa_types.csv
│           └── 6. governance_compliance_full_with_dsa_types.csv
└── mcp-server/
    ├── mcp-server-gold-diamond/
    └── (other files)
```

## Step-by-Step Setup

### 1. Create Project Directory

```bash
mkdir mcp-server-gold-diamond
cd mcp-server-gold-diamond
```

### 2. Initialize Node.js Project

```bash
npm init -y
```

### 3. Install Dependencies

```bash
# Core MCP SDK
npm install @modelcontextprotocol/sdk

# CSV parsing library
npm install csv-parser

# Validation library
npm install zod

# TypeScript support
npm install --save-dev typescript @types/node
```

### 4. Configure TypeScript

Create `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ES2020",
    "moduleResolution": "node",
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src/**/*"]
}
```

### 5. Create Server Implementation

Create `src/server.ts` with the following content:

```typescript
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import * as fs from "fs";
import * as fsPromises from "fs/promises";
import path from "path";
import csv from "csv-parser";

// Define the server
const server = new Server(
  {
    name: "mcp-server-gold-diamond",
    version: "1.0.0",
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

// Root directory for project files
const PROJECT_ROOT = process.cwd();
const REFERENCE_LAYERS_PATH = path.join(PROJECT_ROOT, "..", "..", "DEX-OS-V2", ".reference", "layers");

/**
 * Gold Diamond Protection Tests Engine
 */
class GoldDiamondProtectionEngine {
  static async loadProtectionTests(filePath: string): Promise<any[]> {
    const results: any[] = [];
    
    return new Promise((resolve, reject) => {
      fs.createReadStream(filePath)
        .pipe(csv())
        .on('data', (data: any) => results.push(data))
        .on('end', () => resolve(results))
        .on('error', (error: any) => reject(error));
    });
  }
  
  static async getAllProtectionTestFiles(): Promise<string[]> {
    const files: string[] = [];
    
    try {
      // Get files from the gold layer directory
      const goldLayerPath = path.join(REFERENCE_LAYERS_PATH, "gold");
      const goldFiles = await fsPromises.readdir(goldLayerPath);
      for (const file of goldFiles) {
        // Include all CSV files that contain relevant keywords in their name
        if (file.endsWith('.csv') && (file.includes('protection') || file.includes('web3') || file.includes('detection') || file.includes('resilience') || file.includes('governance') || file.includes('compliance'))) {
          files.push(path.join(goldLayerPath, file));
        }
      }
      
      return files;
    } catch (error) {
      console.error("Error scanning protection test files:", error);
      return [];
    }
  }  
  
  static async searchProtectionTests(query: string): Promise<any[]> {
    try {
      const allFiles = await this.getAllProtectionTestFiles();
      let matchedTests: any[] = [];
      
      for (const file of allFiles) {
        const tests = await this.loadProtectionTests(file);
        
        // Search for matching tests
        for (const test of tests) {
          // Check if query matches any field
          const testString = Object.values(test).join(' ').toLowerCase();
          if (testString.includes(query.toLowerCase())) {
            matchedTests.push({
              ...test,
              source_file: path.basename(file)
            });
          }
        }
      }
      
      return matchedTests;
    } catch (error) {
      console.error("Error searching protection tests:", error);
      return [];
    }
  }
  
  static async getProtectionTestSummary(): Promise<any> {
    try {
      const summary: any = {
        total_files: 0,
        total_tests: 0,
        files: []
      };
      
      // Get files from the gold layer directory
      const goldLayerPath = path.join(REFERENCE_LAYERS_PATH, "gold");
      const goldFiles = await fsPromises.readdir(goldLayerPath);
      
      for (const file of goldFiles) {
        // Include all CSV files that contain relevant keywords in their name
        if (file.endsWith('.csv') && (file.includes('protection') || file.includes('web3') || file.includes('detection') || file.includes('resilience') || file.includes('governance') || file.includes('compliance'))) {
          summary.total_files++;
          
          const filePath = path.join(goldLayerPath, file);
          const tests = await this.loadProtectionTests(filePath);
          summary.files.push({
            name: file,
            test_count: tests.length
          });
          summary.total_tests += tests.length;
        }
      }
      
      return summary;
    } catch (error) {
      console.error("Error getting protection test summary:", error);
      return {};
    }
  }

  /**
   * Implement a protection feature based on a test case
   */
  static async implementProtectionFeature(testCase: any): Promise<any> {
    try {
      // Extract key information from the test case
      const { layer, component, behavior, condition, test_name, owner, stack, tool, metric, sla, severity } = testCase;
      
      // Generate implementation plan
      const implementationPlan = {
        layer,
        component,
        behavior,
        condition,
        test_name,
        owner,
        technology_stack: stack,
        testing_tools: tool,
        success_metric: metric,
        sla_requirement: sla,
        severity,
        implementation_steps: [
          `1. Create security module for ${component} in the ${layer} layer`,
          `2. Implement ${behavior} behavior with ${condition} condition handling`,
          `3. Add appropriate input validation and sanitization`,
          `4. Implement access control enforcement mechanisms`,
          `5. Add monitoring and logging for ${metric} metrics`,
          `6. Create unit tests covering the ${condition} scenario`,
          `7. Integrate with existing security framework`,
          `8. Document the implementation and security considerations`
        ],
        suggested_code_structure: {
          module_name: `${component}_${behavior}_${condition}`.toLowerCase().replace(/[^a-z0-9_]/g, '_'),
          file_path: `dex-core/src/security/${layer.toLowerCase().replace(/[^a-z0-9_]/g, '_')}/${component.toLowerCase().replace(/[^a-z0-9_]/g, '_')}.rs`,
          test_file_path: `dex-core/tests/security_${component.toLowerCase().replace(/[^a-z0-9_]/g, '_')}_tests.rs`
        }
      };
      
      return implementationPlan;
    } catch (error) {
      console.error("Error implementing protection feature:", error);
      throw error;
    }
  }
}

// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: "protection.search_tests",
        description: "Search protection tests by keyword",
        inputSchema: {
          type: "object",
          properties: {
            query: { type: "string", description: "Search query" }
          },
          required: ["query"]
        },
      },
      {
        name: "protection.get_summary",
        description: "Get summary of all gold diamond protection tests",
        inputSchema: {
          type: "object",
          properties: {}
        },
      },
      {
        name: "protection.list_test_files",
        description: "List all available protection test files",
        inputSchema: {
          type: "object",
          properties: {}
        },
      },
      {
        name: "protection.implement_feature",
        description: "Generate implementation plan for a protection feature based on a test case",
        inputSchema: {
          type: "object",
          properties: {
            test_case: { 
              type: "object", 
              description: "The protection test case to implement" 
            }
          },
          required: ["test_case"]
        },
      }
    ],
  };
});

// Handle tool execution
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  try {
    switch (request.params.name) {
      case "protection.search_tests": {
        const { query } = request.params.arguments as { query: string };
        const results = await GoldDiamondProtectionEngine.searchProtectionTests(query);
        
        return {
          content: [{
            type: "text",
            text: JSON.stringify({
              query,
              results_count: results.length,
              results: results.slice(0, 50) // Limit results for readability
            }, null, 2)
          }]
        };
      }
      
      case "protection.get_summary": {
        const summary = await GoldDiamondProtectionEngine.getProtectionTestSummary();
        
        return {
          content: [{
            type: "text",
            text: JSON.stringify(summary, null, 2)
          }]
        };
      }
      
      case "protection.list_test_files": {
        const files = await GoldDiamondProtectionEngine.getAllProtectionTestFiles();
        
        return {
          content: [{
            type: "text",
            text: JSON.stringify({
              total_files: files.length,
              files: files.map(f => path.basename(f))
            }, null, 2)
          }]
        };
      }
      
      case "protection.implement_feature": {
        const { test_case } = request.params.arguments as { test_case: any };
        const implementationPlan = await GoldDiamondProtectionEngine.implementProtectionFeature(test_case);
        
        return {
          content: [{
            type: "text",
            text: JSON.stringify(implementationPlan, null, 2)
          }]
        };
      }
      
      default:
        return {
          content: [{ type: "text", text: `Unknown tool: ${request.params.name}` }],
          isError: true
        };
    }
  } catch (error) {
    return {
      content: [{
        type: "text",
        text: `Error executing tool ${request.params.name}: ${error instanceof Error ? error.message : String(error)}`
      }],
      isError: true
    };
  }
});

// Start the server
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("Gold Diamond Protection Tests MCP Server running on stdio");
}

main().catch((error) => {
  console.error("Server error:", error);
  process.exit(1);
});
```

### 6. Update package.json Scripts

Add the following scripts to your `package.json`:

```json
{
  "name": "mcp-server-gold-diamond",
  "version": "1.0.0",
  "description": "MCP Server for DEX-OS-V2 Gold Diamond Protection Tests",
  "main": "dist/server.js",
  "type": "module",
  "scripts": {
    "build": "tsc",
    "start": "node dist/server.js",
    "dev": "tsc && node dist/server.js"
  },
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.0.0",
    "zod": "^3.22.0",
    "csv-parser": "^3.0.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "typescript": "^5.0.0"
  }
}
```

### 7. Build the Server

```bash
npm run build
```

### 8. Start the Server

```bash
npm start
```

## Creating Client Applications

To interact with your MCP server, create a client application:

```javascript
const { spawn } = require('child_process');
const path = require('path');

// Function to send JSON-RPC request
function sendRequest(child, method, params = {}) {
  const request = {
    jsonrpc: "2.0",
    id: 1,
    method: method,
    params: params
  };
  child.stdin.write(JSON.stringify(request) + '\n');
}

// Spawn the MCP server
const server = spawn('node', [path.join(__dirname, 'dist', 'server.js')], {
  stdio: ['pipe', 'pipe', 'pipe']
});

// Handle server output
server.stdout.on('data', (data) => {
  const output = data.toString();
  
  // Try to parse JSON responses
  const lines = output.split('\n');
  for (const line of lines) {
    if (line.trim()) {
      try {
        const response = JSON.parse(line);
        
        // If this is a tool list response, call a tool
        if (response.result && response.result.tools) {
          console.log('Connected to MCP server');
          // Example: Get summary of protection tests
          sendRequest(server, 'tools/call', {
            name: 'protection.get_summary',
            arguments: {}
          });
        }
        
        // If we get tool results, display them
        if (response.result && response.result.content) {
          console.log('Tool response:');
          console.log(response.result.content[0].text);
          server.kill(); // Close after first response
        }
      } catch (e) {
        // Not JSON, just log it
        console.log('Server message:', line);
      }
    }
  }
});

server.stderr.on('data', (data) => {
  console.error('Server error:', data.toString());
});

// First, list tools to establish connection
setTimeout(() => {
  sendRequest(server, 'tools/list');
}, 1000);
```

## Continuous Implementation Process

To automate the implementation of features from CSV files:

1. Create a continuous implementation script that:
   - Spawns the MCP server
   - Connects to it
   - Calls tools to process test cases
   - Updates CSV files with implementation status

2. Run the script periodically or continuously to process all test cases.

## Monitoring Progress

Create a monitoring script to check implementation progress:

```bash
node implementation-progress-report.cjs
```

This will show:
- Total features implemented
- Remaining features to implement
- Progress percentage
- Details of recently implemented features

## Troubleshooting

### Common Issues

1. **Server won't start**: Check that all dependencies are installed correctly
2. **Tools not found**: Ensure the server is properly built with `npm run build`
3. **File path errors**: Verify the directory structure matches expectations
4. **Permission errors**: Ensure the server has read access to CSV files

### Debugging Tips

1. Use `console.log` statements in the server code to trace execution
2. Check that file paths are correct with `console.log(path.resolve(filePath))`
3. Verify CSV parsing works by testing with a small sample file
4. Use the MCP client example to test individual tools

## Extending the Server

To add new functionality:

1. Add new methods to the `GoldDiamondProtectionEngine` class
2. Register new tools in the `ListToolsRequestSchema` handler
3. Implement tool execution in the `CallToolRequestSchema` handler
4. Rebuild the server with `npm run build`

## Best Practices

1. **Modular Design**: Keep the engine class focused on specific responsibilities
2. **Error Handling**: Always wrap operations in try/catch blocks
3. **Logging**: Use informative console messages for debugging
4. **File Paths**: Use `path.join()` for cross-platform compatibility
5. **Resource Management**: Close file streams and child processes properly
6. **Performance**: Limit result sizes when returning large datasets
7. **Security**: Validate all inputs before processing

This walkthrough provides everything needed to set up and run an MCP server for processing security test cases in the DEX-OS-V2 project.