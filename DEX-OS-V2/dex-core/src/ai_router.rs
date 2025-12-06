//! AI Router for execution engine decision making.
//!
//! Implements the Priority 2 feature from DEX-OS-V2.csv:
//! - Components,Execution Engine,Engine,AI Router,AI Routing,High
use crate::prediction_engine::{MarketContext, PredictionBundle, PredictionEngine};
use crate::types::{TokenId, TradingPair};
use std::cmp::Ordering;

/// Configuration for the AI router scoring function.
#[derive(Debug, Clone)]
pub struct AiRouterConfig {
    /// Weight applied to prediction alignment when computing the score.
    pub prediction_weight: f64,
    /// Latency penalty applied per millisecond of estimated delay.
    pub latency_weight: f64,
    /// Risk tolerance in [0.0, 1.0] where higher values accept larger slippage.
    pub risk_tolerance: f64,
}

impl Default for AiRouterConfig {
    fn default() -> Self {
        Self {
            prediction_weight: 0.35,
            latency_weight: 0.001,
            risk_tolerance: 0.65,
        }
    }
}

/// Segment of the route describing a token hop.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteSegment {
    pub from: TokenId,
    pub to: TokenId,
    pub liquidity: f64,
    pub fee_rate: f64,
    pub estimated_latency_ms: u64,
}

/// Candidate route provided to the router.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteCandidate {
    pub id: String,
    pub path: Vec<RouteSegment>,
    pub base_token: TokenId,
    pub quote_token: TokenId,
    pub expected_output: f64,
    pub estimated_slippage: f64,
    pub estimated_fee_rate: f64,
    pub estimated_latency_ms: u64,
    pub tags: Vec<String>,
}

impl RouteCandidate {
    /// Efficiency ratio used for scoring: favor larger outputs with lower friction.
    pub fn efficiency(&self) -> f64 {
        self.expected_output / (1.0 + self.estimated_fee_rate + self.estimated_slippage)
    }

    /// Primary hops for observability or tooling.
    pub fn hop_tokens(&self) -> Vec<&TokenId> {
        self.path
            .iter()
            .flat_map(|segment| vec![&segment.from, &segment.to])
            .collect()
    }
}

/// Context shared across route evaluations.
#[derive(Debug, Clone)]
pub struct RouteEvaluationContext {
    pub pair: TradingPair,
    pub base_amount: f64,
    pub market_context: MarketContext,
}

/// Represents the breakdown of a candidate score.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteScore {
    pub value: f64,
    pub efficiency: f64,
    pub alignment: f64,
    pub latency_penalty: f64,
    pub risk_penalty: f64,
}

/// Recommendation returned by the router.
#[derive(Debug, Clone)]
pub struct RouteSuggestion {
    pub candidate: RouteCandidate,
    pub score: RouteScore,
    pub prediction: PredictionBundle,
}

/// Execution AI router that ranks routes using predictions.
#[derive(Debug)]
pub struct AiRouter {
    config: AiRouterConfig,
    engine: PredictionEngine,
}

impl AiRouter {
    /// Create a new router with the given prediction engine and configuration.
    pub fn new(engine: PredictionEngine, config: AiRouterConfig) -> Self {
        Self { config, engine }
    }

    /// Create a router with default configuration.
    pub fn with_default(engine: PredictionEngine) -> Self {
        Self::new(engine, AiRouterConfig::default())
    }

    /// Rank the provided candidates and return sorted suggestions.
    pub fn rank_routes(
        &mut self,
        ctx: &RouteEvaluationContext,
        candidates: &[RouteCandidate],
    ) -> Vec<RouteSuggestion> {
        if candidates.is_empty() {
            return Vec::new();
        }

        let prediction = self.engine.predict(&ctx.market_context);
        let mut suggestions: Vec<_> = candidates
            .iter()
            .map(|candidate| RouteSuggestion {
                candidate: candidate.clone(),
                score: self.score_candidate(ctx, candidate, &prediction),
                prediction: prediction.clone(),
            })
            .collect();

        suggestions.sort_by(|a, b| {
            b.score
                .value
                .partial_cmp(&a.score.value)
                .unwrap_or(Ordering::Equal)
        });

        suggestions
    }

    /// Select the top scoring route (None when no candidates).
    pub fn select_route(
        &mut self,
        ctx: &RouteEvaluationContext,
        candidates: &[RouteCandidate],
    ) -> Option<RouteSuggestion> {
        self.rank_routes(ctx, candidates).into_iter().next()
    }

