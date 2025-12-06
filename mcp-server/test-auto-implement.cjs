const { spawn } = require('child_process');
const path = require('path');

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

// Test implementing all unimplemented features
console.log("Testing automatic implementation of all unimplemented features...");

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
        console.log('Parsed response:', JSON.stringify(response, null, 2));
        
        // If this is a tool list response, try to implement all features
        if (response.result && response.result.tools) {
          console.log('Available tools:', response.result.tools.map(t => t.name));
          
          // Send implement_all_unimplemented_features request
          setTimeout(() => {
            console.log('Sending request to implement all unimplemented features...');
            sendRequest(server, 'tools/call', {
              name: 'implement_all_unimplemented_features',
              arguments: {
                batch_size: 1 // Implement 1 feature at a time for debugging
              }
            });
          }, 1000);
        }
        
        // If we get a result from our tool call, exit
        if (response.result && response.method !== 'tools/list') {
          console.log('Success:', response.result);
          // Exit after a short delay to see the output
          setTimeout(() => {
            server.kill();
          }, 2000);
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
  console.log(`Server exited with code ${code}`);
});

// First, list tools to establish connection
setTimeout(() => {
  console.log('Listing available tools...');
  sendRequest(server, 'tools/list');
}, 1000);