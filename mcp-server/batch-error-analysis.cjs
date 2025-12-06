#!/usr/bin/env node

// Enhanced Project Analysis Script for DEX-OS-V2
// This script uses the existing MCP server to analyze the project status,
// check feature implementation, and identify potential issues

const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');

// Project root (assuming this script is in mcp-server directory)
const PROJECT_ROOT = path.resolve(__dirname, '..');
const MCP_SERVER_DIR = __dirname;

console.log('DEX-OS-V2 Project Analysis Script');
console.log('================================');
console.log(`Project root: ${PROJECT_ROOT}`);
console.log('');

// MCP Client to communicate with the server
class MCPClient {
  constructor(serverProcess) {
    this.serverProcess = serverProcess;
    this.requestId = 1;
    this.pendingRequests = new Map();
    
    // Set up message handling
    this.serverProcess.stdout.on('data', (data) => {
      this.handleServerResponse(data.toString());
    });
  }
  
  handleServerResponse(data) {
    const lines = data.split('\n');
    for (const line of lines) {
      if (line.trim() === '') continue;
      
      try {
        const response = JSON.parse(line);
        const requestId = response.id;
        
        if (this.pendingRequests.has(requestId)) {
          const resolver = this.pendingRequests.get(requestId);
          this.pendingRequests.delete(requestId);
          resolver(response);
        }
      } catch (err) {
        // Not a JSON response, might be log output
        console.log('Server output:', line);
      }
    }
  }
  
  sendRequest(method, params = {}) {
    return new Promise((resolve, reject) => {
      const requestId = this.requestId++;
      this.pendingRequests.set(requestId, resolve);
      
      const request = {
        jsonrpc: "2.0",
        id: requestId,
        method: method,
        params: params
      };
      
      this.serverProcess.stdin.write(JSON.stringify(request) + '\n');
      
      // Set timeout
      setTimeout(() => {
        if (this.pendingRequests.has(requestId)) {
          this.pendingRequests.delete(requestId);
          reject(new Error(`Request ${requestId} timed out`));
        }
      }, 15000); // 15 second timeout
    });
  }
  
  async listTools() {
    return this.sendRequest('tools/list');
  }
  
  async callTool(name, args) {
    return this.sendRequest('tools/call', {
      name: name,
      arguments: args
    });
  }
  
  // Get project information
  async getProjectInfo() {
    try {
      const response = await this.callTool('get_project_info', {});
      if (response.result && response.result.content) {
        return response.result.content.map(item => item.text).join('');
      }
      return 'No project info available';
    } catch (err) {
      return `Error getting project info: ${err.message}`;
    }
  }
  
  // Run cargo build to check for compilation errors
  async runBuild() {
    try {
      console.log('  Running cargo build...');
      const response = await this.callTool('run_build', {});
      if (response.result && response.result.content) {
        return response.result.content.map(item => item.text).join('');
      }
      return 'No build output available';
    } catch (err) {
      return `Error running build: ${err.message}`;
    }
  }
  
  // Run cargo tests
  async runTests() {
    try {
      console.log('  Running cargo tests...');
      const response = await this.callTool('run_tests', {});
      if (response.result && response.result.content) {
        return response.result.content.map(item => item.text).join('');
      }
      return 'No test output available';
    } catch (err) {
      return `Error running tests: ${err.message}`;
    }
  }
  
  // List components
  async listComponents() {
    try {
      const response = await this.callTool('list_components', {});
      if (response.result && response.result.content) {
        return response.result.content.map(item => item.text).join('');
      }
      return 'No components list available';
    } catch (err) {
      return `Error listing components: ${err.message}`;
    }
  }
  
  // Check feature status
  async checkFeatureStatus(priority = 0, category = "") {
    try {
      const response = await this.callTool('check_feature_status', {
        priority: priority,
        category: category
      });
      if (response.result && response.result.content) {
        return response.result.content.map(item => item.text).join('');
      }
      return 'No feature status available';
    } catch (err) {
      return `Error checking feature status: ${err.message}`;
    }
  }
  
  // Get feature statistics
  async getFeatureStatistics() {
    try {
      const response = await this.callTool('get_feature_statistics', {});
      if (response.result && response.result.content) {
        return response.result.content.map(item => item.text).join('');
      }
      return 'No statistics available';
    } catch (err) {
      return `Error getting feature statistics: ${err.message}`;
    }
  }
}

