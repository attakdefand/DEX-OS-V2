import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";

async function main() {
  console.log("🔗 Connecting to MCP Server Gold Diamond...");
  
  // Connect to the MCP server
  const transport = new StdioClientTransport({
    command: 'node',
    args: ['./dist/server.js']
  });

  const client = new Client({
    name: "gold-diamond-client",
    version: "1.0.0",
  });

  try {
    await client.connect(transport);
    console.log("✅ Connected to MCP Server Gold Diamond");
    
    // 1. Get summary of all test cases
    console.log("\n📊 Getting test summary...");
    const summaryResult = await client.request("tools/call", {
      name: 'protection.get_summary',
      arguments: {}
    });
    
    console.log('Summary:', JSON.stringify(summaryResult, null, 2));
    
    // 2. List all test files
    console.log("\n📋 Listing test files...");
    const listResult = await client.request("tools/call", {
      name: 'protection.list_test_files',
      arguments: {}
    });
    
    console.log('Files:', JSON.stringify(listResult, null, 2));
    
    // 3. Search for specific test cases (example: limiter tests)
    console.log("\n🔍 Searching for 'limiter' tests...");
    const searchResult = await client.request("tools/call", {
      name: 'protection.search_tests',
      arguments: {
        query: 'limiter'
      }
    });
    
    console.log('Search results:', JSON.stringify(searchResult, null, 2));
    
    // 4. Generate implementation plan for a specific test case
    // First, let's get a sample test case from the search results
    if (searchResult.content && searchResult.content[0]) {
      const searchResults = JSON.parse(searchResult.content[0].text);
      if (searchResults.results && searchResults.results.length > 0) {
        const sampleTestCase = searchResults.results[0];
        console.log("\n🔨 Generating implementation plan for sample test case...");
        
        const implementResult = await client.request("tools/call", {
          name: 'protection.implement_feature',
          arguments: {
            test_case: sampleTestCase
          }
        });
        
        console.log('Implementation plan:', JSON.stringify(implementResult, null, 2));
      }
    }
    
    console.log("\n✅ All operations completed successfully!");
    
  } catch (error) {
    console.error('❌ Error:', error);
  } finally {
    transport.close();
  }
}

main().catch(console.error);