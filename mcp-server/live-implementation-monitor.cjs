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

// Function to check implementation status continuously
async function checkLiveImplementation() {
  console.log("📊 Live Feature Implementation Status Monitor");
  console.log("==========================================");
  console.log("📡 Connecting to MCP server...");
  
  while (true) {
    try {
      // Spawn the MCP server from the correct location
      const serverPath = path.join(__dirname, 'mcp-server-gold-diamond', 'dist', 'server.js');
      const server = spawn('node', [serverPath], {
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
              
              // If this is a tool list response, call protection.get_summary
              if (response.result && response.result.tools) {
                console.log("✅ Connected to MCP server");
                console.log("🔍 Retrieving feature statistics...");
                sendRequest(server, 'tools/call', {
                  name: 'protection.get_summary'
                });
              }
              
              // If we get statistics results, display them
              if (response.result && response.method === 'tools/call') {
                const resultText = response.result.content[0].text;
                console.clear(); // Clear screen for live update
                console.log("📊 Live Feature Implementation Status Monitor");
                console.log("==========================================");
                console.log("📅 Last updated:", new Date().toLocaleString());
                console.log("");
                console.log("FEATURE IMPLEMENTATION PROGRESS:");
                console.log("===============================");
                console.log(resultText);
                console.log("");
                console.log("🔄 Next update in 30 seconds... (Press Ctrl+C to stop)");
                server.kill();
              }
            } catch (e) {
              // Not JSON, just log it
            }
          }
        }
      });

      server.stderr.on('data', (data) => {
        console.error('Server error:', data.toString());
      });

      // First, list tools to establish connection
      setTimeout(() => {
        sendRequest(server, 'tools/list');
      }, 1000);
      
      // Wait for this iteration to complete
      await new Promise(resolve => {
        server.on('close', resolve);
      });
      
      // Wait 30 seconds before next update
      await new Promise(resolve => setTimeout(resolve, 30000));
      
    } catch (error) {
      console.error('❌ Error checking status:', error);
      await new Promise(resolve => setTimeout(resolve, 10000));
    }
  }
}

// Run the live status monitor
checkLiveImplementation().catch(console.error);

// Handle Ctrl+C gracefully
process.on('SIGINT', function() {
  console.log('\n👋 Live status monitor stopped.');
  process.exit();
});