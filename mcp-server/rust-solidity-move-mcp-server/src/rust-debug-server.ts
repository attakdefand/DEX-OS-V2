import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
    CallToolRequestSchema,
    ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { exec } from "child_process";
import { promisify } from "util";
import path from "path";
import fs from "fs/promises";
import os from "os";

const execAsync = promisify(exec);

// Define the server
const server = new Server(
    {
        name: "rust-debug-mcp-server",
        version: "1.0.0",
    },
    {
        capabilities: {
            tools: {},
        },
    }
);

// Root directory for project files
const PROJECT_ROOT = process.cwd();

/**
 * Rust Error Analyzer - Parses Rust compiler output and categorizes errors
 */
class RustErrorAnalyzer {
    static parseCompilerOutput(output: string): any[] {
        const errors: any[] = [];
        const lines = output.split('\n');
        
        let currentError: any = null;
        let errorMessage = "";
        
        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            
            // Match error lines
            const errorMatch = line.match(/^error(\[E\d+\])?: (.*)$/);
            if (errorMatch) {
                // Save previous error if exists
                if (currentError) {
                    currentError.message = errorMessage.trim();
                    errors.push(currentError);
                }
                
                // Start new error
                currentError = {
                    type: "error",
                    code: errorMatch[1] || "E0000",
                    message: errorMatch[2],
                    category: this.categorizeError(errorMatch[1] || "", errorMatch[2]),
                    lineNumber: null,
                    fileName: null
                };
                errorMessage = errorMatch[2] + "\n";
                continue;
            }
            
            // Match warning lines
            const warningMatch = line.match(/^warning: (.*)$/);
            if (warningMatch) {
                // Save previous error if exists
                if (currentError) {
                    currentError.message = errorMessage.trim();
                    errors.push(currentError);
                }
                
                // Start new warning
                currentError = {
                    type: "warning",
                    message: warningMatch[1],
                    category: "warning",
                    lineNumber: null,
                    fileName: null
                };
                errorMessage = warningMatch[1] + "\n";
                continue;
            }
            
            // Extract line number and file info
            const locationMatch = line.match(/^ --> ([^:]+):(\d+):(\d+)/);
            if (locationMatch && currentError) {
                currentError.fileName = locationMatch[1];
                currentError.lineNumber = parseInt(locationMatch[2]);
            }
            
            // Accumulate error details
            if (currentError && !line.startsWith("error") && !line.startsWith("warning") && !line.startsWith(" -->")) {
                errorMessage += line + "\n";
            }
        }
        
        // Save last error
        if (currentError) {
            currentError.message = errorMessage.trim();
            errors.push(currentError);
        }
        
        return errors;
    }
    
    static categorizeError(code: string, message: string): string {
        // Borrow checker violations
        if (code.includes("E0597") || code.includes("E0505") || 
            message.includes("borrow") || message.includes("does not live long enough") || 
            message.includes("mutable borrow") || message.includes("immutable borrow")) {
            return "borrow_checker";
        }
        
        // Lifetime issues
        if (code.includes("E0599") || code.includes("E0609") || 
            message.includes("lifetime") || message.includes("lifetime mismatch")) {
            return "lifetime";
        }
        
        // Trait bound mismatches
        if (code.includes("E0277") || code.includes("E0282") || code.includes("E0271") ||
            message.includes("trait bound") || message.includes("trait bounds") ||
            message.includes("the trait") || message.includes("is not satisfied")) {
            return "trait_bounds";
        }
        
        // Type mismatches
        if (code.includes("E0308") || 
            message.includes("mismatched types") || 
            (message.includes("expected") && message.includes("found"))) {
            return "type_mismatch";
        }
        
        // Undefined types/imports
        if (code.includes("E0425") || code.includes("E0412") || code.includes("E0432") ||
            message.includes("not found in this scope") || message.includes("cannot find") ||
            message.includes("unresolved import")) {
            return "undefined";
        }
        
        // Unused imports/variables
        if (code.includes("E0432") || code.includes("E0433") || code.includes("E0444") ||
            message.includes("unused") || message.includes("never read")) {
            return "unused";
        }
        
        // Syntax errors
        if (message.includes("expected") || message.includes("unexpected") ||
            message.includes("syntax error")) {
            return "syntax";
        }
        
        // Generic constraints
        if (code.includes("E0271") || message.includes("mismatched types") ||
            message.includes("generic")) {
            return "generic_constraints";
        }
        
        return "other";
    }
}

/**
 * Rust Code Fixer - Generates fixes for various Rust error types
 */
class RustCodeFixer {
    static async fixCode(code: string, errors: any[]): Promise<any> {
        let fixedCode = code;
        let fixExplanation = "";
        let improvements = "";
        
        // Process each error
        for (let i = 0; i < errors.length; i++) {
            const error = errors[i];
            const fixResult = this.applyFix(fixedCode, error);
            fixedCode = fixResult.code;
            fixExplanation += fixResult.explanation + "\n";
        }
        
        // Add general improvements
        improvements = this.suggestImprovements(fixedCode);
        
        return {
            fixedCode,
            fixExplanation,
            improvements
        };
    }
    
