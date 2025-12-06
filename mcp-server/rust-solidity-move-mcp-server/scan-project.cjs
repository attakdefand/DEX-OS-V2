#!/usr/bin/env node

// Scan Project for the CodeFix OS MCP Server
// Scans the DEX-OS-V2 project for errors in Rust, Solidity, and Move code

const { spawn } = require('child_process');
const fs = require('fs').promises;
const path = require('path');

async function scanProject() {
    console.log("=== CodeFix OS MCP Server Project Scan ===\n");
    
    // Start the MCP server
    const server = spawn('node', ['dist/server.js'], {
        stdio: ['pipe', 'pipe', 'pipe']
    });

    // Handle server output
    server.stdout.on('data', (data) => {
        // We'll process responses in our request functions
    });

    server.stderr.on('data', (data) => {
        console.log('Server stderr:', data.toString());
    });

    server.on('close', (code) => {
        console.log(`Server exited with code ${code}`);
    });

    // Wait a moment for server to start
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Get list of Rust files in the project
    console.log("Scanning for Rust files...");
    const rustFiles = await findFiles('d:\\DEX-OS-V2\\DEX-OS-V2', '.rs');
    console.log(`Found ${rustFiles.length} Rust files\n`);

    // Track errors found
    let totalErrors = 0;
    let filesWithErrors = 0;

    // Analyze a sample of Rust files (first 5 for demo purposes)
    const sampleFiles = rustFiles.slice(0, 5);
    console.log(`Analyzing first ${sampleFiles.length} Rust files:\n`);

    for (const file of sampleFiles) {
        try {
            console.log(`Analyzing ${path.basename(file)}...`);
            const code = await fs.readFile(file, 'utf8');
            
            // Analyze the file
            const analysis = await sendRequest(server, {
                jsonrpc: "2.0",
                id: Date.now(), // Unique ID
                method: "tools/call",
                params: {
                    name: "rust.analyze_errors",
                    arguments: {
                        code: code,
                        filePath: file
                    }
                }
            });
            
            const result = JSON.parse(analysis.result.content[0].text);
            if (result.errors && result.errors.length > 0) {
                console.log(`  Found ${result.errors.length} errors`);
                totalErrors += result.errors.length;
                filesWithErrors++;
                
                // Show first error for brevity
                if (result.errors.length > 0) {
                    const firstError = result.errors[0];
                    console.log(`    - ${firstError.type}: ${firstError.message.substring(0, 100)}...`);
                }
            } else {
                console.log(`  No errors found`);
            }
        } catch (error) {
            console.log(`  Error analyzing file: ${error.message}`);
        }
        
        // Small delay between requests
        await new Promise(resolve => setTimeout(resolve, 100));
    }

    console.log(`\n${"=".repeat(50)}`);
    console.log(`Scan Summary:`);
    console.log(`  Total files analyzed: ${sampleFiles.length}`);
    console.log(`  Files with errors: ${filesWithErrors}`);
    console.log(`  Total errors found: ${totalErrors}`);
    console.log(`${"=".repeat(50)}\n`);

    // Close stdin to signal end of requests
    server.stdin.end();

    // Wait a bit for responses
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Kill server
    server.kill();
    
    console.log("=== Project Scan completed ===");
}

// Helper function to find files with specific extension
async function findFiles(dir, extension) {
    let results = [];
    try {
        const files = await fs.readdir(dir);
        for (const file of files) {
            const filePath = path.join(dir, file);
            const stat = await fs.stat(filePath);
            
            if (stat.isDirectory()) {
                // Skip node_modules and target directories to avoid huge scans
                if (!file.includes('node_modules') && !file.includes('target')) {
                    const recursiveResults = await findFiles(filePath, extension);
                    results = results.concat(recursiveResults);
                }
            } else if (file.endsWith(extension)) {
                results.push(filePath);
            }
        }
    } catch (error) {
        console.log(`Error reading directory ${dir}: ${error.message}`);
    }
    return results;
}

// Helper function to send requests and get responses
function sendRequest(server, request) {
    return new Promise((resolve, reject) => {
        // Set up a timeout
        const timeout = setTimeout(() => {
            reject(new Error('Request timeout'));
        }, 10000);
        
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

scanProject().catch(console.error);