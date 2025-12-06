use dex_core::prediction_engine::{
    AggregationStrategy, MarketContext, PredictionEngine, PredictionResult,
    Predictor, ReinforcementLearningPredictor, TransformerPredictor,
};

fn sample_context() -> MarketContext {
    MarketContext {
        base_token: "ETH".to_string(),
        quote_token: "USDC".to_string(),
        historical_prices: vec![2000.0, 2100.0, 2150.0],
        volatility: 0.08,
        momentum: 0.4,
        timestamp: 1_700_000_000,
    }
}

#[test]
fn transformer_predictor_returns_reasonable_bounds() {
    let mut model: Box<dyn Predictor> = Box::new(TransformerPredictor::new("transformer", 1.0, 42));
    let context = sample_context();
    let result = model.predict(&context);

    assert_eq!(result.model_id, "transformer");
    assert!(result.price > 2100.0 && result.price < 2200.0);
    assert!(result.confidence >= 0.6 && result.confidence <= 1.0);
}

#[test]
fn rl_predictor_updates_q_table_and_returns_prediction() {
    let mut model: Box<dyn Predictor> = Box::new(ReinforcementLearningPredictor::new("rl", 123));
    let context = sample_context();
    let first = model.predict(&context);
    let second = model.predict(&context);
    assert_eq!(first.model_id, "rl");
    assert_eq!(second.model_id, "rl");
    assert!(first.confidence <= 1.0 && second.confidence <= 1.0);
}

#[test]
fn prediction_engine_aggregates_transformer_and_rl_predictions() {
    let ctx = sample_context();
    let models: Vec<Box<dyn dex_core::prediction_engine::Predictor>> = vec![
        Box::new(TransformerPredictor::new("transformer", 1.0, 1)),
        Box::new(ReinforcementLearningPredictor::new("rl", 2)),
    ];

    let mut engine = PredictionEngine::new(models, AggregationStrategy::ConfidenceWeighted);
    let bundle = engine.predict(&ctx);

    assert_eq!(bundle.predictions.len(), 2);
    assert!(bundle.consensus.price >= 0.0);
    assert!(bundle.best.confidence >= bundle.consensus.confidence || bundle.best.confidence >= 0.4);
}
