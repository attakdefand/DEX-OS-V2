const fs = require('fs');
const path = require('path');

// Function to parse and display current implementation status
async function monitorImplementation() {
    console.log("🔍 Monitoring Implementation Status");
    console.log("=================================");
    
    try {
        // Check the main DEX-OS-V2.csv file
        const csvPath = path.join(__dirname, "..", "DEX-OS-V2", "DEX-OS-V2.csv");
        
        if (fs.existsSync(csvPath)) {
            const csvContent = fs.readFileSync(csvPath, "utf-8");
            const lines = csvContent.split('\n').filter(line => line.trim() !== '');
            
            let totalFeatures = 0;
            let implementedFeatures = 0;
            
            // Skip header line
            for (let i = 1; i < lines.length; i++) {
                const line = lines[i];
                if (line.includes('[IMPLEMENTED]')) {
                    implementedFeatures++;
                }
                totalFeatures++;
            }
            
            console.log(`✅ Main Features: ${implementedFeatures}/${totalFeatures} implemented (${((implementedFeatures/totalFeatures)*100).toFixed(2)}%)`);
        }
        
        // Check gold reference layer files
        const goldReferencePath = path.join(__dirname, "..", "DEX-OS-V2", ".reference", "layers", "gold");
        
        if (fs.existsSync(goldReferencePath)) {
            const files = fs.readdirSync(goldReferencePath).filter(file => file.endsWith('.csv'));
            
            console.log(`\n📁 Gold Reference Layer Files:`);
            
            let totalTestCases = 0;
            for (const file of files) {
                const filePath = path.join(goldReferencePath, file);
                const content = fs.readFileSync(filePath, "utf-8");
                const lines = content.split('\n').filter(line => line.trim() !== '');
                const testCount = lines.length - 1; // Subtract 1 for header
                
                console.log(`   📄 ${file}: ${testCount} test cases`);
                totalTestCases += testCount;
            }
            
            console.log(`\n📈 Total Test Cases: ${totalTestCases}`);
        }
        
    } catch (error) {
        console.error("❌ Error monitoring implementation:", error.message);
    }
}

// Run the monitor
monitorImplementation();