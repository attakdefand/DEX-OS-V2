const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// Test implementing a feature
console.log("Testing feature implementation...");

// Spawn the MCP server
const server = spawn('node', [path.join(__dirname, 'dist', 'index.js')], {
  stdio: ['pipe', 'pipe', 'pipe']
});

// Handle server output
server.stdout.on('data', (data) => {
  const output = data.toString();
  console.log('Server output:', output);
  
  // Try to parse JSON responses
  const lines = output.split('\n');
  for (const line of lines) {
    if (line.trim()) {
      try {
        const response = JSON.parse(line);
        if (response.result) {
          console.log('Success:', response.result);
          // If we get a response, exit
          server.kill();
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
  console.log(`Server process exited with code ${code}`);
});

// Wait a moment for server to start, then send request
setTimeout(() => {
  console.log('Sending implement_feature request...');
  
  // Send the correct MCP format
  const request = {
    method: "call_tool",
    params: {
      name: "implement_feature",
      arguments: {
        priority: 4,
        category: "Liquidity & Incentive",
        component: "Yield Farming/Staking",
        feature: "Staking Contracts"
      }
    }
  };
  
  server.stdin.write(JSON.stringify(request) + '\n');
}, 1000);

// Kill server after 5 seconds if it hasn't exited
setTimeout(() => {
  if (!server.killed) {
    console.log('Killing server process...');
    server.kill();
  }
}, 5000);