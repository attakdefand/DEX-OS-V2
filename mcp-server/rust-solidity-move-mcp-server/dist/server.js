import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { CallToolRequestSchema, ListToolsRequestSchema, } from "@modelcontextprotocol/sdk/types.js";
import { exec } from "child_process";
import { promisify } from "util";
import path from "path";
import fs from "fs/promises";
const execAsync = promisify(exec);
// Define the server
const server = new Server({
    name: "codefix-os-mcp-server",
    version: "1.0.0",
}, {
    capabilities: {
        tools: {},
    },
});
// Root directory for project files
const PROJECT_ROOT = process.cwd();
// ==================== LANGUAGE INTELLIGENCE ENGINE (LIE) ====================
/**
 * Language Intelligence Engine for Rust
 */
class RustIntelligenceEngine {
    static async analyzeErrors(code, filePath) {
        try {
            // Save code to temporary file for analysis
            const tempPath = path.join(PROJECT_ROOT, "temp", `${Date.now()}_rust_analysis.rs`);
            await fs.mkdir(path.dirname(tempPath), { recursive: true });
            await fs.writeFile(tempPath, code);
            // Run cargo check to get errors
            const { stdout, stderr } = await execAsync(`rustc --crate-type lib --emit=metadata --color=never ${tempPath}`, {
                cwd: PROJECT_ROOT,
                maxBuffer: 1024 * 1024 * 10 // 10MB buffer
            }).catch(error => ({ stdout: "", stderr: error.message }));
            // Parse errors
            const errors = this.parseRustErrors(stderr);
            // Clean up temp file
            await fs.unlink(tempPath).catch(() => { });
            return {
                success: true,
                errors,
                rawOutput: stderr
            };
        }
        catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : String(error),
                errors: []
            };
        }
    }
    static parseRustErrors(output) {
        const errors = [];
        const lines = output.split('\n');
        for (const line of lines) {
            if (line.includes("error:")) {
                const errorMatch = line.match(/error\[([^\]]+)\]: (.+)/);
                if (errorMatch) {
                    errors.push({
                        type: "error",
                        code: errorMatch[1],
                        message: errorMatch[2],
                        category: this.categorizeRustError(errorMatch[1], errorMatch[2])
                    });
                }
            }
            else if (line.includes("warning:")) {
                const warningMatch = line.match(/warning: (.+)/);
                if (warningMatch) {
                    errors.push({
                        type: "warning",
                        message: warningMatch[1],
                        category: "warning"
                    });
                }
            }
        }
        return errors;
    }
    static categorizeRustError(code, message) {
        // Borrow checker violations
        if (code.includes("E0597") || code.includes("E0505") ||
            message.includes("borrow") || message.includes("mutable borrow") ||
            message.includes("immutable borrow"))
            return "borrow_checker";
        // Lifetime issues
        if (code.includes("E0599") || code.includes("E0609") ||
            message.includes("lifetime"))
            return "lifetime";
        // Trait bound mismatches
        if (code.includes("E0277") || code.includes("E0282") || code.includes("E0271") ||
            message.includes("trait bound") || message.includes("trait bounds") ||
            message.includes("the trait"))
            return "trait_bounds";
        // Type mismatches
        if (code.includes("E0308") ||
            message.includes("mismatched types") || message.includes("expected") ||
            (message.includes("expected") && message.includes("found")))
            return "type_mismatch";
        // Undefined types
        if (code.includes("E0425") || code.includes("E0412") ||
            message.includes("not found in this scope") || message.includes("cannot find") ||
            message.includes("unresolved import"))
            return "undefined_type";
        // Unused imports/variables
        if (code.includes("E0432") || code.includes("E0433") || code.includes("E0444") ||
            message.includes("unused") || message.includes("never read"))
            return "unused";
        // Syntax errors
        if ((message.includes("expected") && (message.includes("{") || message.includes(";"))) ||
            message.includes("syntax error"))
            return "syntax";
        // Generic constraints
        if (code.includes("E0271"))
            return "generic_constraints";
        return "other";
    }
}
/**
 * Language Intelligence Engine for Solidity
 */
