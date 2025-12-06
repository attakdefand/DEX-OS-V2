import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { CallToolRequestSchema, ListToolsRequestSchema, } from "@modelcontextprotocol/sdk/types.js";
import * as fs from "fs";
import * as fsPromises from "fs/promises";
import path from "path";
import csv from "csv-parser";
// Define the server
const server = new Server({
    name: "mcp-server-gold-diamond",
    version: "1.0.0",
}, {
    capabilities: {
        tools: {},
    },
});
// Root directory for project files
const PROJECT_ROOT = process.cwd();
const REFERENCE_LAYERS_PATH = path.join(PROJECT_ROOT, "..", "..", "DEX-OS-V2", "DEX-OS-V2", ".reference", "layers");
/**
 * Gold Diamond Protection Tests Engine
 */
class GoldDiamondProtectionEngine {
    static async loadProtectionTests(filePath) {
        const results = [];
        return new Promise((resolve, reject) => {
            fs.createReadStream(filePath)
                .pipe(csv())
                .on('data', (data) => results.push(data))
                .on('end', () => resolve(results))
                .on('error', (error) => reject(error));
        });
    }
    static async getAllProtectionTestFiles() {
        const files = [];
        try {
            // Get files from the gold layer directory
            const goldLayerPath = path.join(REFERENCE_LAYERS_PATH, "gold");
            const goldFiles = await fsPromises.readdir(goldLayerPath);
            for (const file of goldFiles) {
                // Include all CSV files that contain relevant keywords in their name
                if (file.endsWith('.csv') && (file.includes('protection') || file.includes('web3') || file.includes('detection') || file.includes('resilience') || file.includes('governance') || file.includes('compliance'))) {
                    files.push(path.join(goldLayerPath, file));
                }
            }
            return files;
        }
        catch (error) {
            console.error("Error scanning protection test files:", error);
            return [];
        }
    }
    static async searchProtectionTests(query) {
        try {
            const allFiles = await this.getAllProtectionTestFiles();
            let matchedTests = [];
            for (const file of allFiles) {
                const tests = await this.loadProtectionTests(file);
                // Search for matching tests
                for (const test of tests) {
                    // Check if query matches any field
                    const testString = Object.values(test).join(' ').toLowerCase();
                    if (testString.includes(query.toLowerCase())) {
                        matchedTests.push({
                            ...test,
                            source_file: path.basename(file)
                        });
                    }
                }
            }
            return matchedTests;
        }
        catch (error) {
            console.error("Error searching protection tests:", error);
            return [];
        }
    }
    static async getProtectionTestSummary() {
        try {
            const summary = {
                total_files: 0,
                total_tests: 0,
                files: []
            };
            // Get files from the gold layer directory
            const goldLayerPath = path.join(REFERENCE_LAYERS_PATH, "gold");
            const goldFiles = await fsPromises.readdir(goldLayerPath);
            for (const file of goldFiles) {
                // Include all CSV files that contain relevant keywords in their name
                if (file.endsWith('.csv') && (file.includes('protection') || file.includes('web3') || file.includes('detection') || file.includes('resilience') || file.includes('governance') || file.includes('compliance'))) {
                    summary.total_files++;
                    const filePath = path.join(goldLayerPath, file);
                    const tests = await this.loadProtectionTests(filePath);
                    summary.files.push({
                        name: file,
                        test_count: tests.length
                    });
                    summary.total_tests += tests.length;
                }
            }
            return summary;
        }
        catch (error) {
            console.error("Error getting protection test summary:", error);
            return {};
        }
    }
    /**
     * Implement a protection feature based on a test case
     */
    static async implementProtectionFeature(testCase) {
        try {
            // Extract key information from the test case
            const { layer, component, behavior, condition, test_name, owner, stack, tool, metric, sla, severity } = testCase;
            // Generate implementation plan
            const implementationPlan = {
                layer,
                component,
                behavior,
                condition,
                test_name,
                owner,
                technology_stack: stack,
                testing_tools: tool,
                success_metric: metric,
                sla_requirement: sla,
                severity,
                implementation_steps: [
                    `1. Create security module for ${component} in the ${layer} layer`,
                    `2. Implement ${behavior} behavior with ${condition} condition handling`,
                    `3. Add appropriate input validation and sanitization`,
                    `4. Implement access control enforcement mechanisms`,
                    `5. Add monitoring and logging for ${metric} metrics`,
                    `6. Create unit tests covering the ${condition} scenario`,
                    `7. Integrate with existing security framework`,
                    `8. Document the implementation and security considerations`
                ],
                suggested_code_structure: {
                    module_name: `${component}_${behavior}_${condition}`.toLowerCase().replace(/[^a-z0-9_]/g, '_'),
                    file_path: `dex-core/src/security/${layer.toLowerCase().replace(/[^a-z0-9_]/g, '_')}/${component.toLowerCase().replace(/[^a-z0-9_]/g, '_')}.rs`,
                    test_file_path: `dex-core/tests/security_${component.toLowerCase().replace(/[^a-z0-9_]/g, '_')}_tests.rs`
                }
            };
            return implementationPlan;
        }
        catch (error) {
            console.error("Error implementing protection feature:", error);
            throw error;
        }
    }
}
// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
    return {
        tools: [
            {
                name: "protection.search_tests",
                description: "Search protection tests by keyword",
                inputSchema: {
                    type: "object",
                    properties: {
                        query: { type: "string", description: "Search query" }
                    },
                    required: ["query"]
                },
            },
            {
                name: "protection.get_summary",
                description: "Get summary of all gold diamond protection tests",
                inputSchema: {
                    type: "object",
                    properties: {}
                },
            },
            {
                name: "protection.list_test_files",
                description: "List all available protection test files",
                inputSchema: {
                    type: "object",
                    properties: {}
                },
            },
            {
                name: "protection.implement_feature",
                description: "Generate implementation plan for a protection feature based on a test case",
                inputSchema: {
                    type: "object",
                    properties: {
                        test_case: {
                            type: "object",
                            description: "The protection test case to implement"
                        }
                    },
                    required: ["test_case"]
                },
            }
        ],
    };
});
// Handle tool execution
server.setRequestHandler(CallToolRequestSchema, async (request) => {
    try {
        switch (request.params.name) {
            case "protection.search_tests": {
                const { query } = request.params.arguments;
                const results = await GoldDiamondProtectionEngine.searchProtectionTests(query);
                return {
                    content: [{
                            type: "text",
                            text: JSON.stringify({
                                query,
                                results_count: results.length,
                                results: results.slice(0, 50) // Limit results for readability
                            }, null, 2)
                        }]
                };
            }
            case "protection.get_summary": {
                const summary = await GoldDiamondProtectionEngine.getProtectionTestSummary();
                return {
                    content: [{
                            type: "text",
                            text: JSON.stringify(summary, null, 2)
                        }]
                };
            }
            case "protection.list_test_files": {
                const files = await GoldDiamondProtectionEngine.getAllProtectionTestFiles();
                return {
                    content: [{
                            type: "text",
                            text: JSON.stringify({
                                total_files: files.length,
                                files: files.map(f => path.basename(f))
                            }, null, 2)
                        }]
                };
            }
            case "protection.implement_feature": {
                const { test_case } = request.params.arguments;
                const implementationPlan = await GoldDiamondProtectionEngine.implementProtectionFeature(test_case);
                return {
                    content: [{
                            type: "text",
                            text: JSON.stringify(implementationPlan, null, 2)
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
// Start the server
async function main() {
    const transport = new StdioServerTransport();
    await server.connect(transport);
    console.error("Gold Diamond Protection Tests MCP Server running on stdio");
}
main().catch((error) => {
    console.error("Server error:", error);
    process.exit(1);
});
