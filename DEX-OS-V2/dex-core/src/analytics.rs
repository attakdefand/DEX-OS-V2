//! Analytics module for on-chain analytics and volume tracking
//!
//! This module implements the Priority 5 feature from DEX-OS-V2.csv:
//! - Analytics & Oracles,On-Chain Analytics,On-Chain Analytics,Volume/Volume Trackers,Volume Tracking,Medium {Security: Layer 4 - Application Security}

use crate::network::MessageBroker;
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Analytics service that provides volume tracking and other analytics
#[derive(Debug, Clone)]
pub struct AnalyticsService {
    /// Volume tracker for monitoring trading volumes
    pub volume_tracker: VolumeTracker,
    /// Analytics configuration
    config: AnalyticsConfig,
    /// Optional message broker for publishing analytics events
    message_broker: Option<MessageBroker>,
}

/// Configuration for analytics service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// Whether to enable volume tracking
    pub enable_volume_tracking: bool,
    /// Maximum history entries to keep
    pub max_history_entries: usize,
    /// Enable time window tracking
    pub enable_time_windows: bool,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enable_volume_tracking: true,
            max_history_entries: 1000,
            enable_time_windows: true,
        }
    }
}

impl AnalyticsService {
    /// Create a new analytics service
    pub fn new() -> Self {
        Self::with_config(AnalyticsConfig::default())
    }

    /// Create a new analytics service with configuration
    pub fn with_config(config: AnalyticsConfig) -> Self {
        Self {
            volume_tracker: VolumeTracker::with_config(config.clone()),
            config,
            message_broker: None,
        }
    }

    /// Create a new analytics service with an attached message broker for Pub-Sub.
    pub fn with_pubsub(config: AnalyticsConfig, message_broker: MessageBroker) -> Self {
        Self {
            volume_tracker: VolumeTracker::with_config(config.clone()),
            config,
            message_broker: Some(message_broker),
        }
    }

    /// Attach or replace the message broker after initialization.
    pub fn set_message_broker(&mut self, message_broker: MessageBroker) {
        self.message_broker = Some(message_broker);
    }

    /// Record a trade volume
    pub fn record_trade_volume(
        &self,
        base_token: String,
        quote_token: String,
        volume: u64,
    ) -> Result<(), VolumeTrackingError> {
        let payload_base = base_token.clone();
        let payload_quote = quote_token.clone();
        self.volume_tracker
            .record_trade_volume(base_token, quote_token, volume)
            .map(|_| {
                if self.config.enable_volume_tracking {
                    if let Some(broker) = &self.message_broker {
                        let broker = broker.clone();
                        let payload = json!({
                            "base_token": payload_base,
                            "quote_token": payload_quote,
                            "volume": volume,
                            "timestamp": crate::analytics::current_timestamp(),
                        });
                        // Fire-and-forget publish; broker handles fan-out.
                        tokio::spawn(async move {
                            let _ = broker.publish("analytics.volume", payload).await;
                        });
                    }
                }
            })
    }

    /// Get volume for a specific token pair
    pub fn get_token_pair_volume(
        &self,
        base_token: &str,
        quote_token: &str,
    ) -> Result<TokenPairVolume, VolumeTrackingError> {
        self.volume_tracker
            .get_token_pair_volume(base_token, quote_token)
    }

    /// Get total volume across all pairs
    pub fn get_total_volume(&self) -> u64 {
        self.volume_tracker.get_total_volume()
    }

    /// Get volume for a specific token pair in a time window
    pub fn get_volume_in_time_window(
        &self,
        base_token: &str,
        quote_token: &str,
        window: TimeWindow,
    ) -> Result<u64, VolumeTrackingError> {
        self.volume_tracker
            .get_volume_in_time_window(base_token, quote_token, window)
    }

    /// Get top N token pairs by volume
    pub fn get_top_pairs_by_volume(&self, n: usize) -> Vec<(String, TokenPairVolume)> {
        self.volume_tracker.get_top_pairs_by_volume(n)
    }

    /// Export all analytics data as JSON
    pub fn export_as_json(&self) -> Result<String, VolumeTrackingError> {
        self.volume_tracker.export_as_json()
    }
}

/// Volume tracker for monitoring trading volumes
#[derive(Debug, Clone)]
pub struct VolumeTracker {
    /// Token pair volumes
    token_pair_volumes: Arc<Mutex<HashMap<String, TokenPairVolume>>>,
    /// Total volume across all pairs
    total_volume: Arc<AtomicU64>,
    /// Volume by time windows
    time_window_volumes: Arc<Mutex<HashMap<TimeWindow, HashMap<String, u64>>>>,
    /// Configuration
    config: AnalyticsConfig,
}

