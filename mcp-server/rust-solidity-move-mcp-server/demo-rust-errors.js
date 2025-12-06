#!/usr/bin/env node

// Demo script to compile a Rust file and analyze errors with the debugging server
const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

async function demoRustErrors() {
    console.log("Rust Error Debugging Demo");
    console.log("========================");
    
    // Path to the example Rust file
    const rustFilePath = path.join(__dirname, 'example-broken-code.rs');
    
    if (!fs.existsSync(rustFilePath)) {
        console.error("Example Rust file not found:", rustFilePath);
        return;
    }
    
    console.log("1. Reading example Rust code...");
    const rustCode = fs.readFileSync(rustFilePath, 'utf8');
    console.log("Example Rust code loaded.");
    
    console.log("\n2. Compiling Rust code to capture errors...");
    try {
        // Try to compile the Rust code and capture errors
        execSync(`rustc ${rustFilePath}`, { 
            stdio: ['pipe', 'pipe', 'pipe'],
            cwd: __dirname
        });
        console.log("Compilation succeeded (unexpected!)");
    } catch (compileError) {
        console.log("Compilation failed as expected. Capturing error output...");
        const compilerOutput = compileError.stderr.toString() || compileError.stdout.toString();
        console.log("Captured compiler output:");
        console.log(compilerOutput);
        
        // Now demonstrate how to send this to our debugging server
        console.log("\n3. To analyze these errors with the debugging server:");
        console.log("   - Start the server: npm run dev-rust");
        console.log("   - Send the compiler output using the MCP protocol");
        console.log("   - The server will analyze and provide fixes");
        
        // Show example MCP request
        console.log("\n4. Example MCP request to send to the server:");
        const mcpRequest = {
            jsonrpc: "2.0",
            id: 1,
            method: "tools/call",
            params: {
                name: "rust.full_analysis",
                arguments: {
                    code: rustCode,
                    compilerOutput: compilerOutput
                }
            }
        };
        
        console.log(JSON.stringify(mcpRequest, null, 2));
        
        console.log("\n5. The server would respond with:");
        console.log("   - Error diagnosis");
        console.log("   - Fix explanation");
        console.log("   - Fixed code");
        console.log("   - Improvement suggestions");
    }
}

demoRustErrors().catch(console.error);