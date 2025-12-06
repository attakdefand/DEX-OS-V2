/**
 * Demo Script for Security Reference Layers MCP Server
 * 
 * This script demonstrates how to use the MCP server to access the DEX-OS-V2 security reference layers.
 * The reference layers contain over 60,000 Web3 vulnerabilities organized into 7 major groups.
 */

console.log("=== DEX-OS-V2 Security Reference Layers MCP Server Demo ===\n");

console.log("The Security Reference Layers MCP Server provides access to:");
console.log("- Over 60,000 Web3 vulnerabilities");
console.log("- Organized into 7 major groups:");
console.log("  1. Smart Contract Vulnerabilities");
console.log("  2. DeFi / Economic Exploits");
console.log("  3. Governance & Admin Failures");
console.log("  4. RPC / Node / Network Attacks");
console.log("  5. Wallet / DApp / Frontend Vulnerabilities");
console.log("  6. Bridge & Cross-Chain Attacks");
console.log("  7. Web2 Backend / API / Server Attacks\n");

console.log("Available Tools:");
console.log("1. security.search_tests - Search security tests by keyword");
console.log("2. security.get_layer_summary - Get summary of all security reference layers");
console.log("3. security.list_test_files - List all available security test files\n");

console.log("Example Usage:");
console.log("// Search for reentrancy vulnerabilities");
console.log('// {"method": "tools/call", "params": {"name": "security.search_tests", "arguments": {"query": "reentrancy"}}}');
console.log("");
console.log("// Get a summary of all layers");
console.log('// {"method": "tools/call", "params": {"name": "security.get_layer_summary", "arguments": {}}}');
console.log("");
console.log("// List all test files");
console.log('// {"method": "tools/call", "params": {"name": "security.list_test_files", "arguments": {}}}');

console.log("\n=== End of Demo ===");