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

console.log("🧪 Testing MCP Server Connection");
console.log("==============================");

// Spawn the MCP server from the correct location
const serverPath = path.join(__dirname, 'mcp-server-gold-diamond', 'dist', 'server.js');
const server = spawn('node', [serverPath], {
  stdio: ['pipe', 'pipe', 'pipe']
});

let hasReceivedResponse = false;

// Handle server output
server.stdout.on('data', (data) => {
  const output = data.toString();
  
  // Try to parse JSON responses
  const lines = output.split('\n');
  for (const line of lines) {
    if (line.trim()) {
      try {
        const response = JSON.parse(line);
        
        // If this is a tool list response, call protection.get_summary
        if (response.result && response.result.tools) {
          console.log("✅ Connected to MCP server");
          console.log("🔍 Available tools:");
          response.result.tools.forEach(tool => {
            console.log(`   • ${tool.name} - ${tool.description}`);
          });
          
          console.log("\n🔍 Requesting summary statistics...");
          sendRequest(server, 'tools/call', {
            name: 'protection.get_summary'
          });
        }
        
        // If we get statistics results, display them
        if (response.result && response.method === 'tools/call' && response.params.name === 'protection.get_summary') {
          const resultText = response.result.content[0].text;
          console.log("📊 Feature Implementation Summary:");
          console.log("===============================");
          console.log(resultText);
          hasReceivedResponse = true;
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
  console.log("📡 Connecting to MCP server...");
  sendRequest(server, 'tools/list');
}, 1000);

// Timeout after 10 seconds
setTimeout(() => {
  if (!hasReceivedResponse) {
    console.log("⏰ Timeout: No response from server");
    server.kill();
    process.exit(1);
  }
}, 10000);