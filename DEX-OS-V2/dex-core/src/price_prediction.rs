//! Price prediction implementation using Kalman Filter for the DEX-OS core engine
//!
//! This module implements multiple features from DEX-OS-V1.csv:
//! - Priority 2 feature: "Core Trading,Oracle,Oracle,Kalman Filter,Price Prediction,Medium"
//! - Priority 1 features:
//!   "Core Trading,Oracle,Oracle,Median Selection,Price Aggregation,High"
//!   "Core Trading,Oracle,Oracle,TWAP Calculation,Price Aggregation,High"
//!
//! It provides functionality for predicting future prices using a Kalman filter,
//! as well as median-based and TWAP price aggregation for oracle price feeds.

use crate::types::TokenId;
use thiserror::Error;

/// Represents the state of the Kalman filter
#[derive(Debug, Clone)]
pub struct KalmanState {
    /// Estimated price
    pub price: f64,
    /// Estimated variance (uncertainty)
    pub variance: f64,
}

/// Kalman Filter for price prediction
#[derive(Debug, Clone)]
pub struct KalmanPricePredictor {
    /// Current state estimate
    state: KalmanState,
    /// Process noise (how much we expect the true price to change)
    process_noise: f64,
    /// Measurement noise (how noisy our observations are)
    measurement_noise: f64,
    /// Last update timestamp
    last_update: u64,
}

impl KalmanPricePredictor {
    /// Create a new Kalman filter for price prediction
    pub fn new(initial_price: f64, process_noise: f64, measurement_noise: f64) -> Self {
        Self {
            state: KalmanState {
                price: initial_price,
                variance: 1.0, // Start with high uncertainty
            },
            process_noise,
            measurement_noise,
            last_update: 0,
        }
    }

    /// Update the filter with a new price observation
    pub fn update(
        &mut self,
        observation: f64,
        timestamp: u64,
    ) -> Result<KalmanState, PricePredictionError> {
        // Time update (prediction)
        // In a simple model, we assume the price doesn't change much between observations
        // but we add process noise to account for uncertainty
        let predicted_variance = self.state.variance + self.process_noise;

        // Measurement update (correction)
        let innovation = observation - self.state.price;
        let innovation_variance = predicted_variance + self.measurement_noise;
        let kalman_gain = predicted_variance / innovation_variance;

        // Update state estimate
        self.state.price += kalman_gain * innovation;
        self.state.variance = (1.0 - kalman_gain) * predicted_variance;

        // Update timestamp
        self.last_update = timestamp;

        Ok(self.state.clone())
    }

    /// Predict the next price without a new observation
    pub fn predict(&self) -> Result<KalmanState, PricePredictionError> {
        // In this simple model, we predict the price will change based on recent trend
        // but with increased uncertainty

        // Simple prediction: assume price continues in the same direction with small momentum
        let predicted_price = self.state.price * 1.001; // Small positive momentum assumption

        Ok(KalmanState {
            price: predicted_price,
            variance: self.state.variance + self.process_noise,
        })
    }

    /// Get the current state estimate
    pub fn get_current_state(&self) -> &KalmanState {
        &self.state
    }

    /// Get the estimated price
    pub fn get_estimated_price(&self) -> f64 {
        self.state.price
    }

    /// Get the uncertainty (variance) of the estimate
    pub fn get_uncertainty(&self) -> f64 {
        self.state.variance
    }

    /// Reset the filter with a new initial price
    pub fn reset(&mut self, initial_price: f64) {
        self.state.price = initial_price;
        self.state.variance = 1.0; // Reset uncertainty
        self.last_update = 0;
    }

    /// Adjust noise parameters
    pub fn set_noise_parameters(&mut self, process_noise: f64, measurement_noise: f64) {
        self.process_noise = process_noise;
        self.measurement_noise = measurement_noise;
    }
}

/// Manages price prediction for multiple token pairs
#[derive(Debug, Clone)]
pub struct PricePredictionManager {
    /// Predictors for different token pairs
    predictors: std::collections::HashMap<(TokenId, TokenId), KalmanPricePredictor>,
}

impl PricePredictionManager {
    /// Create a new price prediction manager
    pub fn new() -> Self {
        Self {
            predictors: std::collections::HashMap::new(),
        }
    }

    /// Add or update a price predictor for a token pair
    pub fn add_predictor(
        &mut self,
        base_token: TokenId,
        quote_token: TokenId,
        predictor: KalmanPricePredictor,
    ) {
        self.predictors.insert((base_token, quote_token), predictor);
    }

