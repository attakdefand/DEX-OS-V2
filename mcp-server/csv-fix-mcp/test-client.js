import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";

async function main() {
  // Connect to the CSV fix MCP server
  const transport = new StdioClientTransport({
    command: 'node',
    args: ['./server.js'],
    cwd: process.cwd()
  });

  const client = new Client({
    name: "csv-fix-client",
    version: "1.0.0",
  });

  try {
    await client.connect(transport);
    
    // Test 1: Analyze CSV errors
    console.log('🔍 Analyzing CSV errors...');
    const analyzeResult = await client.request("tools/call", {
      name: 'analyze_csv_errors',
      arguments: {
        csv_path: '../DEX-OS-V2/DEX-OS-V2.csv'
      }
    });
    
    console.log('Analysis result:', JSON.stringify(analyzeResult, null, 2));
    
    // Test 2: Validate CSV structure
    console.log('\n📋 Validating CSV structure...');
    const validateResult = await client.request("tools/call", {
      name: 'validate_csv_structure',
      arguments: {
        csv_path: '../DEX-OS-V2/DEX-OS-V2.csv'
      }
    });
    
    console.log('Validation result:', JSON.stringify(validateResult, null, 2));
    
    // Test 3: Fix CSV errors (with backup)
    console.log('\n🔧 Fixing CSV errors...');
    const fixResult = await client.request("tools/call", {
      name: 'fix_csv_errors',
      arguments: {
        csv_path: '../DEX-OS-V2/DEX-OS-V2.csv',
        backup: true
      }
    });
    
    console.log('Fix result:', JSON.stringify(fixResult, null, 2));
    
  } catch (error) {
    console.error('Error:', error);
  } finally {
    await client.close();
  }
}

main().catch(console.error);