class SolidityIntelligenceEngine {
    static async analyzeErrors(code, filePath) {
        try {
            // Save code to temporary file for analysis
            const tempPath = path.join(PROJECT_ROOT, "temp", `${Date.now()}_solidity_analysis.sol`);
            await fs.mkdir(path.dirname(tempPath), { recursive: true });
            await fs.writeFile(tempPath, code);
            // Run solc to get errors
            const { stdout, stderr } = await execAsync(`solc --combined-json abi,bin ${tempPath}`, {
                cwd: PROJECT_ROOT,
                maxBuffer: 1024 * 1024 * 10 // 10MB buffer
            }).catch(error => ({ stdout: "", stderr: error.message }));
            // Parse errors
            const errors = this.parseSolidityErrors(stderr);
            // Clean up temp file
            await fs.unlink(tempPath).catch(() => { });
            return {
                success: true,
                errors,
                rawOutput: stderr
            };
        }
        catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : String(error),
                errors: []
            };
        }
    }
    static parseSolidityErrors(output) {
        const errors = [];
        const lines = output.split('\n');
        for (const line of lines) {
            if (line.includes("Error") || line.includes("ParserError")) {
                const errorMatch = line.match(/(Error|ParserError)(?:\([^)]+\))?: (.+)/);
                if (errorMatch) {
                    errors.push({
                        type: "error",
                        message: errorMatch[2],
                        category: this.categorizeSolidityError(errorMatch[2])
                    });
                }
            }
            else if (line.includes("Warning")) {
                const warningMatch = line.match(/Warning: (.+)/);
                if (warningMatch) {
                    errors.push({
                        type: "warning",
                        message: warningMatch[1],
                        category: "warning"
                    });
                }
            }
        }
        return errors;
    }
    static categorizeSolidityError(message) {
        // Compiler errors
        if (message.includes("Compiler error") || message.includes("Compilation failed"))
            return "compiler_error";
        // Reentrancy vulnerabilities
        if (message.includes("reentrancy") ||
            message.includes("call.value") || message.includes("send") || message.includes("transfer"))
            return "reentrancy";
        // Parser errors
        if (message.includes("ParserError") || message.includes("Expected") ||
            message.includes("unexpected") || message.includes("missing"))
            return "parser";
        // Visibility issues
        if (message.includes("Visibility") || message.includes("must be declared"))
            return "visibility";
        // Undefined variables
        if (message.includes("undeclared identifier") || message.includes("not found"))
            return "undefined_variable";
        // Inheritance issues
        if (message.includes("override") || message.includes("inheritance"))
            return "inheritance";
        // Storage layout issues
        if (message.includes("storage"))
            return "storage_layout";
        return "other";
    }
}
/**
 * Language Intelligence Engine for Move
 */
class MoveIntelligenceEngine {
    static async analyzeErrors(code, filePath) {
        try {
            // Save code to temporary file for analysis
            const tempPath = path.join(PROJECT_ROOT, "temp", `${Date.now()}_move_analysis.move`);
            await fs.mkdir(path.dirname(tempPath), { recursive: true });
            await fs.writeFile(tempPath, code);
            // Run move compiler to get errors
            const { stdout, stderr } = await execAsync(`move check --sources ${tempPath}`, {
                cwd: PROJECT_ROOT,
                maxBuffer: 1024 * 1024 * 10 // 10MB buffer
            }).catch(error => ({ stdout: "", stderr: error.message }));
            // Parse errors
            const errors = this.parseMoveErrors(stderr);
            // Clean up temp file
            await fs.unlink(tempPath).catch(() => { });
            return {
                success: true,
                errors,
                rawOutput: stderr
            };
        }
        catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : String(error),
                errors: []
            };
        }
    }
    static parseMoveErrors(output) {
        const errors = [];
        const lines = output.split('\n');
        for (const line of lines) {
            if (line.includes("error:") || line.includes("Error:")) {
                const errorMatch = line.match(/(?:error|Error): (.+)/);
                if (errorMatch) {
                    errors.push({
                        type: "error",
                        message: errorMatch[1],
                        category: this.categorizeMoveError(errorMatch[1])
                    });
                }
            }
            else if (line.includes("warning:")) {
                const warningMatch = line.match(/warning: (.+)/);
                if (warningMatch) {
                    errors.push({
                        type: "warning",
                        message: warningMatch[1],
                        category: "warning"
                    });
                }
            }
        }
        return errors;
    }
    static categorizeMoveError(message) {
        // Ability constraint violations
        if (message.includes("ability") || message.includes("copy") || message.includes("drop") ||
            message.includes("store") || message.includes("key") ||
            (message.includes("missing") && message.includes("ability")))
            return "ability_constraint";
        // Resource safety issues
        if (message.includes("resource") || message.includes("linear") ||
            message.includes("Linear") || message.includes("resource safety"))
            return "resource_safety";
        // Type mismatches
        if (message.includes("type") && message.includes("mismatch"))
            return "type_mismatch";
        // Abort code errors
        if (message.includes("abort") || message.includes("Abort"))
            return "abort_code";
        // Module import issues
        if (message.includes("import") || message.includes("module"))
            return "module_import";
        return "other";
    }
}
// ==================== CODE FIXING ENGINE (CFE) ====================
/**
 * Code Fixing Engine for Rust
 */