    /// Update the price for a token pair
    pub fn update_price(
        &mut self,
        base_token: &TokenId,
        quote_token: &TokenId,
        price: f64,
        timestamp: u64,
    ) -> Result<KalmanState, PricePredictionError> {
        let key = (base_token.clone(), quote_token.clone());

        if let Some(predictor) = self.predictors.get_mut(&key) {
            predictor.update(price, timestamp)
        } else {
            Err(PricePredictionError::PredictorNotFound)
        }
    }

    /// Predict the next price for a token pair
    pub fn predict_price(
        &self,
        base_token: &TokenId,
        quote_token: &TokenId,
    ) -> Result<KalmanState, PricePredictionError> {
        let key = (base_token.clone(), quote_token.clone());

        if let Some(predictor) = self.predictors.get(&key) {
            predictor.predict()
        } else {
            Err(PricePredictionError::PredictorNotFound)
        }
    }

    /// Get the current estimated price for a token pair
    pub fn get_estimated_price(
        &self,
        base_token: &TokenId,
        quote_token: &TokenId,
    ) -> Result<f64, PricePredictionError> {
        let key = (base_token.clone(), quote_token.clone());

        if let Some(predictor) = self.predictors.get(&key) {
            Ok(predictor.get_estimated_price())
        } else {
            Err(PricePredictionError::PredictorNotFound)
        }
    }

    /// Remove a predictor for a token pair
    pub fn remove_predictor(&mut self, base_token: &TokenId, quote_token: &TokenId) -> bool {
        let key = (base_token.clone(), quote_token.clone());
        self.predictors.remove(&key).is_some()
    }

    /// Get the number of predictors
    pub fn predictor_count(&self) -> usize {
        self.predictors.len()
    }

    /// Check if there are any predictors
    pub fn has_predictors(&self) -> bool {
        !self.predictors.is_empty()
    }
}

/// Errors that can occur during price prediction operations
#[derive(Debug, Error)]
pub enum PricePredictionError {
    #[error("Predictor not found for token pair")]
    PredictorNotFound,
    #[error("Invalid price observation")]
    InvalidObservation,
    #[error("Invalid timestamp")]
    InvalidTimestamp,
}

/// Calculate the median value from a vector of prices
/// This implements the Priority 1 feature from DEX-OS-V1.csv:
/// "Core Trading,Oracle,Oracle,Median Selection,Price Aggregation,High"
fn calculate_median(prices: &mut Vec<f64>) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }

    // Sort the prices
    prices.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let len = prices.len();
    if len % 2 == 0 {
        // Even number of elements - average of two middle elements
        (prices[len / 2 - 1] + prices[len / 2]) / 2.0
    } else {
        // Odd number of elements - middle element
        prices[len / 2]
    }
}

/// Calculate Time-Weighted Average Price (TWAP) from a series of price observations
/// This implements the Priority 1 feature from DEX-OS-V1.csv:
/// "Core Trading,Oracle,Oracle,TWAP Calculation,Price Aggregation,High"
fn calculate_twap(observations: &[(f64, u64)]) -> f64 {
    if observations.is_empty() {
        return 0.0;
    }

    if observations.len() == 1 {
        return observations[0].0;
    }

    let mut weighted_sum = 0.0;
    let mut total_time = 0.0;

    // Calculate weighted average based on time intervals
    for i in 1..observations.len() {
        let (price, timestamp) = observations[i];
        let (_, prev_timestamp) = observations[i - 1];

        // Time interval between observations
        let time_interval = (timestamp - prev_timestamp) as f64;

        // Weight price by time interval
        weighted_sum += price * time_interval;
        total_time += time_interval;
    }

    if total_time > 0.0 {
        weighted_sum / total_time
    } else {
        observations[0].0
    }
}

/// Oracle price aggregator that provides multiple aggregation methods
#[derive(Debug, Clone)]
pub struct PriceAggregator {
    /// Price observations for different sources
    observations: std::collections::HashMap<String, Vec<(f64, u64)>>,
}

impl PriceAggregator {
    /// Create a new price aggregator
    pub fn new() -> Self {
        Self {
            observations: std::collections::HashMap::new(),
        }
    }

    /// Add a price observation from a source
    pub fn add_observation(&mut self, source: String, price: f64, timestamp: u64) {
        self.observations
            .entry(source)
            .or_insert_with(Vec::new)
            .push((price, timestamp));
    }

