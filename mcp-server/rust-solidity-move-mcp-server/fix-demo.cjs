#!/usr/bin/env node

// Fix Demo for the CodeFix OS MCP Server
// Demonstrates how to fix code errors using the MCP server

const { spawn } = require('child_process');

async function runFixDemo() {
    console.log("=== CodeFix OS MCP Server Fix Demo ===\n");
    
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

    // Demo 1: Fix Rust code with borrow checker error
    console.log("1. Fixing Rust code with borrow checker error:");
    const rustCodeWithError = `
fn main() {
    let mut s = String::from("hello");
    let r1 = &s;  // immutable borrow
    let r2 = &s;  // immutable borrow
    s.push_str(", world!");  // mutable borrow - ERROR!
    println!("{} and {}", r1, r2);
}
`;
    
    console.log("Original Rust code with error:");
    console.log(rustCodeWithError);
    
    // First analyze the errors
    const rustAnalysis = await sendRequest(server, {
        jsonrpc: "2.0",
        id: 1,
        method: "tools/call",
        params: {
            name: "rust.analyze_errors",
            arguments: {
                code: rustCodeWithError
            }
        }
    });
    
    const rustResult = JSON.parse(rustAnalysis.result.content[0].text);
    console.log(`\nAnalysis found ${rustResult.errors.length} errors:`);
    rustResult.errors.forEach((error, index) => {
        console.log(`  ${index + 1}. ${error.type}: ${error.message} (${error.category})`);
    });
    
    // Now fix the errors
    const rustFix = await sendRequest(server, {
        jsonrpc: "2.0",
        id: 2,
        method: "tools/call",
        params: {
            name: "rust.fix_errors",
            arguments: {
                code: rustCodeWithError,
                errors: rustResult.errors
            }
        }
    });
    
    console.log("\nFixed Rust code:");
    console.log(rustFix.result.content[0].text);
    
    console.log("\n" + "=".repeat(50) + "\n");

    // Demo 2: Fix Solidity code with reentrancy vulnerability
    console.log("2. Fixing Solidity code with reentrancy vulnerability:");
    const solidityCodeWithError = `
pragma solidity ^0.8.0;

contract VulnerableVault {
    mapping(address => uint256) public balances;
    
    function deposit() public payable {
        balances[msg.sender] += msg.value;
    }
    
    function withdraw() public {
        uint256 balance = balances[msg.sender];
        require(balance > 0, "Insufficient balance");
        
        // Vulnerable to reentrancy attack
        (bool sent, ) = msg.sender.call{value: balance}("");
        require(sent, "Failed to send Ether");
        
        balances[msg.sender] = 0;
    }
}
`;
    
    console.log("Original Solidity code with reentrancy vulnerability:");
    console.log(solidityCodeWithError);
    
    // First analyze the errors
    const solidityAnalysis = await sendRequest(server, {
        jsonrpc: "2.0",
        id: 3,
        method: "tools/call",
        params: {
            name: "sol.analyze",
            arguments: {
                code: solidityCodeWithError
            }
        }
    });
    
    const solidityResult = JSON.parse(solidityAnalysis.result.content[0].text);
    console.log(`\nAnalysis found ${solidityResult.errors.length} issues:`);
    solidityResult.errors.forEach((error, index) => {
        console.log(`  ${index + 1}. ${error.type}: ${error.message} (${error.category})`);
    });
    
    // Now fix the errors
    const solidityFix = await sendRequest(server, {
        jsonrpc: "2.0",
        id: 4,
        method: "tools/call",
        params: {
            name: "sol.fix",
            arguments: {
                code: solidityCodeWithError,
                errors: solidityResult.errors
            }
        }
    });
    
    console.log("\nFixed Solidity code:");
    console.log(solidityFix.result.content[0].text);
    
    console.log("\n" + "=".repeat(50) + "\n");
    
    // Close stdin to signal end of requests
    server.stdin.end();

    // Wait a bit for responses
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Kill server
    server.kill();
    
    console.log("=== Fix Demo completed ===");
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

runFixDemo().catch(console.error);