// Function to start the MCP server
function startMCPServer() {
  return new Promise((resolve, reject) => {
    console.log('Starting MCP server...');
    
    // Use the existing MCP server
    const serverPath = path.join(MCP_SERVER_DIR, 'dist', 'index.js');
    
    if (!fs.existsSync(serverPath)) {
      reject(new Error('MCP server not found. Please build the project first with `npx tsc`'));
      return;
    }
    
    const serverProcess = spawn('node', [serverPath], {
      stdio: ['pipe', 'pipe', 'pipe']
    });
    
    console.log('Started original MCP server');
    
    // Set up error handling
    serverProcess.on('error', (err) => {
      console.error('Failed to start MCP server:', err);
      reject(err);
    });
    
    // Wait a moment for the server to start
    setTimeout(() => {
      const client = new MCPClient(serverProcess);
      resolve({ process: serverProcess, client: client });
    }, 3000);
  });
}

// Function to stop the MCP server
function stopMCPServer(serverInfo) {
  if (serverInfo && serverInfo.process) {
    serverInfo.process.kill();
    console.log('Stopped MCP server');
  }
}

// Parse feature statistics to extract key metrics
function parseFeatureStatistics(statisticsText) {
  const metrics = {
    overallProgress: { implemented: 0, total: 0, percentage: 0 },
    priorityStats: {}
  };
  
  if (!statisticsText) return metrics;
  
  // Extract overall progress
  const progressMatch = statisticsText.match(/Overall Progress: (\d+)\/(\d+) \((\d+)%\)/);
  if (progressMatch) {
    metrics.overallProgress = {
      implemented: parseInt(progressMatch[1]),
      total: parseInt(progressMatch[2]),
      percentage: parseInt(progressMatch[3])
    };
  }
  
  // Extract priority stats
  for (let i = 1; i <= 5; i++) {
    const priorityMatch = statisticsText.match(new RegExp(`P${i}: (\\d+)\\/(\\d+) \\((\\d+)%\\)`));
    if (priorityMatch) {
      metrics.priorityStats[i] = {
        implemented: parseInt(priorityMatch[1]),
        total: parseInt(priorityMatch[2]),
        percentage: parseInt(priorityMatch[3])
      };
    }
  }
  
  return metrics;
}

// Analyze build output for errors
function analyzeBuildOutput(buildOutput) {
  const issues = {
    errors: [],
    warnings: []
  };
  
  if (!buildOutput) return issues;
  
  // Look for common error patterns
  if (buildOutput.includes('error:')) {
    const errorLines = buildOutput.split('\n').filter(line => line.includes('error:'));
    issues.errors = errorLines.slice(0, 5); // Limit to first 5 errors
  }
  
  if (buildOutput.includes('warning:')) {
    const warningLines = buildOutput.split('\n').filter(line => line.includes('warning:'));
    issues.warnings = warningLines.slice(0, 5); // Limit to first 5 warnings
  }
  
  return issues;
}