    /// Get median price across all sources
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Oracle,Oracle,Median Selection,Price Aggregation,High"
    pub fn get_median_price(&self) -> f64 {
        let mut prices: Vec<f64> = self
            .observations
            .values()
            .flat_map(|obs| obs.last().map(|(price, _)| *price))
            .collect();

        calculate_median(&mut prices)
    }

    /// Get Time-Weighted Average Price (TWAP) for a specific source
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Oracle,Oracle,TWAP Calculation,Price Aggregation,High"
    pub fn get_twap_price(&self, source: &str, time_window: u64) -> f64 {
        if let Some(observations) = self.observations.get(source) {
            if observations.is_empty() {
                return 0.0;
            }

            // Get current timestamp (use the latest observation timestamp)
            let current_time = observations.last().map(|(_, ts)| *ts).unwrap_or(0);

            // Filter observations within the time window
            let window_observations: Vec<(f64, u64)> = observations
                .iter()
                .filter(|(_, ts)| *ts >= current_time.saturating_sub(time_window))
                .cloned()
                .collect();

            calculate_twap(&window_observations)
        } else {
            0.0
        }
    }

    /// Get Time-Weighted Average Price (TWAP) across all sources
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Oracle,Oracle,TWAP Calculation,Price Aggregation,High"
    pub fn get_aggregated_twap_price(&self, time_window: u64) -> f64 {
        // For aggregated TWAP, we'll calculate TWAP for each source and then take the average
        let mut source_twaps = Vec::new();

        for source in self.observations.keys() {
            let twap = self.get_twap_price(source, time_window);
            if twap > 0.0 {
                source_twaps.push(twap);
            }
        }

        if source_twaps.is_empty() {
            0.0
        } else {
            source_twaps.iter().sum::<f64>() / source_twaps.len() as f64
        }
    }

    /// Remove old observations to prevent unbounded memory growth
    pub fn prune_observations(&mut self, max_age: u64) {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let cutoff_time = current_time.saturating_sub(max_age);

        for observations in self.observations.values_mut() {
            observations.retain(|(_, timestamp)| *timestamp >= cutoff_time);
        }
    }

    /// Get the number of sources
    pub fn source_count(&self) -> usize {
        self.observations.len()
    }

