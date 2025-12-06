const { spawn } = require('child_process');
const path = require('path');

// Function to send JSON-RPC request
function sendRequest(child, method, params = {}) {
  const request = {
    jsonrpc: "2.0",
    id: Date.now(),
    method: method,
    params: params
  };
  console.log("Sending request:", JSON.stringify(request));
  child.stdin.write(JSON.stringify(request) + '\n');
}

console.log("Testing protection.get_summary tool");
console.log("==============================");

// Spawn the MCP server
const serverPath = path.join(__dirname, 'mcp-server-gold-diamond', 'dist', 'server.js');
const server = spawn('node', [serverPath], {
  stdio: ['pipe', 'pipe', 'pipe']
});

let receivedSummary = false;

// Handle server output
server.stdout.on('data', (data) => {
  const output = data.toString();
  console.log("Raw server output:", output);
  
  // Try to parse JSON responses
  const lines = output.split('\n');
  for (const line of lines) {
    if (line.trim()) {
      try {
        const response = JSON.parse(line);
        console.log("Parsed response:", JSON.stringify(response, null, 2));
        
        // If this is a tool list response, call protection.get_summary
        if (response.result && response.result.tools) {
          console.log("✓ Connected to MCP server");
          console.log("✓ Available tools detected");
          console.log("→ Calling protection.get_summary...");
          sendRequest(server, 'tools/call', {
            name: 'protection.get_summary',
            arguments: {}
          });
        }
        
        // If we get summary results, display them
        if (response.result && response.method === 'tools/call' && response.params.name === 'protection.get_summary') {
          const resultText = response.result.content[0].text;
          console.log("✓ Test summary received:");
          console.log("======================");
          console.log(resultText);
          console.log("======================");
          receivedSummary = true;
          server.kill();
          process.exit(0);
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

// First, list tools to establish connection
setTimeout(() => {
  console.log("→ Connecting to tools...");
  sendRequest(server, 'tools/list');
}, 1000);

// Timeout after 10 seconds
setTimeout(() => {
  if (!receivedSummary) {
    console.log("⏰ Timeout: No summary received");
    server.kill();
    process.exit(1);
  }
}, 10000);