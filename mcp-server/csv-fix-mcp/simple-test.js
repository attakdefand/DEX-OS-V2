import fs from "fs/promises";
import path from "path";
import { parse } from "csv-parse";

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

async function main() {
    try {
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
        
        console.log(`Loaded ${records.length} rows from CSV file`);
        
        // Detect errors
        const errors = detectCsvErrors(records);
        
        console.log(`Found ${errors.length} errors:`);
        errors.forEach(error => {
            console.log(`  Line ${error.line}: ${error.type}`);
        });
        
        if (errors.length > 0) {
            console.log("\nFirst few error details:");
            console.log(JSON.stringify(errors.slice(0, 3), null, 2));
        } else {
            console.log("✅ No errors found in the CSV file!");
        }
    } catch (error) {
        console.error('Error:', error);
    }
}

main().catch(console.error);