#!/usr/bin/env node

// Analyze lib.rs file for the CodeFix OS MCP Server
// Analyzes the DEX-OS-V2\dex-api\src\lib.rs file for errors

const { spawn } = require('child_process');
const fs = require('fs').promises;

async function analyzeLibRs() {
    console.log("=== CodeFix OS MCP Server - Analyzing lib.rs ===\n");
    
    // Read the lib.rs file
    const filePath = 'd:\\DEX-OS-V2\\DEX-OS-V2\\dex-api\\src\\lib.rs';
    try {
        const code = await fs.readFile(filePath, 'utf8');
        console.log(`Successfully read ${filePath}\n`);
        
        // Take just the first 500 lines to avoid timeout issues
        const lines = code.split('\n');
        const shortenedCode = lines.slice(0, 500).join('\n');
        console.log(`Analyzing first 500 lines (${shortenedCode.length} characters)\n`);
        
        // Start the MCP server
        const server = spawn('node', ['dist/server.js'], {
            stdio: ['pipe', 'pipe', 'pipe']
        });

        // Handle server output
        server.stdout.on('data', (data) => {
            // We'll process responses in our request functions
        });

        server.stderr.on('data', (data) => {
            //console.log('Server stderr:', data.toString());
        });

        server.on('close', (code) => {
            console.log(`Server exited with code ${code}`);
        });

        // Wait a moment for server to start
        await new Promise(resolve => setTimeout(resolve, 2000));

        // Analyze the Rust file
        console.log("Analyzing Rust code for errors...");
        const startTime = Date.now();
        const analysis = await sendRequest(server, {
            jsonrpc: "2.0",
            id: 1,
            method: "tools/call",
            params: {
                name: "rust.analyze_errors",
                arguments: {
                    code: shortenedCode,
                    filePath: filePath
                }
            }
        }, 15000); // 15 second timeout
        
        const endTime = Date.now();
        console.log(`Analysis completed in ${endTime - startTime}ms`);
        
        const result = JSON.parse(analysis.result.content[0].text);
        console.log(`\nAnalysis Results:`);
        console.log(`Success: ${result.success}`);
        console.log(`Errors found: ${result.errors.length}`);
        
        if (result.errors.length > 0) {
            console.log(`\nDetailed Errors:`);
            result.errors.forEach((error, index) => {
                console.log(`  ${index + 1}. ${error.type}: ${error.message.substring(0, 100)}... (${error.category})`);
            });
        } else {
            console.log(`\nNo errors found in the first 500 lines.`);
        }
        
        // Close stdin to signal end of requests
        server.stdin.end();

        // Wait a bit for responses
        await new Promise(resolve => setTimeout(resolve, 2000));

        // Kill server
        server.kill();
        
        console.log("\n=== Analysis completed ===");
    } catch (error) {
        console.log(`Error reading file or analyzing code: ${error.message}`);
    }
}

// Helper function to send requests and get responses
function sendRequest(server, request, timeoutMs = 15000) {
    return new Promise((resolve, reject) => {
        // Set up a timeout
        const timeout = setTimeout(() => {
            reject(new Error(`Request timeout after ${timeoutMs}ms`));
        }, timeoutMs);
        
        // Listen for the response
        const onResponse = (data) => {
            try {
                const response = JSON.parse(data.toString());
                if (response.id === request.id) {
                    clearTimeout(timeout);
                    server.stdout.removeListener('data', onResponse);
                    resolve(response);
                }
            } catch (e) {
                // Not a JSON response or not the one we're looking for
            }
        };
        
        server.stdout.on('data', onResponse);
        
        // Send the request
        server.stdin.write(JSON.stringify(request) + '\n');
    });
}

analyzeLibRs().catch(console.error);