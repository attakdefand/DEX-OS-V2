import fs from "fs/promises";
import path from "path";
import { parse } from "csv-parse";

async function fixCsvFile() {
    try {
        // Read CSV file
        const csvPath = path.resolve('../../DEX-OS-V2/DEX-OS-V2.csv');
        const csvContent = await fs.readFile(csvPath, "utf-8");
        
        // Create backup
        const backupPath = csvPath + '.backup';
        await fs.copyFile(csvPath, backupPath);
        console.log('✅ Backup created:', backupPath);
        
        // Parse CSV with relaxed options
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
        
        console.log(`📄 Loaded ${records.length} rows from CSV file`);
        
        // Fix records
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
            
            let fixedRecord = [...record];
            let recordChanged = false;
            
            // Fix column count issues by merging extra columns
            if (record.length > 6) {
                // Merge extra columns into the last column (Task Priority)
                const mergedLastColumn = record.slice(5).join(' ');
                fixedRecord = record.slice(0, 5);
                fixedRecord.push(mergedLastColumn);
                recordChanged = true;
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
                        recordChanged = true;
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
                            recordChanged = true;
                        }
                    }
                }
            }
            
            fixedRecords.push(fixedRecord);
            if (recordChanged) fixedCount++;
        });
        
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
        
        // Write fixed CSV back to file
        await fs.writeFile(csvPath, fixedCsvContent);
        console.log(`✅ Fixed ${fixedCount} rows and saved to ${csvPath}`);
        
        // Validate the fix
        console.log('\n🔍 Validating fix...');
        const validationContent = await fs.readFile(csvPath, "utf-8");
        const validationRecords = await new Promise((resolve, reject) => {
            parse(validationContent, {
                columns: false,
                skip_empty_lines: true
            }, (err, records) => {
                if (err) reject(err);
                else resolve(records);
            });
        });
        
        console.log(`✅ Validation: ${validationRecords.length} rows loaded successfully`);
        
        // Check first few rows
        console.log('\n📋 First 3 rows after fix:');
        for (let i = 0; i < Math.min(3, validationRecords.length); i++) {
            console.log(`  Row ${i + 1}: ${validationRecords[i].length} columns`);
        }
        
    } catch (error) {
        console.error('❌ Error:', error);
    }
}

fixCsvFile().catch(console.error);