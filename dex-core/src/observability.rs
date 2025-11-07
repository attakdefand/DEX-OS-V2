//! Observability implementation for the DEX-OS core engine
//!
//! This module implements the Priority 3 observability features from DEX-OS-V2.csv:
//! - Observability,Observability,Observability,Counter Metrics,Performance Monitoring,Medium
//! - Observability,Observability,Observability,Gauge Metrics,State Tracking,Medium
//! - Observability,Observability,Observability,Histogram Metrics,Latency Measurement,Medium

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Observability manager for metrics collection
#[derive(Debug)]
pub struct ObservabilityManager {
    /// Counter metrics
    counters: HashMap<String, Counter>,
    /// Gauge metrics
    gauges: HashMap<String, Gauge>,
    /// Histogram metrics
    histograms: HashMap<String, Histogram>,
}

/// Counter metric - monotonically increasing value
#[derive(Debug)]
pub struct Counter {
    /// The current value
    value: AtomicU64,
    /// Description of the counter
    description: String,
}

impl Clone for Counter {
    fn clone(&self) -> Self {
        Self {
            value: AtomicU64::new(self.value.load(Ordering::Relaxed)),
            description: self.description.clone(),
        }
    }
}

/// Gauge metric - current value that can go up or down
#[derive(Debug)]
pub struct Gauge {
    /// The current value
    value: AtomicI64,
    /// Description of the gauge
    description: String,
}

impl Clone for Gauge {
    fn clone(&self) -> Self {
        Self {
            value: AtomicI64::new(self.value.load(Ordering::Relaxed)),
            description: self.description.clone(),
        }
    }
}

/// Histogram metric - distribution of values
#[derive(Debug)]
pub struct Histogram {
    /// Buckets for histogram
    buckets: Vec<HistogramBucket>,
    /// Count of observations
    count: AtomicU64,
    /// Sum of all observations
    sum: AtomicU64,
    /// Description of the histogram
    description: String,
}

impl Clone for Histogram {
    fn clone(&self) -> Self {
        Self {
            buckets: self.buckets.clone(),
            count: AtomicU64::new(self.count.load(Ordering::Relaxed)),
            sum: AtomicU64::new(self.sum.load(Ordering::Relaxed)),
            description: self.description.clone(),
        }
    }
}

/// Bucket for histogram
#[derive(Debug)]
pub struct HistogramBucket {
    /// Upper bound of the bucket
    upper_bound: u64,
    /// Count of observations in this bucket
    count: AtomicU64,
}

impl Clone for HistogramBucket {
    fn clone(&self) -> Self {
        Self {
            upper_bound: self.upper_bound,
            count: AtomicU64::new(self.count.load(Ordering::Relaxed)),
        }
    }
}

/// Metrics collection for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Counter metrics
    pub counters: HashMap<String, (u64, String)>,
    /// Gauge metrics
    pub gauges: HashMap<String, (i64, String)>,
    /// Histogram metrics
    pub histograms: HashMap<String, HistogramSnapshot>,
}

/// Snapshot of a histogram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramSnapshot {
    /// Bucket data
    pub buckets: Vec<(u64, u64)>,
    /// Count of observations
    pub count: u64,
    /// Sum of all observations
    pub sum: u64,
    /// Description
    pub description: String,
}

