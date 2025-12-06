#!/usr/bin/env node

// Demo script for the CodeFix OS MCP Server
// Demonstrates fixing Rust, Solidity, and Move code errors

const { spawn } = require('child_process');

async function runDemo() {
    console.log("=== CodeFix OS MCP Server Demo ===\n");
    
    // Start the MCP server
    const server = spawn('node', ['dist/server.js'], {
        stdio: ['pipe', 'pipe', 'pipe']
    });

    // Handle server output
    server.stdout.on('data', (data) => {
        // We'll process responses in our request functions
    });

    server.stderr.on('data', (data) => {
        console.log('Server stderr:', data.toString());
    });

    server.on('close', (code) => {
        console.log(`Server exited with code ${code}`);
    });

    // Wait a moment for server to start
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Test 1: List available tools
    console.log("1. Listing available tools:");
    const tools = await sendRequest(server, {
        jsonrpc: "2.0",
        id: 1,
        method: "tools/list",
        params: {}
    });
    
    console.log(`Found ${tools.result.tools.length} tools:\n`);
    tools.result.tools.forEach(tool => {
        console.log(`  - ${tool.name}: ${tool.description}`);
    });
    
    console.log("\n" + "=".repeat(50) + "\n");

    // Test 2: Analyze Rust code with errors
    console.log("2. Analyzing Rust code with errors:");
    const rustCode = `
fn main() {
    let mut x = 5;
    let y = &x;
    x += 1;  // This will cause a borrow checker error
    println!("{}", y);
}
`;
    
    const rustAnalysis = await sendRequest(server, {
        jsonrpc: "2.0",
        id: 2,
        method: "tools/call",
        params: {
            name: "rust.analyze_errors",
            arguments: {
                code: rustCode
            }
        }
    });
    
    console.log("Rust analysis result:");
    const rustResult = JSON.parse(rustAnalysis.result.content[0].text);
    console.log(`Success: ${rustResult.success}`);
    console.log(`Errors found: ${rustResult.errors.length}`);
    if (rustResult.errors.length > 0) {
        rustResult.errors.forEach((error, index) => {
            console.log(`  ${index + 1}. ${error.type}: ${error.message} (${error.category})`);
        });
    }
    
    console.log("\n" + "=".repeat(50) + "\n");

    // Test 3: Analyze Solidity code with errors
    console.log("3. Analyzing Solidity code with errors:");
    const solidityCode = `
pragma solidity ^0.8.0;

contract Test {
    function withdraw() public {
        msg.sender.transfer(address(this).balance);  // Potential reentrancy issue
    }
}
`;
    
    const solidityAnalysis = await sendRequest(server, {
        jsonrpc: "2.0",
        id: 3,
        method: "tools/call",
        params: {
            name: "sol.analyze",
            arguments: {
                code: solidityCode
            }
        }
    });
    
    console.log("Solidity analysis result:");
    const solidityResult = JSON.parse(solidityAnalysis.result.content[0].text);
    console.log(`Success: ${solidityResult.success}`);
    console.log(`Errors found: ${solidityResult.errors.length}`);
    if (solidityResult.errors.length > 0) {
        solidityResult.errors.forEach((error, index) => {
            console.log(`  ${index + 1}. ${error.type}: ${error.message} (${error.category})`);
        });
    }
    
    console.log("\n" + "=".repeat(50) + "\n");

    // Test 4: Analyze Move code with errors
    console.log("4. Analyzing Move code with errors:");
    const moveCode = `
module TestModule {
    struct Resource has key {
        value: u64
    }
    
    public fun create_resource(): Resource {
        Resource { value: 10 }  // Missing ability annotation
    }
}
`;
    
    const moveAnalysis = await sendRequest(server, {
        jsonrpc: "2.0",
        id: 4,
        method: "tools/call",
        params: {
            name: "move.analyze",
            arguments: {
                code: moveCode
            }
        }
    });
    
    console.log("Move analysis result:");
    const moveResult = JSON.parse(moveAnalysis.result.content[0].text);
    console.log(`Success: ${moveResult.success}`);
    console.log(`Errors found: ${moveResult.errors.length}`);
    if (moveResult.errors.length > 0) {
        moveResult.errors.forEach((error, index) => {
            console.log(`  ${index + 1}. ${error.type}: ${error.message} (${error.category})`);
        });
    }
    
    console.log("\n" + "=".repeat(50) + "\n");
    
    // Close stdin to signal end of requests
    server.stdin.end();

    // Wait a bit for responses
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Kill server
    server.kill();
    
    console.log("=== Demo completed ===");
}

// Helper function to send requests and get responses
function sendRequest(server, request) {
    return new Promise((resolve, reject) => {
        // Set up a timeout
        const timeout = setTimeout(() => {
            reject(new Error('Request timeout'));
        }, 5000);
        
        // Listen for the response
        const onResponse = (data) => {
            try {
                const response = JSON.parse(data.toString());
                if (response.id === request.id) {
                    clearTimeout(timeout);
                    server.stdout.removeListener('data', onResponse);
                    resolve(response);
                }
            } catch (e) {
                // Not a JSON response or not the one we're looking for
            }
        };
        
        server.stdout.on('data', onResponse);
        
        // Send the request
        server.stdin.write(JSON.stringify(request) + '\n');
    });
}

runDemo().catch(console.error);