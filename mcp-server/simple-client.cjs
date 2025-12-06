const { spawn } = require('child_process');
const path = require('path');

// Spawn the MCP server process
const server = spawn('node', [path.join(__dirname, 'dist', 'index.js')], {
  stdio: ['pipe', 'pipe', 'pipe']
});

let buffer = '';

// Handle server output
server.stdout.on('data', (data) => {
  buffer += data.toString();
  
  // Try to parse complete JSON responses
  let lines = buffer.split('\n');
  buffer = lines.pop(); // Keep incomplete line in buffer
  
  for (let line of lines) {
    try {
      if (line.trim()) {
        const response = JSON.parse(line);
        console.log('Server response:', JSON.stringify(response, null, 2));
        
        // If this is a tool list response, try to implement a feature
        if (response.tools) {
          console.log('Available tools:', response.tools.map(t => t.name));
          
          // Send implement_feature request
          setTimeout(() => {
            console.log('Sending implement_feature request...');
            sendRequest({
              method: "call_tool",
              params: {
                name: "implement_feature",
                arguments: {
                  priority: 4,
                  category: "Settlement & Consensus",
                  component: "Blockchain Integration",
                  feature: "Block Finality Oracles"
                }
              }
            });
          }, 500);
        }
      }
    } catch (e) {
      // Not a JSON response, might be log message
      console.log('Server log:', line);
    }
  }
});

server.stderr.on('data', (data) => {
  console.error('Server error:', data.toString());
});

server.on('close', (code) => {
  console.log(`Server exited with code ${code}`);
});

// Function to send requests to the server
function sendRequest(request) {
  server.stdin.write(JSON.stringify(request) + '\n');
}

// First, list available tools
setTimeout(() => {
  console.log('Listing available tools...');
  sendRequest({
    method: "list_tools"
  });
}, 500);

// Kill server after some time
setTimeout(() => {
  server.kill();
}, 5000);