impl ObservabilityManager {
    /// Create a new observability manager
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
        }
    }

    /// Register a new counter
    pub fn register_counter(&mut self, name: String, description: String) -> Result<(), ObservabilityError> {
        if self.counters.contains_key(&name) {
            return Err(ObservabilityError::MetricAlreadyExists);
        }
        
        self.counters.insert(name, Counter::new(description));
        Ok(())
    }

    /// Increment a counter
    pub fn increment_counter(&self, name: &str) -> Result<(), ObservabilityError> {
        if let Some(counter) = self.counters.get(name) {
            counter.increment();
            Ok(())
        } else {
            Err(ObservabilityError::MetricNotFound)
        }
    }

    /// Increment a counter by a specific value
    pub fn increment_counter_by(&self, name: &str, value: u64) -> Result<(), ObservabilityError> {
        if let Some(counter) = self.counters.get(name) {
            counter.increment_by(value);
            Ok(())
        } else {
            Err(ObservabilityError::MetricNotFound)
        }
    }

    /// Get counter value
    pub fn get_counter_value(&self, name: &str) -> Result<u64, ObservabilityError> {
        if let Some(counter) = self.counters.get(name) {
            Ok(counter.get_value())
        } else {
            Err(ObservabilityError::MetricNotFound)
        }
    }

    /// Register a new gauge
    pub fn register_gauge(&mut self, name: String, description: String) -> Result<(), ObservabilityError> {
        if self.gauges.contains_key(&name) {
            return Err(ObservabilityError::MetricAlreadyExists);
        }
        
        self.gauges.insert(name, Gauge::new(description));
        Ok(())
    }

    /// Set gauge value
    pub fn set_gauge(&self, name: &str, value: i64) -> Result<(), ObservabilityError> {
        if let Some(gauge) = self.gauges.get(name) {
            gauge.set(value);
            Ok(())
        } else {
            Err(ObservabilityError::MetricNotFound)
        }
    }

    /// Increment gauge value
    pub fn increment_gauge(&self, name: &str) -> Result<(), ObservabilityError> {
        if let Some(gauge) = self.gauges.get(name) {
            gauge.increment();
            Ok(())
        } else {
            Err(ObservabilityError::MetricNotFound)
        }
    }

    /// Decrement gauge value
    pub fn decrement_gauge(&self, name: &str) -> Result<(), ObservabilityError> {
        if let Some(gauge) = self.gauges.get(name) {
            gauge.decrement();
            Ok(())
        } else {
            Err(ObservabilityError::MetricNotFound)
        }
    }

    /// Get gauge value
    pub fn get_gauge_value(&self, name: &str) -> Result<i64, ObservabilityError> {
        if let Some(gauge) = self.gauges.get(name) {
            Ok(gauge.get_value())
        } else {
            Err(ObservabilityError::MetricNotFound)
        }
    }

    /// Register a new histogram
    pub fn register_histogram(&mut self, name: String, description: String, buckets: Vec<u64>) -> Result<(), ObservabilityError> {
        if self.histograms.contains_key(&name) {
            return Err(ObservabilityError::MetricAlreadyExists);
        }
        
        self.histograms.insert(name, Histogram::new(description, buckets));
        Ok(())
    }

    /// Record a value in a histogram
    pub fn record_histogram(&self, name: &str, value: u64) -> Result<(), ObservabilityError> {
        if let Some(histogram) = self.histograms.get(name) {
            histogram.observe(value);
            Ok(())
        } else {
            Err(ObservabilityError::MetricNotFound)
        }
    }

    /// Get histogram snapshot
    pub fn get_histogram_snapshot(&self, name: &str) -> Result<HistogramSnapshot, ObservabilityError> {
        if let Some(histogram) = self.histograms.get(name) {
            Ok(histogram.get_snapshot())
        } else {
            Err(ObservabilityError::MetricNotFound)
        }
    }

    /// Get a snapshot of all metrics
    pub fn get_metrics_snapshot(&self) -> MetricsSnapshot {
        let mut counters = HashMap::new();
        for (name, counter) in &self.counters {
            counters.insert(name.clone(), (counter.get_value(), counter.description.clone()));
        }
        
        let mut gauges = HashMap::new();
        for (name, gauge) in &self.gauges {
            gauges.insert(name.clone(), (gauge.get_value(), gauge.description.clone()));
        }
        
        let mut histograms = HashMap::new();
        for (name, histogram) in &self.histograms {
            histograms.insert(name.clone(), histogram.get_snapshot());
        }
        
        MetricsSnapshot {
            counters,
            gauges,
            histograms,
        }
    }
}

impl Default for ObservabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Counter {
    /// Create a new counter
    pub fn new(description: String) -> Self {
        Self {
            value: AtomicU64::new(0),
            description,
        }
    }