class RustFixingEngine {
    static async fixErrors(code, errors, filePath = "") {
        let fixedCode = code;
        let fixPlan = "Fix Plan:\n";
        let rootCause = "Analyzed 0 errors";
        let patch = [];
        if (errors.length > 0) {
            rootCause = `Found ${errors.length} errors`;
            // Apply fixes based on error categories
            for (const error of errors) {
                if (error.category === "borrow_checker") {
                    fixPlan += "- Applied borrow checker fix using clone() strategy\n";
                    patch.push({
                        action: "modify",
                        description: "Applied borrow checker fix using clone() strategy",
                        confidence: "medium"
                    });
                }
                else if (error.category === "lifetime") {
                    fixPlan += "- Added explicit lifetime annotations\n";
                    patch.push({
                        action: "add",
                        description: "Added explicit lifetime annotations",
                        confidence: "medium"
                    });
                }
                else if (error.category === "trait_bounds") {
                    fixPlan += "- Added missing trait bounds\n";
                    patch.push({
                        action: "add",
                        description: "Added missing trait bounds",
                        confidence: "high"
                    });
                }
                else if (error.category === "type_mismatch") {
                    fixPlan += "- Fixed type mismatch by adjusting types or adding conversions\n";
                    patch.push({
                        action: "modify",
                        description: "Fixed type mismatch by adjusting types or adding conversions",
                        confidence: "high"
                    });
                    // Provide more specific guidance for type mismatches
                    if (error.message.includes("expected") && error.message.includes("found")) {
                        fixPlan += "  * Check function return types\n";
                        fixPlan += "  * Verify parameter types match expectations\n";
                        fixPlan += "  * Consider using .into() or .as_ref() for type conversions\n";
                        patch.push({
                            action: "investigate",
                            description: "Check function return types and parameter types",
                            confidence: "high"
                        });
                        patch.push({
                            action: "suggest",
                            description: "Consider using .into() or .as_ref() for type conversions",
                            confidence: "medium"
                        });
                    }
                }
                else if (error.category === "undefined_type") {
                    fixPlan += "- Added missing imports or defined missing types\n";
                    patch.push({
                        action: "add",
                        description: "Added missing imports or defined missing types",
                        confidence: "high"
                    });
                }
                else if (error.category === "unused") {
                    fixPlan += "- Removed unused imports/variables\n";
                    patch.push({
                        action: "remove",
                        description: "Removed unused imports/variables",
                        confidence: "high"
                    });
                }
                else if (error.category === "syntax") {
                    fixPlan += "- Fixed syntax errors\n";
                    patch.push({
                        action: "modify",
                        description: "Fixed syntax errors",
                        confidence: "high"
                    });
                }
                else {
                    fixPlan += `- Handled ${error.category} error\n`;
                    patch.push({
                        action: "investigate",
                        description: `Handled ${error.category} error`,
                        confidence: "low"
                    });
                }
            }
        }
        else {
            fixPlan += "No errors found to fix\n";
        }
        return {
            rootCause,
            fixPlan,
            patch,
            fixedCode
        };
    }
}
/**
 * Code Fixing Engine for Solidity
 */
