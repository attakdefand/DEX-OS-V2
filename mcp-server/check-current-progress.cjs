#!/usr/bin/env node

// Script to check current implementation progress

const { spawn } = require('child_process');
const path = require('path');

console.log('Checking Current Implementation Progress');
console.log('====================================');

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

// Handle server output
server.stdout.on('data', (data) => {
  const output = data.toString();
  
  // Try to parse JSON responses
  const lines = output.split('\n');
  for (const line of lines) {
    if (line.trim()) {
      try {
        const response = JSON.parse(line);
        
        // If this is a tool list response, get statistics
        if (response.result && response.result.tools) {
          console.log('✓ Connected to MCP server');
          console.log('→ Requesting feature statistics...');
          sendRequest(server, 'tools/call', {
            name: 'get_feature_statistics',
            arguments: {}
          });
        }
        
        // If we get statistics, display them
        if (response.result && response.method === 'tools/call' && response.result.content) {
          const resultText = response.result.content.map(item => item.text).join('');
          console.log('\n📊 CURRENT IMPLEMENTATION PROGRESS:');
          console.log('==================================');
          console.log(resultText);
          
          // Kill server after getting stats
          setTimeout(() => {
            server.kill();
          }, 1000);
        }
      } catch (e) {
        // Not JSON, just log it
        // console.log('Server message:', line);
      }
    }
  }
});

server.stderr.on('data', (data) => {
  console.error('Server error:', data.toString());
});

// First, list tools to establish connection
setTimeout(() => {
  console.log('→ Connecting to MCP server...');
  sendRequest(server, 'tools/list');
}, 1000);