import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";

async function checkGoldDiamondStatus() {
    console.log("🔍 Checking Gold Diamond Implementation Status");
    console.log("==========================================");
    
    try {
        // Connect to the MCP server
        const transport = new StdioClientTransport({
            command: 'node',
            args: ['./dist/server.js']
        });

        const client = new Client({
            name: "gold-diamond-status-checker",
            version: "1.0.0",
        });

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
        
        // 3. Search for a few sample test cases to see if they have implementation status
        console.log("\n🔍 Checking sample protection test cases...");
        const sampleSearch = await client.request("tools/call", {
            name: 'protection.search_tests',
            arguments: {
                query: 'limiter'
            }
        });
        
        console.log('Sample protection tests:', JSON.stringify(sampleSearch, null, 2));
        
        console.log("\n🔍 Checking sample Web3 test cases...");
        const web3Search = await client.request("tools/call", {
            name: 'protection.search_tests',
            arguments: {
                query: 'web3'
            }
        });
        
        console.log('Sample Web3 tests:', JSON.stringify(web3Search, null, 2));
        
        console.log("\n✅ Status check completed!");
        
    } catch (error) {
        console.error('❌ Error:', error);
    }
}

console.log("🔍 Gold Diamond Status Checker");
console.log("============================");
checkGoldDiamondStatus().catch(console.error);