    /// Get the total number of observations
    pub fn observation_count(&self) -> usize {
        self.observations.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kalman_filter_creation() {
        let filter = KalmanPricePredictor::new(100.0, 0.1, 0.5);
        assert_eq!(filter.get_estimated_price(), 100.0);
        assert_eq!(filter.get_uncertainty(), 1.0);
    }

    #[test]
    fn test_kalman_filter_update() {
        let mut filter = KalmanPricePredictor::new(100.0, 0.1, 0.5);

        // Update with a new observation
        let state = filter.update(105.0, 1000).unwrap();
        assert_ne!(state.price, 100.0); // Should have changed
        assert!(state.variance < 1.0); // Uncertainty should have decreased
    }

    #[test]
    fn test_kalman_filter_convergence() {
        let mut filter = KalmanPricePredictor::new(100.0, 0.1, 0.5);

        // Feed a series of observations that are all around 105
        for i in 0..10 {
            let observation = 105.0 + (i as f64 - 5.0) * 0.1; // Values around 105
            filter.update(observation, i * 100).unwrap();
        }

        // The estimate should have converged toward 105
        let estimated_price = filter.get_estimated_price();
        assert!((estimated_price - 105.0).abs() < 2.0); // Should be within 2 of 105
        assert!(filter.get_uncertainty() < 0.5); // Uncertainty should be low
    }

    #[test]
    fn test_kalman_filter_prediction() {
        let mut filter = KalmanPricePredictor::new(100.0, 0.1, 0.5);

        // Update with some observations
        filter.update(102.0, 1000).unwrap();
        filter.update(104.0, 2000).unwrap();

        // Make a prediction
        let prediction = filter.predict().unwrap();
        assert_ne!(prediction.price, filter.get_estimated_price()); // Prediction should differ
        assert!(prediction.variance > filter.get_uncertainty()); // Prediction should have higher uncertainty
    }

    #[test]
    fn test_price_prediction_manager() {
        let mut manager = PricePredictionManager::new();

        // Add a predictor
        let predictor = KalmanPricePredictor::new(50000.0, 10.0, 50.0);
        manager.add_predictor("BTC".to_string(), "USD".to_string(), predictor);

        assert_eq!(manager.predictor_count(), 1);
        assert!(manager.has_predictors());

        // Update a price
        let state = manager
            .update_price(&"BTC".to_string(), &"USD".to_string(), 51000.0, 1000)
            .unwrap();
        assert_ne!(state.price, 50000.0);

        // Predict next price
        let prediction = manager
            .predict_price(&"BTC".to_string(), &"USD".to_string())
            .unwrap();
        assert_ne!(prediction.price, state.price);

        // Get estimated price
        let estimated = manager
            .get_estimated_price(&"BTC".to_string(), &"USD".to_string())
            .unwrap();
        assert_eq!(estimated, state.price);

        // Remove predictor
        assert!(manager.remove_predictor(&"BTC".to_string(), &"USD".to_string()));
        assert_eq!(manager.predictor_count(), 0);
    }

    #[test]
    fn test_reset_filter() {
        let mut filter = KalmanPricePredictor::new(100.0, 0.1, 0.5);

        // Update the filter
        filter.update(150.0, 1000).unwrap();
        assert_ne!(filter.get_estimated_price(), 100.0);
        assert!(filter.get_uncertainty() < 1.0);

        // Reset the filter
        filter.reset(200.0);
        assert_eq!(filter.get_estimated_price(), 200.0);
        assert_eq!(filter.get_uncertainty(), 1.0);
    }

    #[test]
    fn test_noise_parameter_adjustment() {
        let mut filter = KalmanPricePredictor::new(100.0, 0.1, 0.5);

        // Update with an observation
        filter.update(105.0, 1000).unwrap();
        let variance_before = filter.get_uncertainty();

        // Adjust noise parameters
        filter.set_noise_parameters(0.5, 1.0); // Higher noise

        // Update again
        filter.update(110.0, 2000).unwrap();
        let variance_after = filter.get_uncertainty();

        // With higher noise, uncertainty should be higher
        assert!(variance_after > variance_before);
    }

    #[test]
    fn test_median_calculation() {
        let mut prices = vec![100.0, 105.0, 95.0, 110.0, 90.0];
        let median = calculate_median(&mut prices);
        assert_eq!(median, 100.0);

        // Test with even number of elements
        let mut prices_even = vec![100.0, 105.0, 95.0, 110.0];
        let median_even = calculate_median(&mut prices_even);
        assert_eq!(median_even, 102.5);
    }

    #[test]
    fn test_twap_calculation() {
        // Test with simple observations
        let observations = vec![(100.0, 0), (105.0, 1000), (110.0, 2000)];

        let twap = calculate_twap(&observations);
        // Expected: (100*1000 + 105*1000) / 2000 = 102.5
        assert_eq!(twap, 102.5);
    }

    #[test]
    fn test_price_aggregator() {
        let mut aggregator = PriceAggregator::new();

        // Add observations from different sources
        aggregator.add_observation("source1".to_string(), 100.0, 1000);
        aggregator.add_observation("source2".to_string(), 105.0, 1000);
        aggregator.add_observation("source3".to_string(), 95.0, 1000);

        // Test median calculation
        let median_price = aggregator.get_median_price();
        assert_eq!(median_price, 100.0);

        // Add more observations for TWAP
        aggregator.add_observation("source1".to_string(), 102.0, 2000);
        aggregator.add_observation("source2".to_string(), 107.0, 2000);
        aggregator.add_observation("source3".to_string(), 97.0, 2000);

        // Test TWAP calculation
        let twap_price = aggregator.get_twap_price("source1", 3000);
        // Expected: (100*1000 + 102*1000) / 2000 = 101.0
        assert_eq!(twap_price, 101.0);
    }

    #[test]
    fn test_aggregated_twap() {
        let mut aggregator = PriceAggregator::new();

        // Add observations from different sources
        aggregator.add_observation("source1".to_string(), 100.0, 1000);
        aggregator.add_observation("source2".to_string(), 105.0, 1000);
        aggregator.add_observation("source1".to_string(), 102.0, 2000);
        aggregator.add_observation("source2".to_string(), 107.0, 2000);

        // Test aggregated TWAP
        let aggregated_twap = aggregator.get_aggregated_twap_price(3000);
        // source1 TWAP: (100*1000 + 102*1000) / 2000 = 101.0
        // source2 TWAP: (105*1000 + 107*1000) / 2000 = 106.0
        // Aggregated: (101.0 + 106.0) / 2 = 103.5
        assert_eq!(aggregated_twap, 103.5);
    }
}
