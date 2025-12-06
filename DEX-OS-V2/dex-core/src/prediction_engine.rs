//! Prediction Engine prioritizing transformer and reinforcement learning models.
//!
//! Implements Priority 2 feature:
//! - Components,Prediction Engine,Engine,Transformer + RL Models,Prediction Models,High
//! This module exposes an extensible prediction pipeline for market forecasting.

use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::HashMap;
use std::fmt;

/// Metadata describing the market state observed by predictors.
#[derive(Debug, Clone)]
pub struct MarketContext {
    pub base_token: String,
    pub quote_token: String,
    pub historical_prices: Vec<f64>,
    pub volatility: f64,
    pub momentum: f64,
    pub timestamp: u64,
}

impl MarketContext {
    /// Returns the most recent price if available.
    pub fn latest_price(&self) -> Option<f64> {
        self.historical_prices.last().copied()
    }

    /// Returns a normalized momentum bounded between -1 and 1.
    pub fn normalized_momentum(&self) -> f64 {
        self.momentum.clamp(-1.0, 1.0)
    }
}

/// Outcome produced by a predictor.
#[derive(Debug, Clone, PartialEq)]
pub struct PredictionResult {
    pub model_id: String,
    pub price: f64,
    pub confidence: f64,
}

impl PredictionResult {
    pub fn new(model_id: impl Into<String>, price: f64, confidence: f64) -> Self {
        Self {
            model_id: model_id.into(),
            price,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

impl fmt::Display for PredictionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] price={:.3} confidence={:.2}",
            self.model_id, self.price, self.confidence
        )
    }
}

/// Trait implemented by all prediction models.
pub trait Predictor: Send + Sync {
    fn id(&self) -> &str;
    fn predict(&mut self, context: &MarketContext) -> PredictionResult;
}

/// Transformer-style predictor that leverages normalized momentum.
#[derive(Debug)]
pub struct TransformerPredictor {
    id: String,
    sensitivity: f64,
    rng: StdRng,
}

impl TransformerPredictor {
    pub fn new(id: impl Into<String>, sensitivity: f64, seed: u64) -> Self {
        Self {
            id: id.into(),
            sensitivity: sensitivity.clamp(0.1, 2.0),
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

impl Predictor for TransformerPredictor {
    fn id(&self) -> &str {
        &self.id
    }

    fn predict(&mut self, context: &MarketContext) -> PredictionResult {
        let base_price = context.latest_price().unwrap_or(100.0);
        let momentum = context.normalized_momentum();
        let noise = self.rng.gen_range(-0.005..0.005);
        let adjustment = momentum * 0.03 * self.sensitivity;
        let price = base_price * (1.0 + adjustment + noise);
        let confidence = (0.6 + momentum.abs() * 0.3).clamp(0.0, 1.0);

        PredictionResult::new(self.id.clone(), price, confidence)
    }
}

/// Compact state key used by the RL predictor.
type StateKey = (i64, i64);

/// Tabular RL predictor that learns drift in discretized states.
#[derive(Debug)]
pub struct ReinforcementLearningPredictor {
    id: String,
    q_table: HashMap<StateKey, f64>,
    exploration_rate: f64,
    learning_rate: f64,
    rng: StdRng,
}

impl ReinforcementLearningPredictor {
    pub fn new(id: impl Into<String>, seed: u64) -> Self {
        Self {
            id: id.into(),
            q_table: HashMap::new(),
            exploration_rate: 0.2,
            learning_rate: 0.1,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn discretize(&self, value: f64) -> i64 {
        (value * 10.0).round() as i64
    }
}

impl Predictor for ReinforcementLearningPredictor {
    fn id(&self) -> &str {
        &self.id
    }

    fn predict(&mut self, context: &MarketContext) -> PredictionResult {
        let base_price = context.latest_price().unwrap_or(100.0);
        let state = (
            self.discretize(context.volatility),
            self.discretize(context.normalized_momentum()),
        );

        let q_estimate = *self.q_table.get(&state).unwrap_or(&0.0);
        let exploration = self.rng.gen_bool(self.exploration_rate);
        let predicted_delta = if exploration {
            self.rng.gen_range(-0.02..0.02)
        } else {
            (q_estimate * 0.01).clamp(-0.05, 0.05)
        };

        let price = base_price * (1.0 + predicted_delta);
        let reward = (context.volatility * 0.1) - predicted_delta.abs();

        let entry = self.q_table.entry(state).or_insert(0.0);
        *entry += self.learning_rate * (reward - *entry);

        let confidence = (0.4 + (1.0 - self.exploration_rate) * 0.4).clamp(0.0, 1.0);
        PredictionResult::new(self.id.clone(), price, confidence)
    }
}

/// Strategy used to aggregate multiple predictions.
#[derive(Debug, Clone)]
pub enum AggregationStrategy {
    ConfidenceWeighted,
    MaxConfidence,
}

/// Engine that orchestrates matching across transformer and RL models.
pub struct PredictionEngine {
    models: Vec<Box<dyn Predictor>>,
    strategy: AggregationStrategy,
}

impl PredictionEngine {
    pub fn new(models: Vec<Box<dyn Predictor>>, strategy: AggregationStrategy) -> Self {
        Self { models, strategy }
    }

    pub fn predict(&mut self, context: &MarketContext) -> PredictionBundle {
        let mut predictions = Vec::with_capacity(self.models.len());

        for model in &mut self.models {
            predictions.push(model.predict(context));
        }

        let (consensus, best) = match self.strategy {
            AggregationStrategy::ConfidenceWeighted => weighted_consensus(&predictions),
            AggregationStrategy::MaxConfidence => max_confidence(&predictions),
        };

        PredictionBundle {
            predictions,
            consensus,
            best,
        }
    }
}

impl fmt::Debug for PredictionEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PredictionEngine")
            .field("strategy", &self.strategy)
            .field("model_count", &self.models.len())
            .finish()
    }
}

fn weighted_consensus(predictions: &[PredictionResult]) -> (PredictionResult, PredictionResult) {
    let mut weighted_price = 0.0;
    let mut total_weight = 0.0;
    let mut best = predictions
        .first()
        .cloned()
        .unwrap_or_else(|| PredictionResult::new("idle", 0.0, 0.0));

    for pred in predictions {
        weighted_price += pred.price * pred.confidence;
        total_weight += pred.confidence;
        if pred.confidence > best.confidence {
            best = pred.clone();
        }
    }
    if total_weight == 0.0 {
        (
            PredictionResult::new("consensus", 0.0, 0.0),
            best.clone(),
        )
    } else {
        let consensus = PredictionResult::new("consensus", weighted_price / total_weight, 1.0);
        (consensus, best)
    }
}

fn max_confidence(predictions: &[PredictionResult]) -> (PredictionResult, PredictionResult) {
    let mut best = predictions
        .first()
        .cloned()
        .unwrap_or_else(|| PredictionResult::new("idle", 0.0, 0.0));

    for pred in predictions {
        if pred.confidence > best.confidence {
            best = pred.clone();
        }
    }

    let consensus = best.clone();
    (consensus, best)
}

/// Bundle of predictions delivered by the engine.
#[derive(Debug, Clone)]
pub struct PredictionBundle {
    pub predictions: Vec<PredictionResult>,
    pub consensus: PredictionResult,
    pub best: PredictionResult,
}
