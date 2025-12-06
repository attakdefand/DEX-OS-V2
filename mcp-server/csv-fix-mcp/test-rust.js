// Test Rust project checking functionality
import { exec } from "child_process";
import { promisify } from "util";
import fs from "fs/promises";
import path from "path";

const execAsync = promisify(exec);

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

async function testRustCheck() {
    try {
        console.log("🔍 Testing Rust check functionality...");
        
        // Check if we're in the right environment
        const projectRoot = path.resolve('../../');
        console.log(`Project root: ${projectRoot}`);
        
        // Check if cargo is available
        try {
            const versionResult = await runCmd("cargo --version");
            console.log(`✅ Cargo version: ${versionResult.stdout.trim()}`);
        } catch (error) {
            console.log("⚠️  Cargo not found or not in PATH");
            return;
        }
        
        // Check if dex-core exists
        const dexCorePath = path.join(projectRoot, 'DEX-OS-V2', 'dex-core');
        try {
            await fs.access(dexCorePath);
            console.log(`✅ Found dex-core at: ${dexCorePath}`);
        } catch (error) {
            console.log("⚠️  dex-core directory not found");
            return;
        }
        
        // Check if Cargo.toml exists in dex-core
        const cargoTomlPath = path.join(dexCorePath, 'Cargo.toml');
        try {
            await fs.access(cargoTomlPath);
            console.log(`✅ Found Cargo.toml at: ${cargoTomlPath}`);
        } catch (error) {
            console.log("⚠️  Cargo.toml not found in dex-core");
            return;
        }
        
        console.log("✅ Rust check functionality is ready!");
        
    } catch (error) {
        console.error('❌ Error:', error);
    }
}

testRustCheck().catch(console.error);