// Generate a comprehensive analysis report
function generateComprehensiveReport(analysisData) {
  console.log('\n=== COMPREHENSIVE PROJECT ANALYSIS ===\n');
  
  // Project Information
  console.log('PROJECT INFORMATION:');
  console.log('====================');
  if (analysisData.projectInfo && analysisData.projectInfo.length > 100) {
    console.log(analysisData.projectInfo.substring(0, 800) + '...\n');
  } else {
    console.log(analysisData.projectInfo || 'No project information available\n');
  }
  
  // Components
  console.log('PROJECT COMPONENTS:');
  console.log('===================');
  console.log(analysisData.components || 'No components information available\n');
  
  // Feature Statistics
  console.log('FEATURE IMPLEMENTATION PROGRESS:');
  console.log('================================');
  console.log(analysisData.statistics || 'No statistics available\n');
  
  // Parse metrics for detailed analysis
  const metrics = parseFeatureStatistics(analysisData.statistics);
  
  // Overall Progress Analysis
  console.log('PROGRESS ASSESSMENT:');
  console.log('====================');
  if (metrics.overallProgress.total > 0) {
    const { implemented, total, percentage } = metrics.overallProgress;
    console.log(`Overall: ${implemented}/${total} features implemented (${percentage}%)`);
    
    if (percentage < 30) {
      console.log('⚠️  CRITICAL: Very low implementation progress');
    } else if (percentage < 70) {
      console.log('⚠️  WARNING: Moderate implementation progress');
    } else {
      console.log('✅ GOOD: High implementation progress');
    }
  } else {
    console.log('No feature data available for progress assessment');
  }
  
  // Priority Analysis
  console.log('\nPER-PRIORITY ANALYSIS:');
  console.log('======================');
  for (let i = 1; i <= 5; i++) {
    if (metrics.priorityStats[i]) {
      const { implemented, total, percentage } = metrics.priorityStats[i];
      console.log(`Priority ${i}: ${implemented}/${total} (${percentage}%)`);
      
      if (total > 0 && percentage < 50) {
        console.log(`  ⚠️  WARNING: Low implementation for Priority ${i}`);
      }
    }
  }
  
  // Build Status
  console.log('\nBUILD STATUS:');
  console.log('=============');
  if (analysisData.buildOutput) {
    const buildIssues = analyzeBuildOutput(analysisData.buildOutput);
    
    if (buildIssues.errors.length > 0) {
      console.log('❌ BUILD ERRORS FOUND:');
      buildIssues.errors.forEach(error => console.log(`  - ${error}`));
    } else {
      console.log('✅ No build errors found');
    }
    
    if (buildIssues.warnings.length > 0) {
      console.log('\n⚠️  BUILD WARNINGS:');
      buildIssues.warnings.forEach(warning => console.log(`  - ${warning}`));
    }
  } else {
    console.log('No build data available');
  }
  
  // Test Status
  console.log('\nTEST STATUS:');
  console.log('============');
  if (analysisData.testOutput) {
    if (analysisData.testOutput.includes('FAILED')) {
      console.log('❌ TEST FAILURES DETECTED');
      // Extract failed tests
      const failedTests = analysisData.testOutput.split('\n').filter(line => line.includes('FAILED'));
      failedTests.slice(0, 5).forEach(test => console.log(`  - ${test}`));
      if (failedTests.length > 5) {
        console.log(`  ... and ${failedTests.length - 5} more failed tests`);
      }
    } else if (analysisData.testOutput.includes('test result: ok')) {
      console.log('✅ All tests passed');
    } else {
      console.log('Test results inconclusive');
    }
  } else {
    console.log('No test data available');
  }
  
  // Recommendations
  console.log('\nRECOMMENDATIONS:');
  console.log('================');
  
  // Based on progress
  if (metrics.overallProgress.percentage < 50) {
    console.log('1. Focus on implementing high-priority features first');
    console.log('2. Use the "implement_feature" tool to systematically implement missing features');
  } else if (metrics.overallProgress.percentage < 90) {
    console.log('1. Continue implementing remaining features');
    console.log('2. Consider using "implement_all_unimplemented_features" for batch implementation');
  } else {
    console.log('1. Project is nearly complete - focus on testing and refinement');
  }
  
  // Based on build status
  const buildIssues = analyzeBuildOutput(analysisData.buildOutput);
  if (buildIssues.errors.length > 0) {
    console.log('2. Address build errors before proceeding with feature implementation');
  }
  
  // Based on test status
  if (analysisData.testOutput && analysisData.testOutput.includes('FAILED')) {
    console.log('3. Investigate and fix failing tests');
  }
}

// Main analysis function
async function runProjectAnalysis() {
  try {
    console.log('Starting comprehensive project analysis...\n');
    
    // Start the MCP server
    let serverInfo = null;
    try {
      serverInfo = await startMCPServer();
      console.log('Connected to MCP server successfully\n');
    } catch (err) {
      console.log('Could not start/connect to MCP server:', err.message);
      console.log('Exiting analysis.');
      return;
    }
    
    // Collect all analysis data
    const analysisData = {
      projectInfo: '',
      components: '',
      statistics: '',
      featureStatus: '',
      buildOutput: '',
      testOutput: ''
    };
    
    console.log('Collecting project information...');
    analysisData.projectInfo = await serverInfo.client.getProjectInfo();
    
    console.log('Listing project components...');
    analysisData.components = await serverInfo.client.listComponents();
    
    console.log('Getting feature statistics...');
    analysisData.statistics = await serverInfo.client.getFeatureStatistics();
    
    console.log('Checking feature status...');
    analysisData.featureStatus = await serverInfo.client.checkFeatureStatus();
    
    console.log('Running build to check for errors...');
    analysisData.buildOutput = await serverInfo.client.runBuild();
    
    console.log('Running tests...');
    analysisData.testOutput = await serverInfo.client.runTests();
    
    // Stop the MCP server
    stopMCPServer(serverInfo);
    
    // Generate comprehensive report
    generateComprehensiveReport(analysisData);
    
    console.log('\n=== ANALYSIS COMPLETE ===');
    console.log('\nComprehensive project analysis completed using the MCP server.');
    console.log('The report above includes project status, feature progress, build health, and recommendations.');
    
  } catch (error) {
    console.error('Error during project analysis:', error);
    process.exit(1);
  }
}

// Run the analysis
runProjectAnalysis();