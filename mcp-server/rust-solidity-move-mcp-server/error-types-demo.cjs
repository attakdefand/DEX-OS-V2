#!/usr/bin/env node

// Error Types Demo for the CodeFix OS MCP Server
// Demonstrates detection of all error types in Rust, Solidity, and Move

const { spawn } = require('child_process');

async function runErrorTypesDemo() {
    console.log("=== CodeFix OS MCP Server Error Types Demo ===\n");
    
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

    // Test Rust error types
    console.log("1. Rust Error Types Detection:");
    
    // Sample Rust code with various error types
    const rustCodeWithErrors = `
    use std::str::FromStr; // Unused import
    
    fn main() {
        let mut s = String::from("hello");
        let r1 = &s;  // immutable borrow
        let r2 = &s;  // immutable borrow
        s.push_str(", world!");  // mutable borrow - ERROR!
        println!("{} and {}", r1, r2);
        
        let x: i32 = "invalid"; // Type mismatch
        
        undefined_function(); // Undefined function
    }
    `;
    
    const rustAnalysis = await sendRequest(server, {
        jsonrpc: "2.0",
        id: 1,
        method: "tools/call",
        params: {
            name: "rust.analyze_errors",
            arguments: {
                code: rustCodeWithErrors
            }
        }
    });
    
    const rustResult = JSON.parse(rustAnalysis.result.content[0].text);
    console.log(`Found ${rustResult.errors.length} Rust errors:`);
    rustResult.errors.forEach((error, index) => {
        console.log(`  ${index + 1}. ${error.type}: ${error.message.substring(0, 60)}... (${error.category})`);
    });
    
    console.log("\n" + "=".repeat(50) + "\n");

    // Test Solidity error types
    console.log("2. Solidity Error Types Detection:");
    
    // Sample Solidity code with various error types
    const solidityCodeWithErrors = `
    pragma solidity ^0.8.0;

    contract TestContract {
        mapping(address => uint256) public balances;
        
        function withdraw() public {
            uint256 balance = balances[msg.sender];
            require(balance > 0, "Insufficient balance");
            
            // Vulnerable to reentrancy attack
            (bool sent, ) = msg.sender.call{value: balance}("");
            require(sent, "Failed to send Ether");
            
            balances[msg.sender] = 0;
        }
        
        function undefinedFunction() public { // Visibility not declared
            undefined_var = 5; // Undefined variable
        }
    }
    `;
    
    const solidityAnalysis = await sendRequest(server, {
        jsonrpc: "2.0",
        id: 2,
        method: "tools/call",
        params: {
            name: "sol.analyze",
            arguments: {
                code: solidityCodeWithErrors
            }
        }
    });
    
    const solidityResult = JSON.parse(solidityAnalysis.result.content[0].text);
    console.log(`Found ${solidityResult.errors.length} Solidity errors/issues:`);
    solidityResult.errors.forEach((error, index) => {
        console.log(`  ${index + 1}. ${error.type}: ${error.message.substring(0, 60)}... (${error.category})`);
    });
    
    console.log("\n" + "=".repeat(50) + "\n");

    // Test Move error types
    console.log("3. Move Error Types Detection:");
    
    // Sample Move code with various error types
    const moveCodeWithErrors = `
    module TestModule {
        struct Resource has key {
            value: u64
        }
        
        public fun create_resource(): Resource {
            Resource { value: 10 }  // Missing ability annotation
        }
        
        public fun undefined_function() {
            let x = undefined_var; // Undefined variable
        }
    }
    `;
    
    const moveAnalysis = await sendRequest(server, {
        jsonrpc: "2.0",
        id: 3,
        method: "tools/call",
        params: {
            name: "move.analyze",
            arguments: {
                code: moveCodeWithErrors
            }
        }
    });
    
    const moveResult = JSON.parse(moveAnalysis.result.content[0].text);
    console.log(`Found ${moveResult.errors.length} Move errors/issues:`);
    moveResult.errors.forEach((error, index) => {
        console.log(`  ${index + 1}. ${error.type}: ${error.message.substring(0, 60)}... (${error.category})`);
    });
    
    console.log("\n" + "=".repeat(50) + "\n");
    
    // Summary of error types detected
    console.log("Error Types Detected Summary:");
    console.log("Rust:");
    console.log("  - Borrow checker violations");
    console.log("  - Type mismatches");
    console.log("  - Undefined functions");
    console.log("  - Unused imports");
    
    console.log("Solidity:");
    console.log("  - Reentrancy vulnerabilities");
    console.log("  - Visibility issues");
    console.log("  - Undefined variables");
    
    console.log("Move:");
    console.log("  - Ability constraint violations");
    console.log("  - Undefined variables");
    
    // Close stdin to signal end of requests
    server.stdin.end();

    // Wait a bit for responses
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Kill server
    server.kill();
    
    console.log("\n=== Error Types Demo completed ===");
}

// Helper function to send requests and get responses
function sendRequest(server, request) {
    return new Promise((resolve, reject) => {
        // Set up a timeout
        const timeout = setTimeout(() => {
            reject(new Error('Request timeout'));
        }, 10000);
        
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

runErrorTypesDemo().catch(console.error);