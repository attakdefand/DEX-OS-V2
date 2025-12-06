const { spawn } = require('child_process');
const path = require('path');

// Function to send JSON-RPC request
function sendRequest(child, method, params = {}) {
  const request = {
    jsonrpc: "2.0",
    id: Date.now(), // Use timestamp for unique ID
    method: method,
    params: params
  };
  console.log("📤 Sending request:", JSON.stringify(request, null, 2));
  child.stdin.write(JSON.stringify(request) + '\n');
}

console.log("🔬 Detailed MCP Server Test");
console.log("========================");

// Spawn the MCP server from the correct location
const serverPath = path.join(__dirname, 'mcp-server-gold-diamond', 'dist', 'server.js');
const server = spawn('node', [serverPath], {
  stdio: ['pipe', 'pipe', 'pipe']
});

let testPhase = 0; // 0 = initial, 1 = tools listed, 2 = summary requested, 3 = completed

// Handle server output
server.stdout.on('data', (data) => {
  const output = data.toString();
  console.log("📥 Raw server output:", JSON.stringify(output));
  
  // Try to parse JSON responses
  const lines = output.split('\n');
  for (const line of lines) {
    if (line.trim()) {
      try {
        const response = JSON.parse(line);
        console.log("📥 Parsed response:", JSON.stringify(response, null, 2));
        
        // If this is a tool list response
        if (response.result && response.result.tools) {
          console.log("✅ Connected to MCP server");
          console.log("🔍 Available tools:");
          response.result.tools.forEach(tool => {
            console.log(`   • ${tool.name} - ${tool.description}`);
          });
          
          if (testPhase === 0) {
            testPhase = 1;
            console.log("\n🔍 Requesting summary statistics...");
            sendRequest(server, 'tools/call', {
              name: 'protection.get_summary',
              arguments: {}
            });
          }
        }
        
        // If we get a tool call response
        if (response.result && response.method === 'tools/call') {
          console.log("✅ Received tool call response");
          if (response.result.content && response.result.content[0]) {
            console.log("📊 Response content:");
            console.log(response.result.content[0].text);
          } else {
            console.log("⚠️  No content in response");
            console.log(JSON.stringify(response.result, null, 2));
          }
          testPhase = 3;
          server.kill();
          process.exit(0);
        }
        
        // Handle errors
        if (response.error) {
          console.log("❌ Server error response:");
          console.log(JSON.stringify(response.error, null, 2));
          server.kill();
          process.exit(1);
        }
      } catch (e) {
        // Not JSON, just log it
        console.log('📝 Server message:', line);
      }
    }
  }
});

server.stderr.on('data', (data) => {
  console.error('🚨 Server stderr:', data.toString());
});

// Handle server close
server.on('close', (code) => {
  console.log(`🔚 Server process closed with code ${code}`);
});

// First, list tools to establish connection
setTimeout(() => {
  if (testPhase === 0) {
    console.log("📡 Connecting to MCP server...");
    sendRequest(server, 'tools/list');
  }
}, 1000);

// Timeout after 15 seconds
setTimeout(() => {
  if (testPhase !== 3) {
    console.log("⏰ Timeout reached");
    if (testPhase === 0) {
      console.log("   Server didn't respond to tools/list");
    } else if (testPhase === 1) {
      console.log("   Server didn't respond to protection.get_summary");
    }
    server.kill();
    process.exit(1);
  }
}, 15000);