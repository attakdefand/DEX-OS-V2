#!/usr/bin/env node

// Script to continuously implement all remaining unimplemented features

const { spawn } = require('child_process');
const path = require('path');

console.log('Continuously Implementing All Unimplemented Features');
console.log('==================================================');

// Function to send JSON-RPC request
function sendRequest(child, method, params = {}) {
  const request = {
    jsonrpc: "2.0",
    id: 1,
    method: method,
    params: params
  };
  child.stdin.write(JSON.stringify(request) + '\n');
}

// Spawn the MCP server
const server = spawn('node', [path.join(__dirname, 'dist', 'index.js')], {
  stdio: ['pipe', 'pipe', 'pipe']
});

let implementationRound = 0;

// Handle server output
server.stdout.on('data', (data) => {
  const output = data.toString();
  
  // Try to parse JSON responses
  const lines = output.split('\n');
  for (const line of lines) {
    if (line.trim()) {
      try {
        const response = JSON.parse(line);
        
        // If this is a tool list response, try to implement all features
        if (response.result && response.result.tools) {
          console.log('Available tools confirmed, sending implementation request...');
          
          // Send implement_all_unimplemented_features request
          setTimeout(() => {
            console.log(`\n--- Implementation Round ${++implementationRound} ---`);
            sendRequest(server, 'tools/call', {
              name: 'implement_all_unimplemented_features',
              arguments: {
                batch_size: 5 // Implement 5 features at a time
              }
            });
          }, 1000);
        }
        
        // If we get a result from our tool call, check if there are more features
        if (response.result && response.method !== 'tools/list') {
          const resultText = response.result.content ? response.result.content.map(item => item.text).join('') : '';
          console.log('Implementation result:', resultText);
          
          // Check if there are remaining features
          if (resultText.includes('Remaining unimplemented features: 0')) {
            console.log('\n🎉 All features implemented!');
            server.kill();
            return;
          } else if (resultText.includes('Remaining unimplemented features:')) {
            console.log('\nStill have unimplemented features, continuing...');
            // Wait a bit then send another request
            setTimeout(() => {
              console.log(`\n--- Implementation Round ${++implementationRound} ---`);
              sendRequest(server, 'tools/call', {
                name: 'implement_all_unimplemented_features',
                arguments: {
                  batch_size: 5 // Implement 5 features at a time
                }
              });
            }, 3000);
          } else {
            console.log('\nImplementation completed or no more features detected.');
            server.kill();
          }
        }
      } catch (e) {
        // Not JSON, just log it
        console.log('Server message:', line);
      }
    }
  }
});

server.stderr.on('data', (data) => {
  console.error('Server error:', data.toString());
});

server.on('close', (code) => {
  console.log(`\nServer exited with code ${code}`);
});

// First, list tools to establish connection
setTimeout(() => {
  console.log('Establishing connection with MCP server...');
  sendRequest(server, 'tools/list');
}, 1000);