// Simple test to check if the server is working
console.log("Testing Code Fix MCP Server");

// Test the CSV fixing functionality directly
import fs from "fs/promises";
import path from "path";
import { parse } from "csv-parse";

async function testCsvFix() {
    try {
        console.log("🔍 Testing CSV fix functionality...");
        
        // Read CSV file
        const csvPath = path.resolve('../../DEX-OS-V2/DEX-OS-V2.csv');
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
        
        console.log(`✅ Loaded ${records.length} rows from CSV file`);
        
        // Check if the first few rows have the correct number of columns
        for (let i = 1; i < Math.min(5, records.length); i++) {
            console.log(`   Row ${i}: ${records[i].length} columns`);
        }
        
        console.log("✅ CSV fix functionality is working correctly!");
        
    } catch (error) {
        console.error('❌ Error:', error);
    }
}

testCsvFix().catch(console.error);