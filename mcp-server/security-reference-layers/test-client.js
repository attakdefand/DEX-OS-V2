import { spawn } from 'child_process';
import { createInterface } from 'readline';

// Spawn the MCP server
const server = spawn('node', ['dist/server.js'], {
  stdio: ['pipe', 'pipe', 'pipe']
});

// Create readline interface for reading server responses
const rl = createInterface({
  input: server.stdout,
  output: server.stdin
});

// Track request IDs
let requestId = 1;

// Function to send JSON-RPC requests to the server
function sendRequest(method, params = {}) {
  const request = {
    jsonrpc: "2.0",
    id: requestId++,
    method: method,
    params: params
  };
  
  console.log(`Sending request: ${JSON.stringify(request)}`);
  server.stdin.write(JSON.stringify(request) + '\n');
}

// Listen for server responses
rl.on('line', (line) => {
  try {
    const response = JSON.parse(line);
    console.log(`Received response: ${JSON.stringify(response, null, 2)}`);
  } catch (error) {
    console.log(`Received non-JSON output: ${line}`);
  }
});

// Handle server errors
server.stderr.on('data', (data) => {
  console.error(`Server error: ${data}`);
});

// Handle server exit
server.on('close', (code) => {
  console.log(`Server exited with code ${code}`);
});

// Send initialization request
setTimeout(() => {
  sendRequest("tools/list", {});
}, 1000);

// Send a search request after a delay
setTimeout(() => {
  sendRequest("tools/call", {
    name: "security.search_tests",
    arguments: {
      query: "reentrancy"
    }
  });
}, 2000);

// Send a layer summary request
setTimeout(() => {
  sendRequest("tools/call", {
    name: "security.get_layer_summary",
    arguments: {}
  });
}, 3000);

// Send a list files request
setTimeout(() => {
  sendRequest("tools/call", {
    name: "security.list_test_files",
    arguments: {}
  });
}, 4000);

// Close the server after some time
setTimeout(() => {
  server.kill();
}, 5000);