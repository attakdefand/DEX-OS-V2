#!/usr/bin/env node

// Build script for the Rust Debugging MCP Server
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('Building Rust Debugging MCP Server...');

try {
    // Ensure dist directory exists
    const distDir = path.join(__dirname, 'dist');
    if (!fs.existsSync(distDir)) {
        fs.mkdirSync(distDir);
        console.log('Created dist directory');
    }

    // Compile TypeScript files
    execSync('npx tsc', { stdio: 'inherit' });
    console.log('TypeScript compilation successful');

    // Copy necessary files to dist
    const filesToCopy = [
        'mcp.json',
        'package.json'
    ];

    filesToCopy.forEach(file => {
        const srcPath = path.join(__dirname, file);
        const destPath = path.join(distDir, file);
        if (fs.existsSync(srcPath)) {
            fs.copyFileSync(srcPath, destPath);
            console.log(`Copied ${file} to dist`);
        }
    });

    console.log('Build completed successfully!');
    console.log('To run the server:');
    console.log('  node dist/rust-debug-server.js');
    console.log('Or use npm:');
    console.log('  npm run dev-rust');

} catch (error) {
    console.error('Build failed:', error.message);
    process.exit(1);
}