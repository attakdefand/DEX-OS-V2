#!/usr/bin/env node

// Parse Compile Errors for the CodeFix OS MCP Server
// Captures compilation errors and processes them through the MCP server

const { spawn } = require('child_process');
const fs = require('fs').promises;
const path = require('path');

async function parseCompileErrors() {
    console.log("=== CodeFix OS MCP Server - Parsing Compile Errors ===\n");
    
    try {
        // Run cargo build to get compilation errors
        console.log("Running cargo build to capture compilation errors...");
        
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
            await fs.writeFile('compile-output.txt', compileOutput);
            
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

            // Parse the errors
            const errors = parseRustErrors(compileOutput);
            console.log(`Found ${errors.length} errors in the compilation output\n`);
            
            if (errors.length > 0) {
                console.log("Processing errors through MCP server...\n");
                
                // Process first 10 errors to get a better sample
                const errorsToProcess = errors.slice(0, 10);
                
                for (let i = 0; i < errorsToProcess.length; i++) {
                    const error = errorsToProcess[i];
                    console.log(`Processing error ${i + 1}/${errorsToProcess.length}:`);
                    console.log(`  Type: ${error.type}`);
                    console.log(`  Code: ${error.code || 'N/A'}`);
                    console.log(`  Message: ${error.message.substring(0, 80)}${error.message.length > 80 ? '...' : ''}`);
                    console.log(`  Category: ${error.category}`);
                    
                    try {
                        // Send to MCP server for analysis
                        const analysis = await sendRequest(server, {
                            jsonrpc: "2.0",
                            id: i + 1,
                            method: "tools/call",
                            params: {
                                name: "rust.analyze_errors",
                                arguments: {
                                    code: "", // We don't have the specific code snippet, but the MCP server can still categorize
                                    error: error.message
                                }
                            }
                        }, 10000); // 10 second timeout
                        
                        const result = JSON.parse(analysis.result.content[0].text);
                        console.log(`  MCP Analysis: ${result.errors.length} categorized errors`);
                        
                        if (result.errors.length > 0) {
                            result.errors.forEach((err, idx) => {
                                console.log(`    - ${err.category}: ${err.message.substring(0, 60)}${err.message.length > 60 ? '...' : ''}`);
                            });
                        }
                    } catch (err) {
                        console.log(`  MCP Analysis failed: ${err.message}`);
                    }
                    
                    console.log();
                }
                
                // Show a summary of all errors
                console.log("Error Summary by Category:");
                const categoryCount = {};
                errors.forEach(error => {
                    categoryCount[error.category] = (categoryCount[error.category] || 0) + 1;
                });
                
                Object.keys(categoryCount).sort().forEach(category => {
                    console.log(`  ${category}: ${categoryCount[category]}`);
                });
                
                // Show top error categories
                console.log("\nTop Error Categories:");
                const sortedCategories = Object.keys(categoryCount).sort((a, b) => categoryCount[b] - categoryCount[a]);
                sortedCategories.slice(0, 5).forEach(category => {
                    console.log(`  ${category}: ${categoryCount[category]}`);
                });
            } else {
                console.log("No errors found in compilation output.");
            }
            
            // Also check for Solidity files and errors if they exist
            await checkSolidityErrors(server);
            
            // Also check for Move files and errors if they exist
            await checkMoveErrors(server);
            
            // Close stdin to signal end of requests
            server.stdin.end();

            // Wait a bit for responses
            await new Promise(resolve => setTimeout(resolve, 2000));

            // Kill server
            server.kill();
            
            console.log("=== Parse Compile Errors completed ===");
        });
    } catch (error) {
        console.log(`Error running cargo build: ${error.message}`);
    }
}

// Parse Rust compilation errors
function parseRustErrors(output) {
    const errors = [];
    const lines = output.split('\n');
    
    let currentError = null;
    let collectingErrorDetails = false;
    
    for (const line of lines) {
        // Match error lines
        const errorMatch = line.match(/error(\[E\d+\])?: (.+)/);
        if (errorMatch) {
            if (currentError) {
                errors.push(currentError);
            }
            currentError = {
                type: "error",
                code: errorMatch[1] || "E0000",
                message: errorMatch[2],
                category: categorizeRustError(errorMatch[1] || "", errorMatch[2])
            };
            collectingErrorDetails = true;
            continue;
        }
        
        // Match warning lines
        const warningMatch = line.match(/warning: (.+)/);
        if (warningMatch) {
            if (currentError) {
                errors.push(currentError);
            }
            currentError = {
                type: "warning",
                message: warningMatch[1],
                category: "warning"
            };
            collectingErrorDetails = true;
            continue;
        }
        
        // Stop collecting details when we hit a blank line or a new section
        if (collectingErrorDetails && line.trim() === "") {
            collectingErrorDetails = false;
        }
        
        // Add context to current error if we're collecting details
        if (currentError && collectingErrorDetails && line.trim() !== "" && !line.startsWith(" ")) {
            // Skip file paths and line numbers for simplicity
            if (!line.match(/^ -->/) && !line.match(/^\d+ |^\s*\^/)) {
                currentError.message += " " + line.trim();
            }
        }
    }
    
    // Don't forget the last error
    if (currentError) {
        errors.push(currentError);
    }
    
    return errors;
}