/// Volume data for a specific token pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPairVolume {
    /// Base token symbol
    pub base_token: String,
    /// Quote token symbol
    pub quote_token: String,
    /// Total volume (in quote token terms)
    pub volume: u64,
    /// Volume history by timestamp
    pub volume_history: Vec<VolumeRecord>,
    /// Last update timestamp
    pub last_updated: u64,
}

/// Volume record for a specific time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeRecord {
    /// Timestamp of the record
    pub timestamp: u64,
    /// Volume in this period
    pub volume: u64,
}

/// Time window for volume tracking
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeWindow {
    /// 1 hour window
    Hour,
    /// 1 day window
    Day,
    /// 1 week window
    Week,
    /// 1 month window
    Month,
}

/// Volume tracking error
#[derive(Debug, Error)]
pub enum VolumeTrackingError {
    #[error("Token pair not found")]
    TokenPairNotFound,
    #[error("Invalid time window")]
    InvalidTimeWindow,
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl VolumeTracker {
    /// Create a new volume tracker
    pub fn new() -> Self {
        Self::with_config(AnalyticsConfig::default())
    }

    /// Create a new volume tracker with configuration
    pub fn with_config(config: AnalyticsConfig) -> Self {
        Self {
            token_pair_volumes: Arc::new(Mutex::new(HashMap::new())),
            total_volume: Arc::new(AtomicU64::new(0)),
            time_window_volumes: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Record a trade volume for a token pair
    pub fn record_trade_volume(
        &self,
        base_token: String,
        quote_token: String,
        volume: u64,
    ) -> Result<(), VolumeTrackingError> {
        // Check if volume tracking is enabled
        if !self.config.enable_volume_tracking {
            return Ok(());
        }

        let pair_key = format!("{}_{}", base_token, quote_token);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| VolumeTrackingError::SerializationError("Failed to get timestamp".into()))?
            .as_secs();

        // Update token pair volume
        {
            let mut volumes = self.token_pair_volumes.lock().unwrap();
            let pair_volume = volumes
                .entry(pair_key.clone())
                .or_insert_with(|| TokenPairVolume {
                    base_token: base_token.clone(),
                    quote_token: quote_token.clone(),
                    volume: 0,
                    volume_history: Vec::new(),
                    last_updated: timestamp,
                });

            pair_volume.volume += volume;
            pair_volume.last_updated = timestamp;

            // Add to history, but limit the size
            pair_volume
                .volume_history
                .push(VolumeRecord { timestamp, volume });

            // Trim history if needed
            if pair_volume.volume_history.len() > self.config.max_history_entries {
                let excess = pair_volume.volume_history.len() - self.config.max_history_entries;
                pair_volume.volume_history.drain(0..excess);
            }
        }

        // Update total volume
        self.total_volume.fetch_add(volume, Ordering::Relaxed);

        // Update time window volumes if enabled
        if self.config.enable_time_windows {
            self.update_time_window_volumes(&pair_key, volume, timestamp)?;
        }

        Ok(())
    }

    /// Get volume for a specific token pair
    pub fn get_token_pair_volume(
        &self,
        base_token: &str,
        quote_token: &str,
    ) -> Result<TokenPairVolume, VolumeTrackingError> {
        let pair_key = format!("{}_{}", base_token, quote_token);
        let volumes = self.token_pair_volumes.lock().unwrap();

        volumes
            .get(&pair_key)
            .cloned()
            .ok_or(VolumeTrackingError::TokenPairNotFound)
    }

    /// Get total volume across all pairs
    pub fn get_total_volume(&self) -> u64 {
        self.total_volume.load(Ordering::Relaxed)
    }

    /// Get volume for a specific token pair in a time window
    pub fn get_volume_in_time_window(
        &self,
        base_token: &str,
        quote_token: &str,
        window: TimeWindow,
    ) -> Result<u64, VolumeTrackingError> {
        let pair_key = format!("{}_{}", base_token, quote_token);
        let time_windows = self.time_window_volumes.lock().unwrap();

        if let Some(window_volumes) = time_windows.get(&window) {
            Ok(*window_volumes.get(&pair_key).unwrap_or(&0))
        } else {
            Ok(0)
        }
    }

    /// Get top N token pairs by volume
    pub fn get_top_pairs_by_volume(&self, n: usize) -> Vec<(String, TokenPairVolume)> {
        let volumes = self.token_pair_volumes.lock().unwrap();
        let mut pairs: Vec<_> = volumes
            .iter()
            .map(|(key, volume)| (key.clone(), volume.clone()))
            .collect();

        // Sort by volume descending
        pairs.sort_by(|a, b| b.1.volume.cmp(&a.1.volume));

        // Take top N
        pairs.into_iter().take(n).collect()
    }

    /// Update time window volumes
    fn update_time_window_volumes(
        &self,
        pair_key: &str,
        volume: u64,
        timestamp: u64,
    ) -> Result<(), VolumeTrackingError> {
        let mut time_windows = self.time_window_volumes.lock().unwrap();

        // Update hourly volume
        *time_windows
            .entry(TimeWindow::Hour)
            .or_insert_with(HashMap::new)
            .entry(pair_key.to_string())
            .or_insert(0) += volume;

        // Update daily volume
        *time_windows
            .entry(TimeWindow::Day)
            .or_insert_with(HashMap::new)
            .entry(pair_key.to_string())
            .or_insert(0) += volume;

        // Update weekly volume
        *time_windows
            .entry(TimeWindow::Week)
            .or_insert_with(HashMap::new)
            .entry(pair_key.to_string())
            .or_insert(0) += volume;

        // Update monthly volume
        *time_windows
            .entry(TimeWindow::Month)
            .or_insert_with(HashMap::new)
            .entry(pair_key.to_string())
            .or_insert(0) += volume;

        Ok(())
    }

    /// Reset volumes for a specific time window (typically called when window expires)
    pub fn reset_time_window(&self, window: TimeWindow) -> Result<(), VolumeTrackingError> {
        let mut time_windows = self.time_window_volumes.lock().unwrap();
        time_windows.insert(window, HashMap::new());
        Ok(())
    }

    /// Export volume data as JSON
    pub fn export_as_json(&self) -> Result<String, VolumeTrackingError> {
        let volumes = self.token_pair_volumes.lock().unwrap();
        serde_json::to_string(&*volumes)
            .map_err(|e| VolumeTrackingError::SerializationError(e.to_string()))
    }

    /// Save volume data to a file
    pub fn save_to_file(&self, path: &str) -> Result<(), VolumeTrackingError> {
        let json = self.export_as_json()?;
        std::fs::write(path, json)
            .map_err(|e| VolumeTrackingError::SerializationError(format!("Failed to write file: {}", e)))
    }

    /// Load volume data from a file
    pub fn load_from_file(path: &str) -> Result<Self, VolumeTrackingError> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| VolumeTrackingError::SerializationError(format!("Failed to read file: {}", e)))?;
        
        let volumes: HashMap<String, TokenPairVolume> = serde_json::from_str(&json)
            .map_err(|e| VolumeTrackingError::SerializationError(format!("Failed to parse JSON: {}", e)))?;

        let mut total_volume = 0;
        for volume in volumes.values() {
            total_volume += volume.volume;
        }

        let tracker = Self::new();
        *tracker.token_pair_volumes.lock().unwrap() = volumes;
        tracker.total_volume.store(total_volume, Ordering::Relaxed);
        
        // Reconstruct time windows (simplified - just using history)
        // In a real implementation, we might want to serialize time windows too, 
        // or reconstruct them from history if history is deep enough.
        // For now, we'll leave time windows empty until new trades come in, 
        // or we could iterate history to repopulate.
        
        Ok(tracker)
    }