    /// Increment the counter by 1
    pub fn increment(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the counter by a specific value
    pub fn increment_by(&self, value: u64) {
        self.value.fetch_add(value, Ordering::Relaxed);
    }

    /// Get the current value
    pub fn get_value(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl Gauge {
    /// Create a new gauge
    pub fn new(description: String) -> Self {
        Self {
            value: AtomicI64::new(0),
            description,
        }
    }

    /// Set the gauge to a specific value
    pub fn set(&self, value: i64) {
        self.value.store(value, Ordering::Relaxed);
    }

    /// Increment the gauge by 1
    pub fn increment(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement the gauge by 1
    pub fn decrement(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get the current value
    pub fn get_value(&self) -> i64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl Histogram {
    /// Create a new histogram with specified buckets
    pub fn new(description: String, bucket_bounds: Vec<u64>) -> Self {
        let mut buckets = Vec::new();
        
        // Create buckets for each bound
        for bound in bucket_bounds {
            buckets.push(HistogramBucket {
                upper_bound: bound,
                count: AtomicU64::new(0),
            });
        }
        
        // Add an infinity bucket for values exceeding all bounds
        buckets.push(HistogramBucket {
            upper_bound: u64::MAX,
            count: AtomicU64::new(0),
        });
        
        Self {
            buckets,
            count: AtomicU64::new(0),
            sum: AtomicU64::new(0),
            description,
        }
    }

    /// Observe a value
    pub fn observe(&self, value: u64) {
        // Increment count and sum
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add(value, Ordering::Relaxed);
        
        // Find the appropriate bucket and increment its count
        for bucket in &self.buckets {
            if value <= bucket.upper_bound {
                bucket.count.fetch_add(1, Ordering::Relaxed);
                break;
            }
        }
    }

    /// Get a snapshot of the histogram
    pub fn get_snapshot(&self) -> HistogramSnapshot {
        let mut buckets = Vec::new();
        
        for bucket in &self.buckets {
            buckets.push((bucket.upper_bound, bucket.count.load(Ordering::Relaxed)));
        }
        
        HistogramSnapshot {
            buckets,
            count: self.count.load(Ordering::Relaxed),
            sum: self.sum.load(Ordering::Relaxed),
            description: self.description.clone(),
        }
    }
}

/// Timer for measuring latency
pub struct Timer {
    /// Start time
    start: SystemTime,
    /// Histogram to record to
    histogram: String,
    /// Observability manager reference
    manager: *const ObservabilityManager,
}

impl Timer {
    /// Create a new timer that will record to a histogram
    pub fn new(manager: &ObservabilityManager, histogram: String) -> Self {
        Self {
            start: SystemTime::now(),
            histogram,
            manager: manager as *const ObservabilityManager,
        }
    }

    /// Stop the timer and record the duration
    pub fn stop(self) {
        if let Ok(duration) = self.start.elapsed() {
            let duration_ms = duration.as_millis() as u64;
            
            // SAFETY: We only dereference the pointer if we know the manager is still alive
            // In this case, since the Timer is created with a reference to the manager,
            // the manager will outlive the Timer
            unsafe {
                if let Some(manager) = self.manager.as_ref() {
                    let _ = manager.record_histogram(&self.histogram, duration_ms);
                }
            }
        }
    }
}

/// Errors that can occur during observability operations
#[derive(Debug, Error)]
pub enum ObservabilityError {
    #[error("Metric already exists")]
    MetricAlreadyExists,
    #[error("Metric not found")]
    MetricNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_observability_manager_creation() {
        let manager = ObservabilityManager::new();
        assert!(manager.counters.is_empty());
        assert!(manager.gauges.is_empty());
        assert!(manager.histograms.is_empty());
    }

    #[test]
    fn test_counter_metrics() {
        let mut manager = ObservabilityManager::new();
        
        // Register a counter
        assert!(manager.register_counter("test_counter".to_string(), "A test counter".to_string()).is_ok());
        
        // Try to register the same counter again
        assert!(manager.register_counter("test_counter".to_string(), "Another test counter".to_string()).is_err());
        
        // Increment counter
        assert!(manager.increment_counter("test_counter").is_ok());
        assert_eq!(manager.get_counter_value("test_counter").unwrap(), 1);
        
        // Increment by specific value
        assert!(manager.increment_counter_by("test_counter", 5).is_ok());
        assert_eq!(manager.get_counter_value("test_counter").unwrap(), 6);
        
        // Try to increment non-existent counter
        assert!(manager.increment_counter("nonexistent").is_err());
    }

    #[test]
    fn test_gauge_metrics() {
        let mut manager = ObservabilityManager::new();
        
        // Register a gauge
        assert!(manager.register_gauge("test_gauge".to_string(), "A test gauge".to_string()).is_ok());
        
        // Set gauge value
        assert!(manager.set_gauge("test_gauge", 100).is_ok());
        assert_eq!(manager.get_gauge_value("test_gauge").unwrap(), 100);
        
        // Increment gauge
        assert!(manager.increment_gauge("test_gauge").is_ok());
        assert_eq!(manager.get_gauge_value("test_gauge").unwrap(), 101);
        
        // Decrement gauge
        assert!(manager.decrement_gauge("test_gauge").is_ok());
        assert_eq!(manager.get_gauge_value("test_gauge").unwrap(), 100);
        
        // Try to operate on non-existent gauge
        assert!(manager.set_gauge("nonexistent", 50).is_err());
    }

    #[test]
    fn test_histogram_metrics() {
        let mut manager = ObservabilityManager::new();
        
        // Register a histogram with buckets: 10, 50, 100, 500
        assert!(manager.register_histogram(
            "test_histogram".to_string(), 
            "A test histogram".to_string(),
            vec![10, 50, 100, 500]
        ).is_ok());
        
        // Record some values
        assert!(manager.record_histogram("test_histogram", 5).is_ok());   // Bucket 10
        assert!(manager.record_histogram("test_histogram", 25).is_ok());  // Bucket 50
        assert!(manager.record_histogram("test_histogram", 75).is_ok());  // Bucket 100
        assert!(manager.record_histogram("test_histogram", 300).is_ok()); // Bucket 500
        assert!(manager.record_histogram("test_histogram", 1000).is_ok()); // Bucket inf
        
        // Get histogram snapshot
        let snapshot = manager.get_histogram_snapshot("test_histogram").unwrap();
        assert_eq!(snapshot.count, 5);
        assert_eq!(snapshot.sum, 1405); // 5 + 25 + 75 + 300 + 1000
        
        // Check bucket counts
        assert_eq!(snapshot.buckets[0].1, 1); // Values <= 10
        assert_eq!(snapshot.buckets[1].1, 1); // Values <= 50
        assert_eq!(snapshot.buckets[2].1, 1); // Values <= 100
        assert_eq!(snapshot.buckets[3].1, 1); // Values <= 500
        assert_eq!(snapshot.buckets[4].1, 1); // Values <= inf
        
        // Try to record to non-existent histogram
        assert!(manager.record_histogram("nonexistent", 50).is_err());
    }

    #[test]
    fn test_metrics_snapshot() {
        let mut manager = ObservabilityManager::new();
        
        // Register metrics
        manager.register_counter("counter1".to_string(), "Test counter".to_string()).unwrap();
        manager.register_gauge("gauge1".to_string(), "Test gauge".to_string()).unwrap();
        manager.register_histogram("histogram1".to_string(), "Test histogram".to_string(), vec![10, 50]).unwrap();
        
        // Set some values
        manager.increment_counter("counter1").unwrap();
        manager.set_gauge("gauge1", 42).unwrap();
        manager.record_histogram("histogram1", 25).unwrap();
        
        // Get snapshot
        let snapshot = manager.get_metrics_snapshot();
        
        assert_eq!(snapshot.counters.len(), 1);
        assert_eq!(snapshot.gauges.len(), 1);
        assert_eq!(snapshot.histograms.len(), 1);
        
        assert_eq!(snapshot.counters.get("counter1").unwrap().0, 1);
        assert_eq!(snapshot.gauges.get("gauge1").unwrap().0, 42);
        assert_eq!(snapshot.histograms.get("histogram1").unwrap().count, 1);
    }

    #[test]
    fn test_timer() {
        let mut manager = ObservabilityManager::new();
        manager.register_histogram("latency".to_string(), "Latency histogram".to_string(), vec![100, 500, 1000]).unwrap();
        
        // Create and use timer
        let timer = Timer::new(&manager, "latency".to_string());
        
        // Sleep for a bit to simulate work
        thread::sleep(Duration::from_millis(10));
        
        // Stop timer (this records the duration)
        timer.stop();
        
        // Check that the histogram was updated
        let snapshot = manager.get_histogram_snapshot("latency").unwrap();
        assert_eq!(snapshot.count, 1);
        assert!(snapshot.sum > 0);
    }

    #[test]
    fn test_concurrent_access() {
        let mut manager = ObservabilityManager::new();
        manager.register_counter("concurrent_counter".to_string(), "Concurrent counter".to_string()).unwrap();
        let manager = Arc::new(Mutex::new(manager));
        
        // Spawn multiple threads to increment the counter
        let mut handles = vec![];
        
        for _ in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    manager_clone.lock().unwrap().increment_counter("concurrent_counter").unwrap();
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Check final counter value
        assert_eq!(manager.lock().unwrap().get_counter_value("concurrent_counter").unwrap(), 1000);
    }
}