// Check for Solidity errors
async function checkSolidityErrors(server) {
    console.log("\nChecking for Solidity files and errors...");
    
    try {
        // Look for Solidity files
        const solFiles = await findFiles('d:\\DEX-OS-V2\\DEX-OS-V2', '.sol');
        if (solFiles.length > 0) {
            console.log(`Found ${solFiles.length} Solidity files. Checking for errors...`);
            
            // For demonstration, we'll simulate a Solidity error
            const mockSolError = "ParserError: Expected ';' but got '}'";
            console.log("  Simulating Solidity error analysis...");
            
            try {
                const analysis = await sendRequest(server, {
                    jsonrpc: "2.0",
                    id: 999,
                    method: "tools/call",
                    params: {
                        name: "sol.analyze",
                        arguments: {
                            code: "", // We don't have the specific code snippet
                            error: mockSolError
                        }
                    }
                }, 5000);
                
                const result = JSON.parse(analysis.result.content[0].text);
                console.log(`  Solidity MCP Analysis: ${result.errors.length} categorized errors`);
            } catch (err) {
                console.log(`  Solidity MCP Analysis failed: ${err.message}`);
            }
        } else {
            console.log("No Solidity files found.");
        }
    } catch (error) {
        console.log(`Error checking Solidity files: ${error.message}`);
    }
}

// Check for Move errors
async function checkMoveErrors(server) {
    console.log("\nChecking for Move files and errors...");
    
    try {
        // Look for Move files
        const moveFiles = await findFiles('d:\\DEX-OS-V2\\DEX-OS-V2', '.move');
        if (moveFiles.length > 0) {
            console.log(`Found ${moveFiles.length} Move files. Checking for errors...`);
            
            // For demonstration, we'll simulate a Move error
            const mockMoveError = "error: ability 'copy' is required but not satisfied";
            console.log("  Simulating Move error analysis...");
            
            try {
                const analysis = await sendRequest(server, {
                    jsonrpc: "2.0",
                    id: 998,
                    method: "tools/call",
                    params: {
                        name: "move.analyze",
                        arguments: {
                            code: "", // We don't have the specific code snippet
                            error: mockMoveError
                        }
                    }
                }, 5000);
                
                const result = JSON.parse(analysis.result.content[0].text);
                console.log(`  Move MCP Analysis: ${result.errors.length} categorized errors`);
            } catch (err) {
                console.log(`  Move MCP Analysis failed: ${err.message}`);
            }
        } else {
            console.log("No Move files found.");
        }
    } catch (error) {
        console.log(`Error checking Move files: ${error.message}`);
    }
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
        // Ignore errors when reading directories
    }
    return results;
}

// Categorize Rust errors (simplified version of what's in the server)
function categorizeRustError(code, message) {
    // Borrow checker violations
    if (code.includes("E0597") || code.includes("E0505") || 
        message.includes("borrow") || message.includes("mutable borrow") || 
        message.includes("immutable borrow")) return "borrow_checker";
    
    // Lifetime issues
    if (code.includes("E0599") || code.includes("E0609") || 
        message.includes("lifetime")) return "lifetime";
    
    // Trait bound mismatches
    if (code.includes("E0277") || code.includes("E0282") || code.includes("E0271") ||
        message.includes("trait bound") || message.includes("trait bounds") ||
        message.includes("the trait")) return "trait_bounds";
    
    // Type mismatches
    if (code.includes("E0308") || 
        message.includes("mismatched types") || message.includes("expected") ||
        (message.includes("expected") && message.includes("found"))) return "type_mismatch";
    
    // Undefined types
    if (code.includes("E0425") || code.includes("E0412") ||
        message.includes("not found in this scope") || message.includes("cannot find") ||
        message.includes("unresolved import")) return "undefined_type";
    
    // Unused imports/variables
    if (code.includes("E0432") || code.includes("E0433") || code.includes("E0444") ||
        message.includes("unused") || message.includes("never read")) return "unused";
    
    // Syntax errors
    if ((message.includes("expected") && (message.includes("{") || message.includes(";"))) ||
        message.includes("syntax error")) return "syntax";
    
    // Generic constraints
    if (code.includes("E0271")) return "generic_constraints";
    
    // Method not found errors
    if (code.includes("E0599") || message.includes("method named") || message.includes("no method named")) 
        return "undefined_method";
    
    return "other";
}

// Helper function to send requests and get responses
function sendRequest(server, request, timeoutMs = 10000) {
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

parseCompileErrors().catch(console.error);