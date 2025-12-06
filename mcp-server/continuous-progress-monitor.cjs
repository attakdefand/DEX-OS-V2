const fs = require('fs');
const path = require('path');

function checkProgress() {
    try {
        // Path to the protection tests CSV file
        const csvFilePath = path.join(__dirname, "..", "DEX-OS-V2", ".reference", "layers", "gold", "1. protection_tests_full_with_all_metadata.csv");
        
        // Read the file content
        const content = fs.readFileSync(csvFilePath, 'utf8');
        const lines = content.split('\n').filter(line => line.trim() !== '');
        
        // Calculate statistics
        const totalTestCases = lines.length - 1; // Subtract 1 for header
        let implementedCount = 0;
        
        // Check for [IMPLEMENTED] markers in each line (skip header)
        for (let i = 1; i < lines.length; i++) {
            if (lines[i].includes('[IMPLEMENTED]')) {
                implementedCount++;
            }
        }
        
        // Calculate progress
        const progressPercentage = totalTestCases > 0 ? ((implementedCount / totalTestCases) * 100).toFixed(2) : 0;
        
        // Clear console and display updated stats
        console.clear();
        console.log("🔄 CONTINUOUS PROTECTION TESTS PROGRESS MONITOR");
        console.log("=============================================");
        console.log(`📅 Last Updated: ${new Date().toLocaleTimeString()}`);
        console.log("");
        console.log(`📊 File: 1. protection_tests_full_with_all_metadata.csv`);
        console.log(`📈 Total Test Cases: ${totalTestCases.toLocaleString()}`);
        console.log(`✅ Implemented: ${implementedCount.toLocaleString()}`);
        console.log(`⏳ Pending: ${(totalTestCases - implementedCount).toLocaleString()}`);
        console.log(`📊 Progress: ${progressPercentage}%`);
        
        // Progress bar visualization
        const progressBarWidth = 50;
        const filledWidth = Math.round((implementedCount / totalTestCases) * progressBarWidth);
        const emptyWidth = progressBarWidth - filledWidth;
        const progressBar = '█'.repeat(filledWidth) + '░'.repeat(emptyWidth);
        console.log(`\n[${progressBar}] ${progressPercentage}%`);
        
        // Estimate completion time
        if (implementedCount < totalTestCases) {
            const pendingCount = totalTestCases - implementedCount;
            const batchesRemaining = Math.ceil(pendingCount / 5);
            const estimatedSeconds = batchesRemaining * 10;
            const estimatedMinutes = Math.floor(estimatedSeconds / 60);
            const remainingSeconds = estimatedSeconds % 60;
            
            console.log(`\n⏱️  Estimated Time to Completion:`);
            console.log(`   ${batchesRemaining} batches remaining`);
            console.log(`   Approximately ${estimatedMinutes} minutes and ${remainingSeconds} seconds`);
        } else {
            console.log(`\n🎉 ALL TEST CASES HAVE BEEN IMPLEMENTED AND TESTED!`);
        }
        
        console.log(`\n🔄 Monitoring updates every 30 seconds... (Press Ctrl+C to stop)`);
        
    } catch (error) {
        console.error("❌ Error checking progress:", error.message);
    }
}

// Initial check
checkProgress();

// Update every 30 seconds
setInterval(checkProgress, 30000);

// Handle Ctrl+C gracefully
process.on('SIGINT', function() {
    console.log('\n👋 Progress monitoring stopped.');
    process.exit();
});