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
        console.log('Server response:', JSON.parse(line));
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

// Test the implement_feature tool
setTimeout(() => {
  console.log('Testing implement_feature tool...');
  
  // Example request for an unimplemented feature
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
  
  // Give some time for response before exiting
  setTimeout(() => {
    server.kill();
  }, 2000);
}, 1000);