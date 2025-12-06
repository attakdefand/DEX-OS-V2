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
    name: "security-reference-layers-mcp-server",
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
 * Security Reference Layers Engine
 */
class SecurityReferenceLayersEngine {
  static async loadSecurityTests(filePath: string): Promise<any[]> {
    const results: any[] = [];
    
    return new Promise((resolve, reject) => {
      fs.createReadStream(filePath)
        .pipe(csv())
        .on('data', (data: any) => results.push(data))
        .on('end', () => resolve(results))
        .on('error', (error: any) => reject(error));
    });
  }
  
  static async getAllTestFiles(): Promise<string[]> {
    const files: string[] = [];
    
    try {
      // Get files from the main reference directory
      const mainReferencePath = path.join(PROJECT_ROOT, "..", "..", "DEX-OS-V2", ".reference");
      const mainFiles = await fsPromises.readdir(mainReferencePath);
      for (const file of mainFiles) {
        if (file.endsWith('.csv')) {
          files.push(path.join(mainReferencePath, file));
        }
      }
      
      // Get files from the gold layer directory
      const goldLayerPath = path.join(REFERENCE_LAYERS_PATH, "gold");
      const goldFiles = await fsPromises.readdir(goldLayerPath);
      for (const file of goldFiles) {
        if (file.endsWith('.csv')) {
          files.push(path.join(goldLayerPath, file));
        }
      }
      
      return files;
    } catch (error) {
      console.error("Error scanning reference files:", error);
      return [];
    }
  }
  
  static async searchSecurityTests(query: string, layer?: string): Promise<any[]> {
    try {
      const allFiles = await this.getAllTestFiles();
      let matchedTests: any[] = [];
      
      for (const file of allFiles) {
        // Filter by layer if specified
        if (layer && !file.includes(layer)) {
          continue;
        }
        
        const tests = await this.loadSecurityTests(file);
        
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
      console.error("Error searching security tests:", error);
      return [];
    }
  }
  
  static async getLayerSummary(): Promise<any> {
    try {
      const summary: any = {
        total_files: 0,
        layers: {}
      };
      
      // Get files from the main reference directory
      const mainReferencePath = path.join(PROJECT_ROOT, "..", "..", "DEX-OS-V2", ".reference");
      const mainFiles = await fsPromises.readdir(mainReferencePath);
      
      for (const file of mainFiles) {
        if (file.endsWith('.csv')) {
          summary.total_files++;
          if (!summary.layers['main']) {
            summary.layers['main'] = { files: [], test_count: 0 };
          }
          
          const filePath = path.join(mainReferencePath, file);
          const tests = await this.loadSecurityTests(filePath);
          summary.layers['main'].files.push(file);
          summary.layers['main'].test_count += tests.length;
        }
      }
      
      // Get files from the gold layer directory
      const goldLayerPath = path.join(REFERENCE_LAYERS_PATH, "gold");
      const goldFiles = await fsPromises.readdir(goldLayerPath);
      
      for (const file of goldFiles) {
        if (file.endsWith('.csv')) {
          summary.total_files++;
          if (!summary.layers['gold']) {
            summary.layers['gold'] = { files: [], test_count: 0 };
          }
          
          const filePath = path.join(goldLayerPath, file);
          const tests = await this.loadSecurityTests(filePath);
          summary.layers['gold'].files.push(file);
          summary.layers['gold'].test_count += tests.length;
        }
      }
      
      return summary;
    } catch (error) {
      console.error("Error getting layer summary:", error);
      return {};
    }
  }
}

// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: "security.search_tests",
        description: "Search security tests by keyword",
        inputSchema: {
          type: "object",
          properties: {
            query: { type: "string", description: "Search query" },
            layer: { type: "string", description: "Layer to search in (optional)" }
          },
          required: ["query"]
        },
      },
      {
        name: "security.get_layer_summary",
        description: "Get summary of all security reference layers",
        inputSchema: {
          type: "object",
          properties: {}
        },
      },
      {
        name: "security.list_test_files",
        description: "List all available security test files",
        inputSchema: {
          type: "object",
          properties: {}
        },
      }
    ],
  };
});

// Handle tool execution
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  try {
    switch (request.params.name) {
      case "security.search_tests": {
        const { query, layer } = request.params.arguments as { query: string; layer?: string };
        const results = await SecurityReferenceLayersEngine.searchSecurityTests(query, layer);
        
        return {
          content: [{
            type: "text",
            text: JSON.stringify({
              query,
              layer: layer || "all",
              results_count: results.length,
              results: results.slice(0, 50) // Limit results for readability
            }, null, 2)
          }]
        };
      }
      
      case "security.get_layer_summary": {
        const summary = await SecurityReferenceLayersEngine.getLayerSummary();
        
        return {
          content: [{
            type: "text",
            text: JSON.stringify(summary, null, 2)
          }]
        };
      }
      
      case "security.list_test_files": {
        const files = await SecurityReferenceLayersEngine.getAllTestFiles();
        
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
  console.error("Security Reference Layers MCP Server running on stdio");
}

main().catch((error) => {
  console.error("Server error:", error);
  process.exit(1);
});