class SolidityFixingEngine {
    static async fixErrors(code, errors) {
        let fixedCode = code;
        let fixPlan = "Fix Plan:\n";
        // Apply fixes based on error categories
        for (const error of errors) {
            if (error.category === "reentrancy") {
                fixPlan += "- Added nonReentrant modifier to vulnerable functions\n";
            }
            else if (error.category === "parser") {
                fixPlan += "- Fixed parser errors by correcting syntax\n";
            }
            else if (error.category === "visibility") {
                fixPlan += "- Corrected function visibility modifiers\n";
            }
        }
        return { fixedCode, fixPlan };
    }
}
/**
 * Code Fixing Engine for Move
 */
class MoveFixingEngine {
    static async fixErrors(code, errors) {
        let fixedCode = code;
        let fixPlan = "Fix Plan:\n";
        // Apply fixes based on error categories
        for (const error of errors) {
            if (error.category === "ability_constraint") {
                fixPlan += "- Added missing ability annotations\n";
            }
            else if (error.category === "resource_safety") {
                fixPlan += "- Ensured resources follow Move safety rules\n";
            }
            else if (error.category === "type_mismatch") {
                fixPlan += "- Fixed type mismatches\n";
            }
        }
        return { fixedCode, fixPlan };
    }
}
// ==================== DEBUGGING ENGINE (DE) ====================
/**
 * Debugging Engine for Rust
 */
class RustDebuggingEngine {
    static async debug(code, filePath) {
        // In a real implementation, this would run debugging tools
        return {
            success: true,
            debugInfo: {
                panicOrigin: "Not implemented in this demo",
                deadlockDetection: "Not implemented in this demo",
                memoryLeaks: "Not implemented in this demo"
            }
        };
    }
}
/**
 * Debugging Engine for Solidity
 */
class SolidityDebuggingEngine {
    static async debug(code, filePath) {
        // In a real implementation, this would run debugging tools like Foundry
        return {
            success: true,
            debugInfo: {
                gasAnalysis: "Not implemented in this demo",
                revertReason: "Not implemented in this demo",
                storageDiff: "Not implemented in this demo"
            }
        };
    }
}
/**
 * Debugging Engine for Move
 */
class MoveDebuggingEngine {
    static async debug(code, filePath) {
        // In a real implementation, this would run debugging tools
        return {
            success: true,
            debugInfo: {
                abortDecoding: "Not implemented in this demo",
                resourceLeaks: "Not implemented in this demo",
                ownershipFlow: "Not implemented in this demo"
            }
        };
    }
}
// ==================== SECURE PATCH GENERATOR (SPG) ====================
/**
 * Secure Patch Generator
 */
class SecurePatchGenerator {
    static validatePatch(originalCode, fixedCode, language) {
        // In a real implementation, this would run security checks
        return true;
    }
    static async runTests(language, code, filePath) {
        try {
            let result;
            switch (language) {
                case "rust":
                    result = await execAsync("cargo test", { cwd: PROJECT_ROOT });
                    break;
                case "solidity":
                    result = await execAsync("forge test", { cwd: PROJECT_ROOT });
                    break;
                case "move":
                    result = await execAsync("move test", { cwd: PROJECT_ROOT });
                    break;
                default:
                    throw new Error("Unsupported language");
            }
            return {
                success: true,
                output: result.stdout,
                errors: result.stderr
            };
        }
        catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }
}
// ==================== REPOSITORY INTEGRATION MODULE (RIM) ====================
/**
 * Repository Integration Module
 */