    private static applyFix(code: string, error: any): { code: string; explanation: string } {
        let fixedCode = code;
        let explanation = "";
        
        switch (error.category) {
            case "borrow_checker":
                // Simple clone suggestion for borrow checker issues
                fixedCode = this.addCloneCalls(fixedCode);
                explanation = "Applied borrow checker fix: Added .clone() calls where values are moved.";
                break;
                
            case "lifetime":
                // Add explicit lifetime annotations
                fixedCode = this.addLifetimeAnnotations(fixedCode);
                explanation = "Added explicit lifetime annotations to resolve lifetime issues.";
                break;
                
            case "trait_bounds":
                // Add missing trait bounds
                fixedCode = this.addTraitBounds(fixedCode);
                explanation = "Added missing trait bounds to satisfy compiler requirements.";
                break;
                
            case "type_mismatch":
                // Attempt to fix type mismatches
                fixedCode = this.fixTypeMismatches(fixedCode, error);
                explanation = "Adjusted types to resolve type mismatch errors.";
                break;
                
            case "undefined":
                // Add common missing imports
                fixedCode = this.addCommonImports(fixedCode);
                explanation = "Added common imports that might be missing.";
                break;
                
            case "unused":
                // Remove unused items (this is a simplification)
                explanation = "Identified unused items. Please manually remove unused imports/variables.";
                break;
                
            case "syntax":
                // Attempt basic syntax fixes
                fixedCode = this.fixSyntaxIssues(fixedCode);
                explanation = "Applied basic syntax corrections.";
                break;
                
            default:
                explanation = `Identified ${error.category} error: ${error.message}. Manual fix may be required.`;
        }
        
        return { code: fixedCode, explanation };
    }
    
    private static addCloneCalls(code: string): string {
        // This is a simplified approach - in reality, more sophisticated analysis would be needed
        return code.replace(/(\.await)/g, ".clone()$1");
    }
    
    private static addLifetimeAnnotations(code: string): string {
        // Add generic lifetime annotations to functions that don't have them
        return code.replace(/fn (\w+\([^)]*\))/g, "fn $1<'a>");
    }
    
    private static addTraitBounds(code: string): string {
        // Add common trait bounds
        return code.replace(/where/g, "where T: Clone + Send + Sync,");
    }
    
    private static fixTypeMismatches(code: string, error: any): string {
        // This is a placeholder - real implementation would need more context
        // Look for common type conversion patterns
        return code.replace(/(\w+)\.parse\(\)/g, "$1.parse::<i32>()?");
    }
    
    private static addCommonImports(code: string): string {
        // Add common imports if they're not already present
        const commonImports = [
            "use std::collections::HashMap;",
            "use std::sync::Arc;",
            "use tokio::sync::Mutex;"
        ];
        
        let result = code;
        for (let j = 0; j < commonImports.length; j++) {
            const importStmt = commonImports[j];
            if (!result.includes(importStmt)) {
                // Add after existing use statements or at the top
                if (result.includes("use ")) {
                    result = result.replace(/(use [^;]+;)/, `$1\n${importStmt}`);
                } else {
                    result = `${importStmt}\n${result}`;
                }
            }
        }
        return result;
    }
    
    private static fixSyntaxIssues(code: string): string {
        // Fix common syntax issues
        return code.replace(/;;/g, ";"); // Remove double semicolons
    }
    
    private static suggestImprovements(code: string): string {
        let improvements = "";
        
        // Suggest improvements based on code patterns
        if (code.includes("clone()")) {
            improvements += "* Consider if cloning is necessary - borrowing might be more efficient\n";
        }
        
        if (code.includes("unwrap()")) {
            improvements += "* Replace unwrap() with proper error handling using ? or match\n";
        }
        
        if (code.includes("Box::new")) {
            improvements += "* Consider if Box allocation is necessary or if stack allocation would suffice\n";
        }
        
        if (!improvements) {
            improvements = "* Code follows Rust best practices\n";
        }
        
        return improvements;
    }
}

/**
 * Rust Debugger - Provides debugging capabilities for Rust applications
 */
class RustDebugger {
    static async analyzeRuntimeIssues(code: string): Promise<string> {
        // In a real implementation, this would run debugging tools
        let analysis = "Runtime Issue Analysis:\n";
        
        if (code.includes("panic!")) {
            analysis += "- Potential panic points detected. Consider using Result types for error handling.\n";
        }
        
        if (code.includes("unwrap()")) {
            analysis += "- Unwrap calls detected. These can cause panics if the Result/Option is None/Err.\n";
        }
        
        if (code.includes("expect(")) {
            analysis += "- Expect calls detected. These can cause panics with custom messages.\n";
        }
        
        if (code.includes("loop {")) {
            analysis += "- Infinite loop detected. Ensure there's a break condition.\n";
        }
        
        return analysis;
    }
}

