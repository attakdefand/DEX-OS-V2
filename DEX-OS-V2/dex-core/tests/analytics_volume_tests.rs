use dex_core::analytics::{AnalyticsService, AnalyticsConfig, VolumeTracker, TimeWindow};
use std::thread;
use std::time::Duration;
use std::sync::Arc;

#[test]
fn test_analytics_volume_tracking_integration() {
    // 1. Initialize Analytics Service
    let config = AnalyticsConfig {
        enable_volume_tracking: true,
        max_history_entries: 100,
        enable_time_windows: true,
    };
    let service = AnalyticsService::with_config(config);

    // 2. Simulate Trading Activity
    let pairs = vec![
        ("BTC", "USDT"),
        ("ETH", "USDT"),
        ("SOL", "USDT"),
        ("BTC", "ETH"),
    ];

    // Simulate a series of trades
    for _ in 0..10 {
        for (base, quote) in &pairs {
            service.record_trade_volume(base.to_string(), quote.to_string(), 1000).unwrap();
        }
    }

    // 3. Verify Total Volume
    // 4 pairs * 10 trades * 1000 volume = 40,000
    assert_eq!(service.get_total_volume(), 40_000);

    // 4. Verify Pair Volumes
    let btc_usdt = service.get_token_pair_volume("BTC", "USDT").unwrap();
    assert_eq!(btc_usdt.volume, 10_000);
    assert_eq!(btc_usdt.volume_history.len(), 10);

    // 5. Verify Time Windows
    let hourly_volume = service.get_volume_in_time_window("BTC", "USDT", TimeWindow::Hour).unwrap();
    assert_eq!(hourly_volume, 10_000);

    // 6. Verify Top Pairs
    // Add a huge trade to skew the top pairs
    service.record_trade_volume("BTC".to_string(), "USDT".to_string(), 50_000).unwrap();
    
    let top_pairs = service.get_top_pairs_by_volume(1);
    assert_eq!(top_pairs.len(), 1);
    assert_eq!(top_pairs[0].0, "BTC_USDT");
    assert_eq!(top_pairs[0].1.volume, 60_000);
}

#[test]
fn test_volume_tracker_persistence() {
    let tracker = VolumeTracker::new();
    tracker.record_trade_volume("PERSIST".to_string(), "TEST".to_string(), 12345).unwrap();

    let file_path = "volume_data_test.json";
    
    // Save
    tracker.save_to_file(file_path).unwrap();

    // Load
    let loaded_tracker = VolumeTracker::load_from_file(file_path).unwrap();

    // Verify
    let pair_volume = loaded_tracker.get_token_pair_volume("PERSIST", "TEST").unwrap();
    assert_eq!(pair_volume.volume, 12345);
    assert_eq!(loaded_tracker.get_total_volume(), 12345);

    // Cleanup
    std::fs::remove_file(file_path).unwrap();
}

#[test]
fn test_concurrent_volume_updates() {
    let service = Arc::new(AnalyticsService::new());
    let mut handles = vec![];

    // Spawn 10 threads, each recording 100 trades of 10 volume
    for _ in 0..10 {
        let service_clone = service.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                service_clone.record_trade_volume("CONC".to_string(), "TEST".to_string(), 10).unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Total should be 10 * 100 * 10 = 10,000
    assert_eq!(service.get_total_volume(), 10_000);
    
    let pair_volume = service.get_token_pair_volume("CONC", "TEST").unwrap();
    assert_eq!(pair_volume.volume, 10_000);
}
