import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
    CallToolRequestSchema,
    ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import fs from "fs/promises";
import path from "path";
import { parse } from "csv-parse";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

// Define the server
const server = new Server(
    {
        name: "code-fix-mcp-server",
        version: "0.1.0",
    },
    {
        capabilities: {
            tools: {},
        },
    }
);

// Helper function to detect CSV errors
function detectCsvErrors(records) {
    const errors = [];
    
    // Check for inconsistent column counts
    if (records.length > 0) {
        const expectedColumns = records[0].length;
        records.forEach((record, index) => {
            // Skip empty rows
            if (record.length === 0 || (record.length === 1 && record[0].trim() === '')) {
                return;
            }
            
            // Skip comment rows (lines starting with //)
            if (record.length > 0 && record[0].startsWith('//')) {
                return;
            }
            
            if (record.length !== expectedColumns) {
                errors.push({
                    line: index + 1,
                    type: 'Column Count Mismatch',
                    expected: expectedColumns,
                    actual: record.length,
                    data: record
                });
            }
        });
    }
    
    // Check for duplicate [IMPLEMENTED] markers
    records.forEach((record, index) => {
        // Skip empty rows
        if (record.length === 0 || (record.length === 1 && record[0].trim() === '')) {
            return;
        }
        
        // Skip comment rows
        if (record.length > 0 && record[0].startsWith('//')) {
            return;
        }
        
        const recordData = record.join(',');
        const implementedCount = (recordData.match(/\[IMPLEMENTED\]/g) || []).length;
        if (implementedCount > 1) {
            errors.push({
                line: index + 1,
                type: 'Duplicate [IMPLEMENTED] Marker',
                count: implementedCount,
                data: record
            });
        }
    });
    
    // Check for malformed security tags
    records.forEach((record, index) => {
        // Skip empty rows
        if (record.length === 0 || (record.length === 1 && record[0].trim() === '')) {
            return;
        }
        
        // Skip comment rows
        if (record.length > 0 && record[0].startsWith('//')) {
            return;
        }
        
        const recordData = record.join(',');
        const securityTags = (recordData.match(/\{Security: Layer \d+ - [^\}]*\}/g) || []);
        if (securityTags.length > 1) {
            // Check if they're identical
            const uniqueTags = [...new Set(securityTags)];
            if (uniqueTags.length < securityTags.length) {
                errors.push({
                    line: index + 1,
                    type: 'Duplicate Security Tags',
                    data: record
                });
            }
        }
    });
    
    return errors;
}

// Helper function to fix CSV errors
function fixCsvErrors(records) {
    const fixedRecords = [];
    let fixedCount = 0;
    
    records.forEach((record, index) => {
        // Skip header row
        if (index === 0) {
            fixedRecords.push(record);
            return;
        }
        
        // Skip empty rows
        if (record.length === 0 || (record.length === 1 && record[0].trim() === '')) {
            fixedRecords.push(record);
            return;
        }
        
        // Skip comment rows
        if (record.length > 0 && record[0].startsWith('//')) {
            fixedRecords.push(record);
            return;
        }
        
        let fixedRecord = [...record];
        let recordFixed = false;
        
        // Fix column count issues by merging extra columns
        if (record.length > 6) {
            // Merge extra columns into the last column (Task Priority)
            const mergedLastColumn = record.slice(5).join(' ');
            fixedRecord = record.slice(0, 5);
            fixedRecord.push(mergedLastColumn);
            recordFixed = true;
        }
        
        // Fix duplicate [IMPLEMENTED] markers in the last column
        if (fixedRecord.length >= 6) {
            const lastColumn = fixedRecord[5];
            if (typeof lastColumn === 'string' && lastColumn.includes('[IMPLEMENTED]')) {
                const implementedCount = (lastColumn.match(/\[IMPLEMENTED\]/g) || []).length;
                if (implementedCount > 1) {
                    // Remove all [IMPLEMENTED] markers and add one at the end
                    const cleanedColumn = lastColumn.replace(/\s*\[IMPLEMENTED\]\s*/g, '').trim();
                    fixedRecord[5] = cleanedColumn + ' [IMPLEMENTED]';
                    recordFixed = true;
                }
            }
            
            // Fix duplicate security tags
            const lastCol = fixedRecord[5];
            if (typeof lastCol === 'string') {
                const securityTags = (lastCol.match(/\{Security: Layer \d+ - [^\}]*\}/g) || []);
                if (securityTags.length > 1) {
                    // Keep only unique tags
                    const uniqueTags = [...new Set(securityTags)];
                    if (uniqueTags.length < securityTags.length) {
                        let newColumn = lastCol;
                        securityTags.forEach(tag => {
                            // Remove all instances
                            newColumn = newColumn.replace(tag, '');
                        });
                        // Add back unique tags
                        uniqueTags.forEach(tag => {
                            if (!newColumn.includes(tag)) {
                                newColumn += ' ' + tag;
                            }
                        });
                        fixedRecord[5] = newColumn.trim();
                        recordFixed = true;
                    }
                }
            }
        }
        
        fixedRecords.push(fixedRecord);
        if (recordFixed) fixedCount++;
    });
    
    return { fixedRecords, fixedCount };
}