    /// Get analytics configuration
    pub fn get_config(&self) -> &AnalyticsConfig {
        &self.config
    }
}

impl Default for VolumeTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AnalyticsService {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_tracker_creation() {
        let tracker = VolumeTracker::new();
        assert_eq!(tracker.get_total_volume(), 0);
    }

    #[test]
    fn test_record_trade_volume() {
        let tracker = VolumeTracker::new();

        // Record a trade
        assert!(tracker
            .record_trade_volume("BTC".to_string(), "USDT".to_string(), 1000)
            .is_ok());

        // Check volume was recorded
        assert_eq!(tracker.get_total_volume(), 1000);

        // Check token pair volume
        let pair_volume = tracker.get_token_pair_volume("BTC", "USDT").unwrap();
        assert_eq!(pair_volume.volume, 1000);
        assert_eq!(pair_volume.base_token, "BTC");
        assert_eq!(pair_volume.quote_token, "USDT");
        assert_eq!(pair_volume.volume_history.len(), 1);
        assert_eq!(pair_volume.volume_history[0].volume, 1000);
    }

    #[test]
    fn test_multiple_trades_same_pair() {
        let tracker = VolumeTracker::new();

        // Record multiple trades for the same pair
        tracker
            .record_trade_volume("ETH".to_string(), "USDT".to_string(), 500)
            .unwrap();
        tracker
            .record_trade_volume("ETH".to_string(), "USDT".to_string(), 300)
            .unwrap();
        tracker
            .record_trade_volume("ETH".to_string(), "USDT".to_string(), 200)
            .unwrap();

        // Check cumulative volume
        assert_eq!(tracker.get_total_volume(), 1000);

        let pair_volume = tracker.get_token_pair_volume("ETH", "USDT").unwrap();
        assert_eq!(pair_volume.volume, 1000);
        assert_eq!(pair_volume.volume_history.len(), 3);
    }

