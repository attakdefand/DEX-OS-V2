use dex_core::liquidity_aggregator::OrderLevel;
use dex_core::types::{OrderSide, TradingPair};
use dex_core::unified_liquidity_os::{
    UnifiedLiquidityConfig, UnifiedLiquidityError, UnifiedLiquidityOS, ULTRA_LOW_SLIPPAGE_TARGET_BPS,
};

fn sample_pair() -> TradingPair {
    TradingPair {
        base: "ATOM".to_string(),
        quote: "USDC".to_string(),
    }
}

#[test]
fn guarantees_ultra_low_slippage_on_uniform_depth() {
    let mut os = UnifiedLiquidityOS::new(UnifiedLiquidityConfig::default());
    let pair = sample_pair();

    os.upsert_venue_book(
        "venue-a".to_string(),
        pair.clone(),
        vec![OrderLevel {
            price: 995,
            quantity: 200_000,
        }],
        vec![OrderLevel {
            price: 1_000,
            quantity: 300_000,
        }],
    );

    os.upsert_venue_book(
        "venue-b".to_string(),
        pair.clone(),
        vec![OrderLevel {
            price: 996,
            quantity: 150_000,
        }],
        vec![OrderLevel {
            price: 1_000,
            quantity: 600_000,
        }],
    );

    let plan = os
        .plan_execution(&pair, OrderSide::Buy, 600_000)
        .expect("should keep slippage under 0.0001%");

    assert!(plan.fully_covered);
    assert!(plan.achieved_slippage_bps <= ULTRA_LOW_SLIPPAGE_TARGET_BPS + 1e-6);
    assert_eq!(plan.slices[0].levels_consumed, 1); // all depth at the same price
    assert!(!plan.slices[0].used_virtual_liquidity);
}

#[test]
fn rejects_requests_when_target_cannot_be_met() {
    let mut os = UnifiedLiquidityOS::new(UnifiedLiquidityConfig::default());
    let pair = sample_pair();

    os.upsert_venue_book(
        "shallow-venue".to_string(),
        pair.clone(),
        vec![],
        vec![
            OrderLevel {
                price: 10_000,
                quantity: 40,
            },
            OrderLevel {
                price: 10_001,
                quantity: 60,
            },
        ],
    );

    let err = os
        .plan_execution(&pair, OrderSide::Buy, 100)
        .expect_err("slippage should exceed the 0.0001% target");

    match err {
        UnifiedLiquidityError::CannotMeetTarget {
            max_executable, ..
        } => assert_eq!(max_executable, 40),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn virtual_liquidity_extends_depth_at_best_price() {
    let mut os = UnifiedLiquidityOS::new(UnifiedLiquidityConfig::default());
    let pair = sample_pair();

    os.upsert_venue_book(
        "base-venue".to_string(),
        pair.clone(),
        vec![],
        vec![
            OrderLevel {
                price: 5_000,
                quantity: 50,
            },
            OrderLevel {
                price: 5_010,
                quantity: 200,
            },
        ],
    );

    // Inject synthetic liquidity at the best price to keep slippage inside the 0.0001% envelope.
    os.upsert_virtual_liquidity(pair.clone(), OrderSide::Buy, 5_000, 200);

    let plan = os
        .plan_execution(&pair, OrderSide::Buy, 180)
        .expect("virtual depth should preserve ultra-low slippage");

    assert!(plan.achieved_slippage_bps <= ULTRA_LOW_SLIPPAGE_TARGET_BPS + 1e-6);
    assert_eq!(plan.slices[0].levels_consumed, 1);
    assert!(plan.slices[0].used_virtual_liquidity);
}
