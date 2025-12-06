//! Example showing how to use the advanced DeFi features

use dex_core::advanced_defi::AdvancedDeFiPlatform;
use dex_core::governance::vote_escrow::VeNFTType;

fn main() {
    println!("Creating Advanced DeFi Platform...");
    let mut platform = AdvancedDeFiPlatform::new();
    
    // Create a Vote Escrow NFT (Supermassive)
    println!("Creating Vote Escrow NFT...");
    let venft_id = platform.create_venft(
        "user1".to_string(),
        1000,  // 1000 tokens
        365 * 24 * 60 * 60, // 1 year lock
        VeNFTType::Supermassive,
    );
    
    println!("Created veNFT with ID: {}", venft_id);
    println!("User voting power: {}", platform.get_owner_voting_power("user1"));
    
    // Create a Genesis Pool
    println!("Creating Genesis Pool...");
    platform.create_genesis_pool(
        "project_pool".to_string(),
        "PROJECT".to_string(),    // Project token
        "USDC".to_string(),       // Contribution token
        1000000,                  // 1M project tokens
        5000000,                  // $5M target
        1000,                     // Min contribution
        100000,                   // Max contribution
        1000000,                  // Start time
        2000000,                  // End time
    );
    
    // Contribute to the Genesis Pool
    println!("Contributing to Genesis Pool...");
    let result = platform.contribute_to_genesis_pool(
        "project_pool",
        "user1".to_string(),
        50000,     // 50,000 USDC
        1500000,   // Current time
    );
    
    match result {
        Ok(_) => println!("Successfully contributed to Genesis Pool"),
        Err(e) => println!("Failed to contribute: {}", e),
    }
    
    // Create a Concentrated Liquidity Pool
    println!("Creating Concentrated Liquidity Pool...");
    let pool_id = platform.create_concentrated_pool(
        "cl_pool1".to_string(),
        "TOKEN0".to_string(),
        "TOKEN1".to_string(),
        30, // 0.3% fee
        10,
        1000000, // 1:1 price (scaled)
    );
    
    println!("Created concentrated liquidity pool with ID: {}", pool_id);
    
    // Add liquidity to the pool
    println!("Adding liquidity to concentrated pool...");
    let result = platform.add_concentrated_liquidity(
        &pool_id,
        "user1".to_string(),
        -100,  // tick_lower
        100,   // tick_upper
        10000, // amount0
        10000, // amount1
    );
    
    match result {
        Ok(position_id) => println!("Successfully added liquidity. Position ID: {}", position_id),
        Err(e) => println!("Failed to add liquidity: {}", e),
    }
    
    // Perform a swap
    println!("Performing swap...");
    let result = platform.swap(
        &pool_id,
        true,   // zero_for_one (token0 for token1)
        1000,   // amount specified
    );
    
    match result {
        Ok((amount0, amount1)) => {
            println!("Swap successful!");
            println!("  Amount0 (token0): {}", amount0);
            println!("  Amount1 (token1): {}", amount1);
        },
        Err(e) => println!("Swap failed: {}", e),
    }
    
    // Show platform statistics
    let stats = platform.get_platform_stats();
    println!("Platform Statistics:");
    println!("  Total veNFTs: {}", stats.total_venfts);
    println!("  Total Genesis Pools: {}", stats.total_genesis_pools);
    println!("  Total Concentrated Liquidity Pools: {}", stats.total_cl_pools);
    
    println!("Advanced DeFi Platform example completed!");
}