    #[test]
    fn test_multiple_token_pairs() {
        let tracker = VolumeTracker::new();

        // Record trades for different pairs
        tracker
            .record_trade_volume("BTC".to_string(), "USDT".to_string(), 10000)
            .unwrap();
        tracker
            .record_trade_volume("ETH".to_string(), "USDT".to_string(), 5000)
            .unwrap();
        tracker
            .record_trade_volume("SOL".to_string(), "USDT".to_string(), 2000)
            .unwrap();

        // Check total volume
        assert_eq!(tracker.get_total_volume(), 17000);

        // Check individual pair volumes
        assert_eq!(
            tracker.get_token_pair_volume("BTC", "USDT").unwrap().volume,
            10000
        );
        assert_eq!(
            tracker.get_token_pair_volume("ETH", "USDT").unwrap().volume,
            5000
        );
        assert_eq!(
            tracker.get_token_pair_volume("SOL", "USDT").unwrap().volume,
            2000
        );
    }

    #[test]
    fn test_top_pairs_by_volume() {
        let tracker = VolumeTracker::new();

        // Record trades with different volumes
        tracker
            .record_trade_volume("LOW".to_string(), "USDT".to_string(), 100)
            .unwrap();
        tracker
            .record_trade_volume("HIGH".to_string(), "USDT".to_string(), 10000)
            .unwrap();
        tracker
            .record_trade_volume("MED".to_string(), "USDT".to_string(), 1000)
            .unwrap();

        // Get top 2 pairs
        let top_pairs = tracker.get_top_pairs_by_volume(2);
        assert_eq!(top_pairs.len(), 2);

        // First should be HIGH pair (highest volume)
        assert!(top_pairs[0].0.contains("HIGH"));
        assert_eq!(top_pairs[0].1.volume, 10000);

        // Second should be MED pair (medium volume)
        assert!(top_pairs[1].0.contains("MED"));
        assert_eq!(top_pairs[1].1.volume, 1000);
    }

    #[test]
    fn test_volume_in_time_window() {
        let tracker = VolumeTracker::new();

        // Record a trade
        tracker
            .record_trade_volume("BTC".to_string(), "USDT".to_string(), 5000)
            .unwrap();

        // Check volume in different time windows
        assert_eq!(
            tracker
                .get_volume_in_time_window("BTC", "USDT", TimeWindow::Hour)
                .unwrap(),
            5000
        );
        assert_eq!(
            tracker
                .get_volume_in_time_window("BTC", "USDT", TimeWindow::Day)
                .unwrap(),
            5000
        );
        assert_eq!(
            tracker
                .get_volume_in_time_window("BTC", "USDT", TimeWindow::Week)
                .unwrap(),
            5000
        );
        assert_eq!(
            tracker
                .get_volume_in_time_window("BTC", "USDT", TimeWindow::Month)
                .unwrap(),
            5000
        );

        // Check non-existent pair
        assert_eq!(
            tracker
                .get_volume_in_time_window("NON", "EXIST", TimeWindow::Hour)
                .unwrap(),
            0
        );
    }

    #[test]
    fn test_reset_time_window() {
        let tracker = VolumeTracker::new();

        // Record a trade
        tracker
            .record_trade_volume("BTC".to_string(), "USDT".to_string(), 5000)
            .unwrap();

        // Check volume exists
        assert_eq!(
            tracker
                .get_volume_in_time_window("BTC", "USDT", TimeWindow::Hour)
                .unwrap(),
            5000
        );

        // Reset hourly window
        tracker.reset_time_window(TimeWindow::Hour).unwrap();

        // Check volume was reset
        assert_eq!(
            tracker
                .get_volume_in_time_window("BTC", "USDT", TimeWindow::Hour)
                .unwrap(),
            0
        );
    }

    #[test]
    fn test_export_as_json() {
        let tracker = VolumeTracker::new();

        // Record some trades
        tracker
            .record_trade_volume("BTC".to_string(), "USDT".to_string(), 10000)
            .unwrap();
        tracker
            .record_trade_volume("ETH".to_string(), "USDT".to_string(), 5000)
            .unwrap();

        // Export as JSON
        let json = tracker.export_as_json().unwrap();
        assert!(!json.is_empty());
        assert!(json.contains("BTC_USDT"));
        assert!(json.contains("ETH_USDT"));
    }