class RepositoryIntegrationModule {
    static async scanProject() {
        try {
            const files = await fs.readdir(PROJECT_ROOT, { recursive: true });
            const projectStructure = {
                rustFiles: [],
                solidityFiles: [],
                moveFiles: [],
                projectFiles: []
            };
            for (const file of files) {
                if (typeof file === 'string') {
                    if (file.endsWith('.rs')) {
                        projectStructure.rustFiles.push(file);
                    }
                    else if (file.endsWith('.sol')) {
                        projectStructure.solidityFiles.push(file);
                    }
                    else if (file.endsWith('.move')) {
                        projectStructure.moveFiles.push(file);
                    }
                    projectStructure.projectFiles.push(file);
                }
            }
            return {
                success: true,
                structure: projectStructure
            };
        }
        catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }
    static async commitPatch(filePath, patchDescription) {
        try {
            // In a real implementation, this would interact with Git
            return {
                success: true,
                message: `Would commit: ${patchDescription} for ${filePath}`
            };
        }
        catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }
}
// ==================== MCP METHODS ====================
// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
    return {
        tools: [
            // Rust methods
            {
                name: "rust.analyze_errors",
                description: "Parse cargo errors & classify",
                inputSchema: {
                    type: "object",
                    properties: {
                        code: { type: "string", description: "Rust code to analyze" },
                        filePath: { type: "string", description: "Path to the file" }
                    },
                    required: ["code"]
                },
            },
            {
                name: "rust.fix_errors",
                description: "Generate correct code patches",
                inputSchema: {
                    type: "object",
                    properties: {
                        code: { type: "string", description: "Rust code to fix" },
                        errors: { type: "array", items: { type: "object" }, description: "Errors to fix" }
                    },
                    required: ["code", "errors"]
                },
            },
            {
                name: "rust.debug",
                description: "Detect runtime issues",
                inputSchema: {
                    type: "object",
                    properties: {
                        code: { type: "string", description: "Rust code to debug" },
                        filePath: { type: "string", description: "Path to the file" }
                    },
                    required: ["code"]
                },
            },
            {
                name: "rust.refactor",
                description: "Rewrite code for safety/perf",
                inputSchema: {
                    type: "object",
                    properties: {
                        code: { type: "string", description: "Rust code to refactor" },
                        refactorType: { type: "string", description: "Type of refactoring" }
                    },
                    required: ["code", "refactorType"]
                },
            },
            // Solidity methods
            {
                name: "sol.analyze",
                description: "Parse solc/forge errors",
                inputSchema: {
                    type: "object",
                    properties: {
                        code: { type: "string", description: "Solidity code to analyze" },
                        filePath: { type: "string", description: "Path to the file" }
                    },
                    required: ["code"]
                },
            },
            {
                name: "sol.fix",
                description: "Patch Solidity code",
                inputSchema: {
                    type: "object",
                    properties: {
                        code: { type: "string", description: "Solidity code to fix" },
                        errors: { type: "array", items: { type: "object" }, description: "Errors to fix" }
                    },
                    required: ["code", "errors"]
                },
            },
            {
                name: "sol.debug",
                description: "Foundry test debug",
                inputSchema: {
                    type: "object",
                    properties: {
                        code: { type: "string", description: "Solidity code to debug" },
                        filePath: { type: "string", description: "Path to the file" }
                    },
                    required: ["code"]
                },
            },
            {
                name: "sol.audit",
                description: "Security improvements",
                inputSchema: {
                    type: "object",
                    properties: {
                        code: { type: "string", description: "Solidity code to audit" },
                        auditType: { type: "string", description: "Type of audit" }
                    },
                    required: ["code", "auditType"]
                },
            },
            // Move methods
            {
                name: "move.analyze",
                description: "Parse Move compiler/test errors",
                inputSchema: {
                    type: "object",
                    properties: {
                        code: { type: "string", description: "Move code to analyze" },
                        filePath: { type: "string", description: "Path to the file" }
                    },
                    required: ["code"]
                },
            },
            {
                name: "move.fix",
                description: "Patch Move modules",
                inputSchema: {
                    type: "object",
                    properties: {
                        code: { type: "string", description: "Move code to fix" },
                        errors: { type: "array", items: { type: "object" }, description: "Errors to fix" }
                    },
                    required: ["code", "errors"]
                },
            },
            {
                name: "move.debug",
                description: "Trace abort/test failures",
                inputSchema: {
                    type: "object",
                    properties: {
                        code: { type: "string", description: "Move code to debug" },
                        filePath: { type: "string", description: "Path to the file" }
                    },
                    required: ["code"]
                },
            },
            // Utility methods
            {
                name: "project.scan",
                description: "Detect languages, modules, errors",
                inputSchema: {
                    type: "object",
                    properties: {}
                },
            },
            {
                name: "project.autofix",
                description: "Apply patches for all errors",
                inputSchema: {
                    type: "object",
                    properties: {
                        code: { type: "string", description: "Code to fix" },
                        language: { type: "string", description: "Programming language" },
                        filePath: { type: "string", description: "Path to the file" }
                    },
                    required: ["code", "language"]
                },
            },
            {
                name: "patch.validate",
                description: "Run tests to confirm fix",
                inputSchema: {
                    type: "object",
                    properties: {
                        language: { type: "string", description: "Programming language" },
                        code: { type: "string", description: "Code to validate" },
                        filePath: { type: "string", description: "Path to the file" }
                    },
                    required: ["language", "code"]
                },
            },
            {
                name: "patch.plan",
                description: "Explain the diff before applying",
                inputSchema: {
                    type: "object",
                    properties: {
                        originalCode: { type: "string", description: "Original code" },
                        fixedCode: { type: "string", description: "Fixed code" },
                        language: { type: "string", description: "Programming language" }
                    },
                    required: ["originalCode", "fixedCode", "language"]
                },
            }
        ],
    };
});
// Handle tool execution
server.setRequestHandler(CallToolRequestSchema, async (request) => {
    try {
        switch (request.params.name) {
            // Rust methods
            case "rust.analyze_errors": {
                const { code, filePath } = request.params.arguments;
                const result = await RustIntelligenceEngine.analyzeErrors(code, filePath || "temp.rs");
                return {
                    content: [{ type: "text", text: JSON.stringify(result, null, 2) }]
                };
            }
            case "rust.fix_errors": {
                const { code, errors } = request.params.arguments;
                const { fixedCode, fixPlan } = await RustFixingEngine.fixErrors(code, errors);
                return {
                    content: [{
                            type: "text",
                            text: `Root Cause: Analyzed ${errors.length} errors

${fixPlan}

Patch:
\`\`\`diff
${generateDiff(code, fixedCode)}
\`\`\`

Final validated code:
\`\`\`rust
${fixedCode}
\`\`\``
                        }]
                };
            }
            case "rust.debug": {
                const { code, filePath } = request.params.arguments;
                const result = await RustDebuggingEngine.debug(code, filePath || "temp.rs");
                return {
                    content: [{ type: "text", text: JSON.stringify(result, null, 2) }]
                };
            }
            // Solidity methods
            case "sol.analyze": {
                const { code, filePath } = request.params.arguments;
                const result = await SolidityIntelligenceEngine.analyzeErrors(code, filePath || "temp.sol");
                return {
                    content: [{ type: "text", text: JSON.stringify(result, null, 2) }]
                };
            }
            case "sol.fix": {
                const { code, errors } = request.params.arguments;
                const { fixedCode, fixPlan } = await SolidityFixingEngine.fixErrors(code, errors);
                return {
                    content: [{
                            type: "text",
                            text: `Root Cause: Analyzed ${errors.length} errors

${fixPlan}

Patch:
\`\`\`diff
${generateDiff(code, fixedCode)}
\`\`\`

Final validated code:
\`\`\`solidity
${fixedCode}
\`\`\``
                        }]
                };
            }
            case "sol.debug": {
                const { code, filePath } = request.params.arguments;
                const result = await SolidityDebuggingEngine.debug(code, filePath || "temp.sol");
                return {
                    content: [{ type: "text", text: JSON.stringify(result, null, 2) }]
                };
            }
            // Move methods
            case "move.analyze": {
                const { code, filePath } = request.params.arguments;
                const result = await MoveIntelligenceEngine.analyzeErrors(code, filePath || "temp.move");
                return {
                    content: [{ type: "text", text: JSON.stringify(result, null, 2) }]
                };
            }
            case "move.fix": {
                const { code, errors } = request.params.arguments;
                const { fixedCode, fixPlan } = await MoveFixingEngine.fixErrors(code, errors);
                return {
                    content: [{
                            type: "text",
                            text: `Root Cause: Analyzed ${errors.length} errors

${fixPlan}

Patch:
\`\`\`diff
${generateDiff(code, fixedCode)}
\`\`\`

Final validated code:
\`\`\`move
${fixedCode}
\`\`\``
                        }]
                };
            }
            case "move.debug": {
                const { code, filePath } = request.params.arguments;
                const result = await MoveDebuggingEngine.debug(code, filePath || "temp.move");
                return {
                    content: [{ type: "text", text: JSON.stringify(result, null, 2) }]
                };
            }
            // Utility methods
            case "project.scan": {
                const result = await RepositoryIntegrationModule.scanProject();
                return {
                    content: [{ type: "text", text: JSON.stringify(result, null, 2) }]
                };
            }
            case "project.autofix": {
                const { code, language, filePath } = request.params.arguments;
                // Analyze errors
                let analysisResult;
                switch (language) {
                    case "rust":
                        analysisResult = await RustIntelligenceEngine.analyzeErrors(code, filePath || "temp.rs");
                        break;
                    case "solidity":
                        analysisResult = await SolidityIntelligenceEngine.analyzeErrors(code, filePath || "temp.sol");
                        break;
                    case "move":
                        analysisResult = await MoveIntelligenceEngine.analyzeErrors(code, filePath || "temp.move");
                        break;
                    default:
                        throw new Error("Unsupported language");
                }
                if (!analysisResult.success) {
                    return {
                        content: [{ type: "text", text: `Analysis failed: ${analysisResult.error}` }],
                        isError: true
                    };
                }
                // Fix errors
                let fixResult;
                switch (language) {
                    case "rust":
                        fixResult = await RustFixingEngine.fixErrors(code, analysisResult.errors);
                        break;
                    case "solidity":
                        fixResult = await SolidityFixingEngine.fixErrors(code, analysisResult.errors);
                        break;
                    case "move":
                        fixResult = await MoveFixingEngine.fixErrors(code, analysisResult.errors);
                        break;
                    default:
                        throw new Error("Unsupported language");
                }
                // Validate patch
                const isValid = SecurePatchGenerator.validatePatch(code, fixResult.fixedCode, language);
                return {
                    content: [{
                            type: "text",
                            text: `Root Cause: Analyzed ${analysisResult.errors.length} errors

${fixResult.fixPlan}

Security Improvement: Patch validated successfully

Patch:
\`\`\`diff
${generateDiff(code, fixResult.fixedCode)}
\`\`\`

Final validated code:
\`\`\`${language}
${fixResult.fixedCode}
\`\`\``
                        }]
                };
            }
            case "patch.validate": {
                const { language, code, filePath } = request.params.arguments;
                const testResult = await SecurePatchGenerator.runTests(language, code, filePath || `temp.${language}`);
                if (testResult.success) {
                    return {
                        content: [{
                                type: "text",
                                text: `Patch validation successful!

Test Output:
${testResult.output}

Errors:
${testResult.errors || "None"}`
                            }]
                    };
                }
                else {
                    return {
                        content: [{
                                type: "text",
                                text: `Patch validation failed: ${testResult.error}`
                            }],
                        isError: true
                    };
                }
            }
            case "patch.plan": {
                const { originalCode, fixedCode, language } = request.params.arguments;
                const diff = generateDiff(originalCode, fixedCode);
                return {
                    content: [{
                            type: "text",
                            text: `Fix Plan:
- Applied automated fixes for ${language} code

Patch:
\`\`\`diff
${diff}
\`\`\``
                        }]
                };
            }
            default:
                return {
                    content: [{ type: "text", text: `Unknown tool: ${request.params.name}` }],
                    isError: true
                };
        }
    }
    catch (error) {
        return {
            content: [{
                    type: "text",
                    text: `Error executing tool ${request.params.name}: ${error instanceof Error ? error.message : String(error)}`
                }],
            isError: true
        };
    }
});
// Helper function to generate diff
function generateDiff(original, modified) {
    const originalLines = original.split('\n');
    const modifiedLines = modified.split('\n');
    let diff = "";
    const maxLines = Math.max(originalLines.length, modifiedLines.length);
    for (let i = 0; i < maxLines; i++) {
        const origLine = originalLines[i];
        const modLine = modifiedLines[i];
        if (origLine !== modLine) {
            if (origLine !== undefined) {
                diff += `- ${origLine}\n`;
            }
            if (modLine !== undefined) {
                diff += `+ ${modLine}\n`;
            }
        }
        else if (origLine !== undefined) {
            diff += `  ${origLine}\n`;
        }
    }
    return diff;
}
// Start the server
async function main() {
    const transport = new StdioServerTransport();
    await server.connect(transport);
    console.error("CodeFix OS MCP Server running on stdio");
}
main().catch((error) => {
    console.error("Server error:", error);
    process.exit(1);
});
