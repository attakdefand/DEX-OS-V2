#!/usr/bin/env node

// Fix Type Mismatches for the CodeFix OS MCP Server
// Targets specifically the 3 type mismatch errors found in the DEX-OS-V2 project

const { spawn } = require('child_process');
const fs = require('fs').promises;
const path = require('path');

async function fixTypeMismatches() {
    console.log("=== CodeFix OS MCP Server - Fixing Type Mismatches ===\n");
    
    try {
        // Run cargo build to get compilation errors
        console.log("Running cargo build to capture type mismatch errors...");
        
        const cargoProcess = spawn('cargo', ['build', '--workspace'], {
            cwd: 'd:\\DEX-OS-V2\\DEX-OS-V2',
            stdio: ['pipe', 'pipe', 'pipe']
        });
        
        let stdout = '';
        let stderr = '';
        
        cargoProcess.stdout.on('data', (data) => {
            stdout += data.toString();
        });
        
        cargoProcess.stderr.on('data', (data) => {
            stderr += data.toString();
        });
        
        cargoProcess.on('close', async (code) => {
            console.log(`Cargo build completed with exit code ${code}\n`);
            
            // Combine stdout and stderr
            const compileOutput = stdout + stderr;
            
            // Save the output to a file for debugging
            await fs.writeFile('type-mismatch-output.txt', compileOutput);
            
            // Start the MCP server
            console.log("Starting MCP server...");
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

            // Parse the type mismatch errors
            const typeMismatchErrors = parseTypeMismatchErrors(compileOutput);
            console.log(`Found ${typeMismatchErrors.length} type mismatch errors in the compilation output\n`);
            
            if (typeMismatchErrors.length > 0) {
                console.log("Processing type mismatch errors through MCP server...\n");
                
                for (let i = 0; i < typeMismatchErrors.length; i++) {
                    const error = typeMismatchErrors[i];
                    console.log(`Processing type mismatch error ${i + 1}/${typeMismatchErrors.length}:`);
                    console.log(`  File: ${error.file || 'Unknown'}`);
                    console.log(`  Line: ${error.line || 'Unknown'}`);
                    console.log(`  Message: ${error.message.substring(0, 100)}${error.message.length > 100 ? '...' : ''}`);
                    
                    try {
                        // Send to MCP server for analysis and fix recommendation
                        const analysis = await sendRequest(server, {
                            jsonrpc: "2.0",
                            id: i + 1,
                            method: "tools/call",
                            params: {
                                name: "rust.fix_errors",
                                arguments: {
                                    code: "", // We don't have the specific code snippet, but the MCP server can still provide guidance
                                    error: error.message,
                                    filepath: error.file || ""
                                }
                            }
                        }, 15000); // 15 second timeout
                        
                        const result = JSON.parse(analysis.result.content[0].text);
                        console.log(`  MCP Fix Recommendation:`);
                        console.log(`    Root Cause: ${result.rootCause || 'Not specified'}`);
                        console.log(`    Fix Plan: ${result.fixPlan || 'Not specified'}`);
                        
                        if (result.patch && result.patch.length > 0) {
                            console.log(`    Suggested Patch:`);
                            result.patch.forEach((patch, idx) => {
                                console.log(`      ${idx + 1}. ${patch.action}: ${patch.description}`);
                            });
                        }
                    } catch (err) {
                        console.log(`  MCP Fix Recommendation failed: ${err.message}`);
                    }
                    
                    console.log();
                }
            } else {
                console.log("No type mismatch errors found in compilation output.");
            }
            
            // Close stdin to signal end of requests
            server.stdin.end();

            // Wait a bit for responses
            await new Promise(resolve => setTimeout(resolve, 2000));

            // Kill server
            server.kill();
            
            console.log("=== Fix Type Mismatches completed ===");
        });
    } catch (error) {
        console.log(`Error running cargo build: ${error.message}`);
    }
}

// Parse type mismatch errors from compilation output
function parseTypeMismatchErrors(output) {
    const errors = [];
    const lines = output.split('\n');
    
    let currentError = null;
    
    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        
        // Match type mismatch error lines
        const typeMismatchMatch = line.match(/error\[E0308\]: mismatched types/);
        if (typeMismatchMatch) {
            // Get the file and line information from the previous lines
            let file = "";
            let lineNum = "";
            
            // Look backwards for file info
            for (let j = i - 1; j >= Math.max(0, i - 5); j--) {
                const prevLine = lines[j];
                const fileMatch = prevLine.match(/^ --> (.+):(\d+):(\d+)/);
                if (fileMatch) {
                    file = fileMatch[1];
                    lineNum = fileMatch[2];
                    break;
                }
            }
            
            // Collect the error message
            let errorMessage = "mismatched types";
            let collecting = true;
            
            // Collect subsequent lines that are part of this error
            for (let j = i + 1; j < lines.length && collecting; j++) {
                const nextLine = lines[j];
                if (nextLine.startsWith("error[") || nextLine.startsWith("warning:")) {
                    collecting = false;
                } else if (nextLine.trim() !== "") {
                    errorMessage += " " + nextLine.trim();
                }
            }
            
            errors.push({
                type: "type_mismatch",
                code: "E0308",
                file: file,
                line: lineNum,
                message: errorMessage
            });
        }
    }
    
    return errors;
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

fixTypeMismatches().catch(console.error);