    #[test]
    fn test_analytics_service_creation() {
        let service = AnalyticsService::new();
        assert_eq!(service.get_total_volume(), 0);
    }

    #[test]
    fn test_analytics_service_record_trade() {
        let service = AnalyticsService::new();

        // Record a trade through the service
        assert!(service
            .record_trade_volume("BTC".to_string(), "USDT".to_string(), 2000)
            .is_ok());

        // Check volume was recorded
        assert_eq!(service.get_total_volume(), 2000);

        // Check token pair volume
        let pair_volume = service.get_token_pair_volume("BTC", "USDT").unwrap();
        assert_eq!(pair_volume.volume, 2000);
    }

    #[test]
    fn test_analytics_service_with_config() {
        let config = AnalyticsConfig {
            enable_volume_tracking: false,
            max_history_entries: 500,
            enable_time_windows: false,
        };
        let service = AnalyticsService::with_config(config);

        // Recording should succeed but not track when disabled
        assert!(service
            .record_trade_volume("BTC".to_string(), "USDT".to_string(), 1000)
            .is_ok());
        assert_eq!(service.get_total_volume(), 0);

        // Getting non-existent pair should fail
        assert!(service.get_token_pair_volume("BTC", "USDT").is_err());
    }

    #[test]
    fn test_volume_history_limiting() {
        let config = AnalyticsConfig {
            enable_volume_tracking: true,
            max_history_entries: 2,
            enable_time_windows: true,
        };
        let tracker = VolumeTracker::with_config(config);

        // Record more trades than the history limit
        for i in 0..5 {
            tracker
                .record_trade_volume("TEST".to_string(), "TOKEN".to_string(), 100 * (i + 1))
                .unwrap();
        }

        // Check that history is limited
        let pair_volume = tracker.get_token_pair_volume("TEST", "TOKEN").unwrap();
        assert_eq!(pair_volume.volume_history.len(), 2);
        // Should have the last 2 entries
        assert_eq!(pair_volume.volume_history[0].volume, 400); // 4th trade (100*4)
        assert_eq!(pair_volume.volume_history[1].volume, 500); // 5th trade (100*5)
        assert_eq!(pair_volume.volume, 1500); // Sum of all trades (100+200+300+400+500)
    }

    #[test]
    fn test_time_windows_disabled() {
        let config = AnalyticsConfig {
            enable_volume_tracking: true,
            max_history_entries: 1000,
            enable_time_windows: false,
        };
        let tracker = VolumeTracker::with_config(config);

        // Record a trade
        tracker
            .record_trade_volume("BTC".to_string(), "USDT".to_string(), 1000)
            .unwrap();

        // Time window query should return 0 when disabled
        assert_eq!(
            tracker
                .get_volume_in_time_window("BTC", "USDT", TimeWindow::Hour)
                .unwrap(),
            0
        );
    }

    #[test]
    fn test_volume_tracker_with_custom_config() {
        let config = AnalyticsConfig {
            enable_volume_tracking: true,
            max_history_entries: 50,
            enable_time_windows: true,
        };
        let tracker = VolumeTracker::with_config(config.clone());

        // Check that config is stored correctly
        assert_eq!(tracker.get_config().max_history_entries, 50);
        assert_eq!(tracker.get_config().enable_volume_tracking, true);
        assert_eq!(tracker.get_config().enable_time_windows, true);
    }

    #[test]
    fn test_analytics_service_default() {
        let service = AnalyticsService::default();
        assert_eq!(service.get_total_volume(), 0);
    }

    #[test]
    fn test_time_window_enum() {
        // Test that time windows can be created and compared
        let hour = TimeWindow::Hour;
        let day = TimeWindow::Day;

        assert_ne!(hour, day);

        // Test cloning
        let hour_clone = hour.clone();
        assert_eq!(hour, hour_clone);
    }

    #[test]
    fn test_volume_tracking_error_display() {
        let error = VolumeTrackingError::TokenPairNotFound;
        let error_str = format!("{}", error);
        assert_eq!(error_str, "Token pair not found");

        let error = VolumeTrackingError::InvalidTimeWindow;
        let error_str = format!("{}", error);
        assert_eq!(error_str, "Invalid time window");

        let error = VolumeTrackingError::SerializationError("test error".to_string());
        let error_str = format!("{}", error);
        assert!(error_str.contains("Serialization error"));
        assert!(error_str.contains("test error"));
    }
}
