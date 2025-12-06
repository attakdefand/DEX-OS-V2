import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";

async function main() {
  // Connect to the MCP server
  const transport = new StdioClientTransport({
    command: 'node',
    args: ['./dist/server.js']
  });

  const client = new Client({
    name: "test-client",
    version: "1.0.0",
  });

  try {
    await client.connect(transport);
    
    // Test 1: Get summary
    console.log('🔍 Getting protection test summary...');
    const summaryResult = await client.request("tools/call", {
      name: 'protection.get_summary',
      arguments: {}
    });
    
    console.log('Summary result:', JSON.stringify(summaryResult, null, 2));
    
    // Test 2: List test files
    console.log('\n📋 Listing protection test files...');
    const listResult = await client.request("tools/call", {
      name: 'protection.list_test_files',
      arguments: {}
    });
    
    console.log('List result:', JSON.stringify(listResult, null, 2));
    
    // Test 3: Search tests
    console.log('\n🔍 Searching for "limiter" tests...');
    const searchResult = await client.request("tools/call", {
      name: 'protection.search_tests',
      arguments: {
        query: 'limiter'
      }
    });
    
    console.log('Search result:', JSON.stringify(searchResult, null, 2));
    
  } catch (error) {
    console.error('Error:', error);
  } finally {
    transport.close();
  }
}

main().catch(console.error);