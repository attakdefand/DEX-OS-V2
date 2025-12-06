#!/usr/bin/env node

// Simple build script for the CodeFix OS MCP Server
const { execSync } = require('child_process');
const fs = require('fs');

console.log('Building CodeFix OS MCP Server...');

try {
    // Create dist directory if it doesn't exist
    if (!fs.existsSync('./dist')) {
        fs.mkdirSync('./dist');
    }

    // Compile TypeScript
    console.log('Compiling TypeScript...');
    execSync('npx tsc', { stdio: 'inherit' });

    console.log('Build completed successfully!');
    console.log('');
    console.log('To run the server:');
    console.log('  npm start');
    console.log('');
    console.log('To run the test client:');
    console.log('  node test-client.js');
} catch (error) {
    console.error('Build failed:', error.message);
    process.exit(1);
}