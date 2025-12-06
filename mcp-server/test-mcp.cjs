#!/usr/bin/env node

// Simple test script to check MCP server functionality

const { spawn } = require('child_process');
const path = require('path');

console.log('Testing MCP Server Functionality');
console.log('==============================');

// Start the MCP server
const serverPath = path.join(__dirname, 'dist', 'index.js');
const serverProcess = spawn('node', [serverPath], {
  stdio: ['pipe', 'pipe', 'pipe']
});

console.log('Started MCP server');

let responseCount = 0;

// Handle server output
serverProcess.stdout.on('data', (data) => {
  const dataStr = data.toString();
  console.log('Server stdout:', dataStr);
  
  // Try to parse JSON responses
  const lines = dataStr.split('\n');
  for (const line of lines) {
    if (line.trim() === '') continue;
    
    try {
      const response = JSON.parse(line);
      responseCount++;
      
      if (response.id === 1 && response.result && response.result.tools) {
        console.log('Available tools:');
        response.result.tools.forEach(tool => {
          console.log(`  - ${tool.name}: ${tool.description}`);
        });
      }
      
      if (responseCount >= 2) {
        // Got both responses, close the connection
        setTimeout(() => {
          serverProcess.stdin.end();
          serverProcess.kill();
          console.log('Test completed');
        }, 1000);
      }
    } catch (err) {
      // Not JSON, might be log output
      console.log('Server log:', line);
    }
  }
});

serverProcess.stderr.on('data', (data) => {
  console.log('Server stderr:', data.toString());
});

// Wait for server to initialize, then send requests
setTimeout(() => {
  // Send list_tools request
  const listToolsRequest = {
    jsonrpc: "2.0",
    id: 1,
    method: "list_tools",
    params: {}
  };
  
  console.log('Sending list_tools request...');
  serverProcess.stdin.write(JSON.stringify(listToolsRequest) + '\n');
  
  // Send get_project_info request
  setTimeout(() => {
    const projectInfoRequest = {
      jsonrpc: "2.0",
      id: 2,
      method: "call_tool",
      params: {
        name: "get_project_info",
        arguments: {}
      }
    };
    
    console.log('Sending get_project_info request...');
    serverProcess.stdin.write(JSON.stringify(projectInfoRequest) + '\n');
  }, 1000);
}, 5000);

// Kill server after 15 seconds if it hasn't closed already
setTimeout(() => {
  serverProcess.kill();
  console.log('Test completed (timeout)');
}, 15000);