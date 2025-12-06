const fs = require('fs/promises');
const path = require('path');

async function checkCsvFormat() {
  try {
    const PROJECT_ROOT = path.resolve(__dirname, "..", "DEX-OS-V2");
    const csvPath = path.join(PROJECT_ROOT, "DEX-OS-V2.csv");
    const csvContent = await fs.readFile(csvPath, "utf-8");
    
    // Split into lines and show first few lines
    const lines = csvContent.split('\n');
    console.log("First 10 lines of CSV:");
    for (let i = 0; i < Math.min(10, lines.length); i++) {
      console.log(`${i + 1}: ${lines[i]}`);
    }
    
    // Show line 3 which had the error
    console.log("\nLine 3 (problematic line):");
    console.log(`3: ${lines[2]}`);
    
    // Count commas in header line
    const header = lines[0];
    const commaCount = (header.match(/,/g) || []).length;
    console.log(`\nHeader has ${commaCount} commas`);
    
    // Count commas in line 3
    const line3 = lines[2];
    const line3Commas = (line3.match(/,/g) || []).length;
    console.log(`Line 3 has ${line3Commas} commas`);
    
  } catch (error) {
    console.error("Error reading CSV:", error.message);
  }
}

checkCsvFormat();