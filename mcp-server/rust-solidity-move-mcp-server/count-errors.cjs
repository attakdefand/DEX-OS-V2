#!/usr/bin/env node

// Count errors in the lib.rs file
// Parses compiler output to count errors

const { spawn } = require('child_process');
const fs = require('fs').promises;

async function countErrors() {
    console.log("=== Counting Errors in lib.rs ===\n");
    
    try {
        // Change to the DEX-OS-V2 directory and run cargo check
        console.log("Running cargo check on the DEX-OS-V2 project...");
        
        const child = spawn('cargo', ['check'], {
            cwd: 'd:\\DEX-OS-V2\\DEX-OS-V2',
            stdio: ['pipe', 'pipe', 'pipe']
        });
        
        let stdout = '';
        let stderr = '';
        
        child.stdout.on('data', (data) => {
            stdout += data.toString();
        });
        
        child.stderr.on('data', (data) => {
            stderr += data.toString();
        });
        
        child.on('close', (code) => {
            console.log(`Cargo check completed with exit code ${code}`);
            
            // Combine stdout and stderr
            const output = stdout + stderr;
            
            // Count errors
            const errorMatches = output.match(/error\[E\d+\]/g) || [];
            const errorCount = errorMatches.length;
            
            // Count warnings
            const warningMatches = output.match(/warning:/g) || [];
            const warningCount = warningMatches.length;
            
            console.log(`\nError Summary:`);
            console.log(`  Errors: ${errorCount}`);
            console.log(`  Warnings: ${warningCount}`);
            
            // Show some sample errors
            if (errorCount > 0) {
                console.log(`\nSample Errors:`);
                const errorLines = output.split('\n').filter(line => line.includes('error['));
                errorLines.slice(0, 5).forEach((line, index) => {
                    console.log(`  ${index + 1}. ${line.trim()}`);
                });
                if (errorLines.length > 5) {
                    console.log(`  ... and ${errorLines.length - 5} more errors`);
                }
            }
            
            console.log("\n=== Error counting completed ===");
        });
    } catch (error) {
        console.log(`Error running cargo check: ${error.message}`);
    }
}

countErrors().catch(console.error);