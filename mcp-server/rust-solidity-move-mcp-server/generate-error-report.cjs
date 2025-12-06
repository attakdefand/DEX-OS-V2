#!/usr/bin/env node

// Generate Error Report for the CodeFix OS MCP Server
// Creates a detailed report showing exactly which files contain the type mismatch errors

const { spawn } = require('child_process');
const fs = require('fs').promises;
const path = require('path');

async function generateErrorReport() {
    console.log("=== CodeFix OS MCP Server - Generating Error Report ===\n");
    
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
            
            // Parse all errors with file information
            const errorsWithFiles = parseErrorsWithFiles(compileOutput);
            
            // Generate detailed report
            const report = generateDetailedReport(errorsWithFiles);
            
            // Save report to file
            await fs.writeFile('error-report.txt', report);
            console.log("Error report saved to error-report.txt\n");
            
            // Display summary
            console.log(report);
            
            console.log("=== Error Report Generation completed ===");
        });
    } catch (error) {
        console.log(`Error running cargo build: ${error.message}`);
    }
}

// Parse errors with file information
function parseErrorsWithFiles(output) {
    const errors = [];
    const lines = output.split('\n');
    
    let currentError = null;
    
    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        
        // Match error lines with file information
        const fileMatch = line.match(/^ --> (.+):(\d+):(\d+)/);
        if (fileMatch) {
            if (currentError) {
                errors.push(currentError);
            }
            
            currentError = {
                file: fileMatch[1],
                line: fileMatch[2],
                column: fileMatch[3],
                message: "",
                category: "unknown"
            };
            continue;
        }
        
        // Match error type
        const errorMatch = line.match(/error(\[E\d+\])?: (.+)/);
        if (errorMatch && currentError) {
            currentError.message = errorMatch[2];
            currentError.code = errorMatch[1] || "";
            currentError.category = categorizeError(currentError.code, currentError.message);
            continue;
        }
        
        // Match warning type
        const warningMatch = line.match(/warning: (.+)/);
        if (warningMatch && currentError) {
            currentError.message = warningMatch[1];
            currentError.category = "warning";
            continue;
        }
        
        // Add context to current error
        if (currentError && line.trim() !== "" && 
            !line.startsWith(" -->") && 
            !line.match(/^\d+ |^\s*\^/) &&
            !line.startsWith("error[") &&
            !line.startsWith("warning:")) {
            currentError.message += " " + line.trim();
        }
        
        // End current error when we hit a blank line or new error
        if (currentError && (line.trim() === "" || line.startsWith("error[") || line.startsWith("warning:"))) {
            if (currentError.message.trim() !== "") {
                errors.push(currentError);
                currentError = null;
            }
        }
    }
    
    // Don't forget the last error
    if (currentError && currentError.message.trim() !== "") {
        errors.push(currentError);
    }
    
    return errors;
}

// Categorize errors
function categorizeError(code, message) {
    // Type mismatches
    if (code.includes("E0308") || message.includes("mismatched types") || 
        (message.includes("expected") && message.includes("found"))) {
        return "type_mismatch";
    }
    
    // Borrow checker violations
    if (code.includes("E0597") || code.includes("E0505") || 
        message.includes("borrow") || message.includes("mutable borrow") || 
        message.includes("immutable borrow")) {
        return "borrow_checker";
    }
    
    // Lifetime issues
    if (code.includes("E0599") || code.includes("E0609") || 
        message.includes("lifetime")) {
        return "lifetime";
    }
    
    // Trait bound mismatches
    if (code.includes("E0277") || code.includes("E0282") || code.includes("E0271") ||
        message.includes("trait bound") || message.includes("trait bounds") ||
        message.includes("the trait")) {
        return "trait_bounds";
    }
    
    // Unused imports/variables
    if (message.includes("unused") || message.includes("never read")) {
        return "unused";
    }
    
    return "other";
}

// Generate detailed report
function generateDetailedReport(errors) {
    let report = "=== DEX-OS-V2 Error Report ===\n\n";
    
    // Summary
    report += "SUMMARY:\n";
    report += `  Total errors: ${errors.length}\n\n`;
    
    // Categorize errors
    const categoryCount = {};
    errors.forEach(error => {
        categoryCount[error.category] = (categoryCount[error.category] || 0) + 1;
    });
    
    report += "ERRORS BY CATEGORY:\n";
    Object.keys(categoryCount).sort().forEach(category => {
        report += `  ${category}: ${categoryCount[category]}\n`;
    });
    report += "\n";
    
    // Group errors by file
    const files = {};
    errors.forEach(error => {
        if (!files[error.file]) {
            files[error.file] = [];
        }
        files[error.file].push(error);
    });
    
    report += "ERRORS BY FILE:\n";
    Object.keys(files).sort().forEach(filePath => {
        const fileErrors = files[filePath];
        report += `\n${filePath} (${fileErrors.length} errors):\n`;
        
        // Group by category within each file
        const fileCategories = {};
        fileErrors.forEach(error => {
            if (!fileCategories[error.category]) {
                fileCategories[error.category] = [];
            }
            fileCategories[error.category].push(error);
        });
        
        Object.keys(fileCategories).sort().forEach(category => {
            const categoryErrors = fileCategories[category];
            report += `  ${category} (${categoryErrors.length}):\n`;
            categoryErrors.forEach(error => {
                report += `    Line ${error.line}: ${error.message.substring(0, 100)}${error.message.length > 100 ? '...' : ''}\n`;
            });
        });
    });
    
    // Focus on type mismatches (most critical)
    const typeMismatches = errors.filter(error => error.category === "type_mismatch");
    if (typeMismatches.length > 0) {
        report += "\n\n=== CRITICAL TYPE MISMATCH ERRORS ===\n";
        report += "These errors prevent successful compilation and should be fixed first.\n\n";
        
        typeMismatches.forEach((error, index) => {
            report += `${index + 1}. ${error.file}:${error.line}\n`;
            report += `   Message: ${error.message}\n`;
            report += `   Code: ${error.code || 'N/A'}\n\n`;
        });
    }
    
    return report;
}

generateErrorReport().catch(console.error);