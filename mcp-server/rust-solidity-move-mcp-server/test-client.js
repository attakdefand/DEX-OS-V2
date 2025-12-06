#!/usr/bin/env node

// Simple test client for the CodeFix OS MCP Server
const { spawn } = require('child_process');
const { once } = require('events');

async function runTest() {
    // Start the MCP server
    const server = spawn('node', ['dist/server.js'], {
        stdio: ['pipe', 'pipe', 'pipe']
    });

    // Handle server output
    server.stdout.on('data', (data) => {
        console.log('Server stdout:', data.toString());
    });

    server.stderr.on('data', (data) => {
        console.log('Server stderr:', data.toString());
    });

    server.on('close', (code) => {
        console.log(`Server exited with code ${code}`);
    });

    // Send JSON-RPC request for listing tools
    const listToolsRequest = {
        jsonrpc: "2.0",
        id: 1,
        method: "tools/list",
        params: {}
    };

    // Wait a moment for server to start
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Send request
    server.stdin.write(JSON.stringify(listToolsRequest) + '\n');

    // Send a sample Rust code analysis request
    const rustRequest = {
        jsonrpc: "2.0",
        id: 2,
        method: "tools/call",
        params: {
            name: "rust.analyze_errors",
            arguments: {
                code: "fn main() { let x = y; }"
            }
        }
    };

    server.stdin.write(JSON.stringify(rustRequest) + '\n');

    // Close stdin to signal end of requests
    server.stdin.end();

    // Wait a bit for responses
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Kill server
    server.kill();
}

runTest().catch(console.error);