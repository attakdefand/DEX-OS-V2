#!/usr/bin/env node

// Test client for the Rust Debugging MCP Server
const { spawn } = require('child_process');
const { once } = require('events');

async function runTest() {
    // Spawn the MCP server
    const server = spawn('npm', ['run', 'dev-rust'], {
        stdio: ['pipe', 'pipe', 'pipe'],
        shell: true
    });

    // Handle server output
    server.stdout.on('data', (data) => {
        console.log('Server stdout:', data.toString());
    });

    server.stderr.on('data', (data) => {
        console.log('Server stderr:', data.toString());
    });

    // Wait for server to start
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Send initialization message
    const initMessage = {
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: {
            protocolVersion: "2024-01-01",
            capabilities: {},
            clientInfo: {
                name: "test-client",
                version: "1.0.0"
            }
        }
    };

    server.stdin.write(JSON.stringify(initMessage) + '\n');

    // Wait for response
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Request tools list
    const toolsMessage = {
        jsonrpc: "2.0",
        id: 2,
        method: "tools/list",
        params: {}
    };

    server.stdin.write(JSON.stringify(toolsMessage) + '\n');

    // Wait for response
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Test the rust.analyze_errors tool
    const testErrorOutput = `
error[E0308]: mismatched types
 --> src/main.rs:4:22
  |
4 | fn main() -> String {
  |              ------ expected \`String\` because of return type
5 |     println!("Hello, world!");
  |     ------------------------- expected this to be \`String\`, found \`()\`
  |
  = note: expected struct \`String\`
           found unit type \`()\`

error[E0597]: \`x\` does not live long enough
 --> src/main.rs:8:16
  |
7 |     let x = String::from("hello");
  |         - binding \`x\` declared here
8 |     let y = &x;
  |                ^ borrowed value does not live long enough
9 | }
  | - \`x\` dropped here while still borrowed
`;

    const analyzeMessage = {
        jsonrpc: "2.0",
        id: 3,
        method: "tools/call",
        params: {
            name: "rust.analyze_errors",
            arguments: {
                compilerOutput: testErrorOutput
            }
        }
    };

    server.stdin.write(JSON.stringify(analyzeMessage) + '\n');

    // Wait for response
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Close the server
    server.kill();
}

runTest().catch(console.error);