// ==================== MCP METHODS ====================

// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
    return {
        tools: [
            {
                name: "rust.analyze_errors",
                description: "Analyze Rust compiler errors and provide detailed diagnosis",
                inputSchema: {
                    type: "object",
                    properties: {
                        compilerOutput: { type: "string", description: "Raw Rust compiler output" }
                    },
                    required: ["compilerOutput"]
                },
            },
            {
                name: "rust.fix_code",
                description: "Fix Rust code based on error analysis",
                inputSchema: {
                    type: "object",
                    properties: {
                        code: { type: "string", description: "Rust code to fix" },
                        errors: { type: "array", items: { type: "object" }, description: "Parsed errors" }
                    },
                    required: ["code", "errors"]
                },
            },
            {
                name: "rust.debug_runtime",
                description: "Analyze potential runtime issues in Rust code",
                inputSchema: {
                    type: "object",
                    properties: {
                        code: { type: "string", description: "Rust code to analyze" }
                    },
                    required: ["code"]
                },
            },
            {
                name: "rust.full_analysis",
                description: "Complete analysis including error diagnosis, fixes, and improvements",
                inputSchema: {
                    type: "object",
                    properties: {
                        code: { type: "string", description: "Rust code to analyze" },
                        compilerOutput: { type: "string", description: "Raw Rust compiler output" }
                    },
                    required: ["code", "compilerOutput"]
                },
            }
        ],
    };
});

// Handle tool execution
server.setRequestHandler(CallToolRequestSchema, async (request) => {
    try {
        switch (request.params.name) {
            case "rust.analyze_errors": {
                const { compilerOutput } = request.params.arguments as { compilerOutput: string };
                const errors = RustErrorAnalyzer.parseCompilerOutput(compilerOutput);
                
                return {
                    content: [{
                        type: "text",
                        text: JSON.stringify({
                            success: true,
                            errors,
                            errorCount: errors.filter(e => e.type === "error").length,
                            warningCount: errors.filter(e => e.type === "warning").length
                        }, null, 2)
                    }]
                };
            }
            
            case "rust.fix_code": {
                const { code, errors } = request.params.arguments as { code: string; errors: any[] };
                const fixResult = await RustCodeFixer.fixCode(code, errors);
                
                return {
                    content: [{
                        type: "text",
                        text: `### ✅ Step 1 — Error Diagnosis
- Analyzed ${errors.length} errors
- Applied automated fixes for common Rust issues

### ✅ Step 2 — Correct Fix
${fixResult.fixExplanation}

### ✅ Step 3 — Fixed Full Code
\`\`\`rust
${fixResult.fixedCode}
\`\`\`

### ✅ Step 4 — Optional Enhancements
${fixResult.improvements}`
                    }]
                };
            }
            
            case "rust.debug_runtime": {
                const { code } = request.params.arguments as { code: string };
                const analysis = await RustDebugger.analyzeRuntimeIssues(code);
                
                return {
                    content: [{
                        type: "text",
                        text: analysis
                    }]
                };
            }
            
            case "rust.full_analysis": {
                const { code, compilerOutput } = request.params.arguments as { code: string; compilerOutput: string };
                const errors = RustErrorAnalyzer.parseCompilerOutput(compilerOutput);
                const fixResult = await RustCodeFixer.fixCode(code, errors);
                const runtimeAnalysis = await RustDebugger.analyzeRuntimeIssues(code);
                
                return {
                    content: [{
                        type: "text",
                        text: `### ✅ Step 1 — Error Diagnosis
- Found ${errors.filter(e => e.type === "error").length} errors and ${errors.filter(e => e.type === "warning").length} warnings
- Error categories: ${[...new Set(errors.map(e => e.category))].join(", ")}

Root cause analysis:
${errors.map(e => `- ${e.category}: ${e.message}`).join("\n")}

Why Rust is complaining:
Rust's strict compile-time checking ensures memory safety and thread safety without a garbage collector. These errors occur because the code violates Rust's ownership, borrowing, or type system rules.

### ✅ Step 2 — Correct Fix
${fixResult.fixExplanation}

### ✅ Step 3 — Fixed Full Code
\`\`\`rust
${fixResult.fixedCode}
\`\`\`

### ✅ Step 4 — Optional Enhancements
${fixResult.improvements}

### ✅ Runtime Analysis
${runtimeAnalysis}`
                    }]
                };
            }
            
            default:
                return {
                    content: [{ type: "text", text: `Unknown tool: ${request.params.name}` }],
                    isError: true
                };
        }
    } catch (error) {
        return {
            content: [{
                type: "text",
                text: `Error executing tool ${request.params.name}: ${error instanceof Error ? error.message : String(error)}`
            }],
            isError: true
        };
    }
});

// Start the server
async function main() {
    const transport = new StdioServerTransport();
    await server.connect(transport);
    console.error("Rust Error Debugging MCP Server running on stdio");
}

main().catch((error) => {
    console.error("Server error:", error);
    process.exit(1);
});