// Helper: run a shell command in a directory
function runCmd(cmd, cwd = null) {
    return new Promise((resolve) => {
        exec(cmd, { cwd }, (error, stdout, stderr) => {
            resolve({
                cmd,
                cwd,
                exit_code: error ? error.code : 0,
                stdout: stdout.toString(),
                stderr: stderr.toString(),
            });
        });
    });
}

// Tool to analyze CSV file for errors
server.setRequestHandler(CallToolRequestSchema, async (request) => {
    if (request.params.name === "analyze_csv_errors") {
        try {
            const csvPath = path.resolve(request.params.arguments.csv_path);
            
            // Check if file exists
            try {
                await fs.access(csvPath);
            } catch (error) {
                return {
                    error: `CSV file not found: ${csvPath}`
                };
            }
            
            // Read CSV file
            const csvContent = await fs.readFile(csvPath, "utf-8");
            const records = await new Promise((resolve, reject) => {
                parse(csvContent, {
                    columns: false,
                    skip_empty_lines: true,
                    relax_quotes: true,
                    relax_column_count: true
                }, (err, records) => {
                    if (err) reject(err);
                    else resolve(records);
                });
            });
            
            // Detect errors
            const errors = detectCsvErrors(records);
            
            return {
                content: [{
                    type: "text",
                    text: JSON.stringify({
                        total_rows: records.length,
                        errors_found: errors.length,
                        error_details: errors,
                        message: `Analysis complete. Found ${errors.length} errors in ${records.length} rows.`
                    }, null, 2)
                }]
            };
        } catch (error) {
            return {
                error: `Failed to analyze CSV file: ${error.message}`
            };
        }
    }
    
    if (request.params.name === "fix_csv_errors") {
        try {
            const csvPath = path.resolve(request.params.arguments.csv_path);
            const createBackup = request.params.arguments.backup !== false;
            
            // Check if file exists
            try {
                await fs.access(csvPath);
            } catch (error) {
                return {
                    error: `CSV file not found: ${csvPath}`
                };
            }
            
            // Create backup if requested
            if (createBackup) {
                const backupPath = csvPath + '.backup';
                await fs.copyFile(csvPath, backupPath);
            }
            
            // Read CSV file
            const csvContent = await fs.readFile(csvPath, "utf-8");
            const records = await new Promise((resolve, reject) => {
                parse(csvContent, {
                    columns: false,
                    skip_empty_lines: true,
                    relax_quotes: true,
                    relax_column_count: true
                }, (err, records) => {
                    if (err) reject(err);
                    else resolve(records);
                });
            });
            
            // Fix errors
            const { fixedRecords, fixedCount } = fixCsvErrors(records);
            
            // Convert back to CSV format properly
            let fixedCsvContent = '';
            
            fixedRecords.forEach((record, index) => {
                if (record.length === 0) {
                    fixedCsvContent += '\n';
                    return;
                }
                
                const escapedRecord = record.map(field => {
                    if (typeof field === 'string' && (field.includes(',') || field.includes('"') || field.includes('\n'))) {
                        // Escape quotes and wrap in quotes
                        return `"${field.replace(/"/g, '""')}"`;
                    }
                    return field;
                });
                
                fixedCsvContent += escapedRecord.join(',') + '\n';
            });
            
            await fs.writeFile(csvPath, fixedCsvContent);
            
            return {
                content: [{
                    type: "text",
                    text: JSON.stringify({
                        rows_processed: records.length,
                        errors_fixed: fixedCount,
                        backup_created: createBackup,
                        message: `Fixed ${fixedCount} errors in ${records.length} rows.${createBackup ? ' Backup created.' : ''}`
                    }, null, 2)
                }]
            };
        } catch (error) {
            return {
                error: `Failed to fix CSV file: ${error.message}`
            };
        }
    }
    
    if (request.params.name === "validate_csv_structure") {
        try {
            const csvPath = path.resolve(request.params.arguments.csv_path);
            
            // Check if file exists
            try {
                await fs.access(csvPath);
            } catch (error) {
                return {
                    error: `CSV file not found: ${csvPath}`
                };
            }
            
            // Read CSV file
            const csvContent = await fs.readFile(csvPath, "utf-8");
            const records = await new Promise((resolve, reject) => {
                parse(csvContent, {
                    columns: false,
                    skip_empty_lines: true,
                    relax_quotes: true,
                    relax_column_count: true
                }, (err, records) => {
                    if (err) reject(err);
                    else resolve(records);
                });
            });
            
            // Validate structure
            const expectedHeaders = [
                'Development Priority',
                'Category',
                'Component',
                'Algorithm/Data Structure',
                'Feature',
                'Task Priority'
            ];
            
            let isValid = true;
            let issues = [];
            
            if (records.length > 0) {
                const actualHeaders = records[0];
                if (actualHeaders.length !== expectedHeaders.length) {
                    isValid = false;
                    issues.push(`Expected ${expectedHeaders.length} columns, found ${actualHeaders.length}`);
                }
                
                expectedHeaders.forEach((header, index) => {
                    if (actualHeaders[index] !== header) {
                        isValid = false;
                        issues.push(`Column ${index + 1}: Expected "${header}", found "${actualHeaders[index]}"`);
                    }
                });
            }
            
            return {
                content: [{
                    type: "text",
                    text: JSON.stringify({
                        is_valid: isValid,
                        issues: issues,
                        total_rows: records.length,
                        message: isValid ? 
                            `CSV structure is valid with ${records.length} rows.` : 
                            `CSV structure has issues: ${issues.join('; ')}`
                    }, null, 2)
                }]
            };
        } catch (error) {
            return {
                error: `Failed to validate CSV structure: ${error.message}`
            };
        }
    }
    
    // Rust tools
    if (request.params.name === "rust_check_project") {
        try {
            const projectPath = path.resolve(request.params.arguments.project_path);
            const cargoTomlPath = path.join(projectPath, "Cargo.toml");
            
            // Check if Cargo.toml exists
            try {
                await fs.access(cargoTomlPath);
            } catch (error) {
                return {
                    error: `Cargo.toml not found in ${projectPath}`
                };
            }
            
            // Run cargo check
            const result = await runCmd("cargo check", projectPath);
            
            return {
                content: [{
                    type: "text",
                    text: JSON.stringify({
                        language: "rust",
                        tool: "cargo check",
                        ...result
                    }, null, 2)
                }]
            };
        } catch (error) {
            return {
                error: `Failed to run rust check: ${error.message}`
            };
        }
    }
    
    if (request.params.name === "rust_fmt_project") {
        try {
            const projectPath = path.resolve(request.params.arguments.project_path);
            const cargoTomlPath = path.join(projectPath, "Cargo.toml");
            
            // Check if Cargo.toml exists
            try {
                await fs.access(cargoTomlPath);
            } catch (error) {
                return {
                    error: `Cargo.toml not found in ${projectPath}`
                };
            }
            
            // Run cargo fmt
            const result = await runCmd("cargo fmt", projectPath);
            
            return {
                content: [{
                    type: "text",
                    text: JSON.stringify({
                        language: "rust",
                        tool: "cargo fmt",
                        ...result
                    }, null, 2)
                }]
            };
        } catch (error) {
            return {
                error: `Failed to run rust fmt: ${error.message}`
            };
        }
    }
    
    // Solidity tools
    if (request.params.name === "solidity_build_project") {
        try {
            const projectPath = path.resolve(request.params.arguments.project_path);
            const mainFile = request.params.arguments.main_file || "";
            
            const foundryConf = path.join(projectPath, "foundry.toml");
            const hardhatConf = path.join(projectPath, "hardhat.config.js");
            
            let result, toolUsed;
            
            // Prefer Forge if available
            try {
                await fs.access(foundryConf);
                result = await runCmd("forge build", projectPath);
                toolUsed = "forge build";
            } catch {
                // Fall back to solc on a single file
                if (!mainFile) {
                    return {
                        error: "No foundry.toml. Please specify 'main_file' for solc."
                    };
                }
                const solPath = path.join(projectPath, mainFile);
                try {
                    await fs.access(solPath);
                } catch {
                    return {
                        error: `Solidity file not found: ${solPath}`
                    };
                }
                
                result = await runCmd(`solc --optimize --bin --abi "${solPath}"`, projectPath);
                toolUsed = "solc";
            }
            
            return {
                content: [{
                    type: "text",
                    text: JSON.stringify({
                        language: "solidity",
                        tool: toolUsed,
                        ...result
                    }, null, 2)
                }]
            };
        } catch (error) {
            return {
                error: `Failed to build solidity project: ${error.message}`
            };
        }
    }
    
    // File utilities
    if (request.params.name === "read_code_file") {
        try {
            const filePath = path.resolve(request.params.arguments.path);
            
            // Check if file exists
            try {
                await fs.access(filePath);
            } catch (error) {
                return {
                    error: `File not found: ${filePath}`
                };
            }
            
            const content = await fs.readFile(filePath, "utf-8");
            
            return {
                content: [{
                    type: "text",
                    text: JSON.stringify({
                        path: filePath,
                        content: content
                    }, null, 2)
                }]
            };
        } catch (error) {
            return {
                error: `Failed to read file: ${error.message}`
            };
        }
    }
    
    if (request.params.name === "write_code_file") {
        try {
            const filePath = path.resolve(request.params.arguments.path);
            const content = request.params.arguments.content;
            
            // Create directory if it doesn't exist
            const dirPath = path.dirname(filePath);
            await fs.mkdir(dirPath, { recursive: true });
            
            await fs.writeFile(filePath, content, "utf-8");
            
            return {
                content: [{
                    type: "text",
                    text: JSON.stringify({
                        status: "ok",
                        message: `Wrote file ${filePath}`
                    }, null, 2)
                }]
            };
        } catch (error) {
            return {
                error: `Failed to write file: ${error.message}`
            };
        }
    }
    
    if (request.params.name === "apply_patch") {
        try {
            const filePath = path.resolve(request.params.arguments.path);
            const patches = request.params.arguments.patches;
            
            // Check if file exists
            try {
                await fs.access(filePath);
            } catch (error) {
                return {
                    error: `File not found: ${filePath}`
                };
            }
            
            let content = await fs.readFile(filePath, "utf-8");
            const originalContent = content;
            const applied = [];
            
            for (const patch of patches) {
                const oldText = patch.old_text;
                const newText = patch.new_text;
                if (content.includes(oldText)) {
                    content = content.replace(oldText, newText);
                    applied.push({ old_text: oldText, new_text: newText, status: "replaced" });
                } else {
                    applied.push({ old_text: oldText, new_text: newText, status: "not_found" });
                }
            }
            
            if (content !== originalContent) {
                await fs.writeFile(filePath, content, "utf-8");
            }
            
            return {
                content: [{
                    type: "text",
                    text: JSON.stringify({
                        status: "ok",
                        file: filePath,
                        patches: applied
                    }, null, 2)
                }]
            };
        } catch (error) {
            return {
                error: `Failed to apply patch: ${error.message}`
            };
        }
    }
    
    return {
        error: `Unknown tool: ${request.params.name}`
    };
});

// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
    return {
        tools: [
            // CSV tools
            {
                name: "analyze_csv_errors",
                description: "Analyze DEX-OS-V2.csv file for formatting errors and inconsistencies",
                inputSchema: {
                    type: "object",
                    properties: {
                        csv_path: {
                            type: "string",
                            description: "Path to the CSV file to analyze"
                        }
                    },
                    required: ["csv_path"],
                },
            },
            {
                name: "validate_csv_structure",
                description: "Validate the structure of DEX-OS-V2.csv file",
                inputSchema: {
                    type: "object",
                    properties: {
                        csv_path: {
                            type: "string",
                            description: "Path to the CSV file to validate"
                        }
                    },
                    required: ["csv_path"],
                },
            },
            {
                name: "fix_csv_errors",
                description: "Fix formatting errors and inconsistencies in DEX-OS-V2.csv file",
                inputSchema: {
                    type: "object",
                    properties: {
                        csv_path: {
                            type: "string",
                            description: "Path to the CSV file to fix"
                        },
                        backup: {
                            type: "boolean",
                            description: "Create a backup of the original file before fixing",
                            default: true
                        }
                    },
                    required: ["csv_path"],
                },
            },
            // Rust tools
            {
                name: "rust_check_project",
                description: "Run `cargo check` in a Rust project directory to get compiler errors and warnings",
                inputSchema: {
                    type: "object",
                    properties: {
                        project_path: {
                            type: "string",
                            description: "Path to the Rust project directory (where Cargo.toml lives)"
                        }
                    },
                    required: ["project_path"]
                }
            },
            {
                name: "rust_fmt_project",
                description: "Run `cargo fmt` in a Rust project (format code)",
                inputSchema: {
                    type: "object",
                    properties: {
                        project_path: {
                            type: "string"
                        }
                    },
                    required: ["project_path"]
                }
            },
            // Solidity tools
            {
                name: "solidity_build_project",
                description: "Compile Solidity project. If foundry.toml exists, uses `forge build`. Otherwise tries `solc` on a single file.",
                inputSchema: {
                    type: "object",
                    properties: {
                        project_path: {
                            type: "string",
                            description: "Path to the Solidity project (Foundry/Hardhat style) or directory containing .sol files."
                        },
                        main_file: {
                            type: "string",
                            description: "Optional: specific .sol file (relative to project_path) when using solc.",
                            default: ""
                        }
                    },
                    required: ["project_path"]
                }
            },
            // File utilities
            {
                name: "read_code_file",
                description: "Read a source file (Rust/Solidity/Move or any text file) and return its contents.",
                inputSchema: {
                    type: "object",
                    properties: {
                        path: {
                            type: "string",
                            description: "Path to the file (relative or absolute)."
                        }
                    },
                    required: ["path"]
                }
            },
            {
                name: "write_code_file",
                description: "Overwrite a source file with new content. Use with care.",
                inputSchema: {
                    type: "object",
                    properties: {
                        path: {
                            type: "string"
                        },
                        content: {
                            type: "string"
                        }
                    },
                    required: ["path", "content"]
                }
            },
            {
                name: "apply_patch",
                description: "Apply simple search-and-replace patches to a file. Each patch has old_text and new_text. This is useful for fixing errors suggested by the AI.",
                inputSchema: {
                    type: "object",
                    properties: {
                        path: {
                            type: "string"
                        },
                        patches: {
                            type: "array",
                            items: {
                                type: "object",
                                properties: {
                                    old_text: {
                                        type: "string"
                                    },
                                    new_text: {
                                        type: "string"
                                    }
                                },
                                required: ["old_text", "new_text"]
                            }
                        }
                    },
                    required: ["path", "patches"]
                }
            }
        ],
    };
});

// Start the server
async function main() {
    const transport = new StdioServerTransport();
    await server.connect(transport);
}

main().catch(console.error);