    fn score_candidate(
        &self,
        ctx: &RouteEvaluationContext,
        candidate: &RouteCandidate,
        prediction: &PredictionBundle,
    ) -> RouteScore {
        let base_amount = ctx.base_amount.max(f64::EPSILON);
        let consensus_price = prediction.consensus.price.max(f64::EPSILON);
        let candidate_price = candidate.expected_output / base_amount;
        let deviation = (candidate_price - consensus_price).abs() / consensus_price;

        let alignment = (1.0 - deviation).max(0.0);
        let latency_penalty = self.config.latency_weight * candidate.estimated_latency_ms as f64;
        let risk_penalty = candidate.estimated_slippage * (1.0 - self.config.risk_tolerance);
        let efficiency = candidate.efficiency().max(f64::EPSILON);

        let prediction_bonus =
            1.0 + (alignment * prediction.best.confidence * self.config.prediction_weight);

        let value = (efficiency * prediction_bonus) - latency_penalty - risk_penalty;

        RouteScore {
            value,
            efficiency,
            alignment,
            latency_penalty,
            risk_penalty,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prediction_engine::{AggregationStrategy, ReinforcementLearningPredictor, TransformerPredictor};

    fn build_engine() -> PredictionEngine {
        let models: Vec<Box<dyn crate::prediction_engine::Predictor>> = vec![
            Box::new(TransformerPredictor::new("transformer", 1.0, 42)),
            Box::new(ReinforcementLearningPredictor::new("rl", 24)),
        ];

        PredictionEngine::new(models, AggregationStrategy::ConfidenceWeighted)
    }

    fn default_context() -> RouteEvaluationContext {
        RouteEvaluationContext {
            pair: TradingPair {
                base: "ETH".into(),
                quote: "USDC".into(),
            },
            base_amount: 100.0,
            market_context: MarketContext {
                base_token: "ETH".into(),
                quote_token: "USDC".into(),
                historical_prices: vec![1700.0, 1725.0, 1710.0],
                volatility: 0.15,
                momentum: 0.2,
                timestamp: 1_700_000_001_000,
            },
        }
    }

    fn build_candidate(id: &str, expected_output: f64, slippage: f64) -> RouteCandidate {
        RouteCandidate {
            id: id.into(),
            path: vec![RouteSegment {
                from: "ETH".into(),
                to: "USDC".into(),
                liquidity: 1_000_000.0,
                fee_rate: 0.001,
                estimated_latency_ms: 15,
            }],
            base_token: "ETH".into(),
            quote_token: "USDC".into(),
            expected_output,
            estimated_slippage: slippage,
            estimated_fee_rate: 0.001,
            estimated_latency_ms: 15,
            tags: vec!["primary".into()],
        }
    }

    #[test]
    fn select_route_prefers_best_efficiency() {
        let mut router = AiRouter::with_default(build_engine());
        let ctx = default_context();

        let candidates = vec![
            build_candidate("cheap", 171_500.0, 0.01),
            build_candidate("expensive", 171_700.0, 0.005),
        ];

        let suggestion = router.select_route(&ctx, &candidates).unwrap();
        assert_eq!(suggestion.candidate.id, "expensive");
        assert!(suggestion.score.value > 0.0);
    }

    #[test]
    fn rank_routes_penalizes_latency() {
        let mut router = AiRouter::new(build_engine(), AiRouterConfig {
            latency_weight: 0.2,
            ..AiRouterConfig::default()
        });
        let ctx = default_context();

        let fast = RouteCandidate {
            id: "fast".into(),
            estimated_latency_ms: 10,
            ..build_candidate("fast", 171_600.0, 0.01)
        };

        let slow = RouteCandidate {
            id: "slow".into(),
            estimated_latency_ms: 500,
            ..build_candidate("slow", 171_600.0, 0.01)
        };

        let ranked = router.rank_routes(&ctx, &[slow.clone(), fast.clone()]);
        assert_eq!(ranked.first().unwrap().candidate.id, "fast");
        assert!(ranked[0].score.value > ranked[1].score.value);
    }

    #[test]
    fn handles_empty_candidate_list() {
        let mut router = AiRouter::with_default(build_engine());
        let ctx = default_context();
        assert!(router.select_route(&ctx, &[]).is_none());
        assert!(router.rank_routes(&ctx, &[]).is_empty());
    }
}
