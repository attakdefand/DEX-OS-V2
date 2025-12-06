//! Integration tests for the DEX-OS core engine
//!
//! This module provides comprehensive tests for all the features implemented.

#[cfg(test)]
mod tests {
    use dex_core::{
        amm::ConstantProductAMM,
        orderbook::OrderBook,
        types::{
            Order, OrderSide, OrderType, Trade, TradingPair,
        },
    };

    #[test]
    fn test_order_matching() {
        let mut orderbook = OrderBook::new();
        let pair = TradingPair {
            base: "BTC".to_string(),
            quote: "USD".to_string(),
        };

        // Add a sell order
        let sell_order = Order {
            id: 1,
            trader_id: "seller".to_string(),
            pair: pair.clone(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 100,
            timestamp: 1234567890,
        };

        let trades = orderbook.add_order(sell_order).unwrap();
        assert!(trades.is_empty());

        // Add a buy order that matches
        let buy_order = Order {
            id: 2,
            trader_id: "buyer".to_string(),
            pair: pair.clone(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 50,
            timestamp: 1234567891,
        };

        let trades = orderbook.add_order(buy_order).unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, 50000);
        assert_eq!(trades[0].quantity, 50);
        assert_eq!(trades[0].maker_order_id, 1);
        assert_eq!(trades[0].taker_order_id, 2);

        // Check that the sell order was partially filled
        let remaining_sell_order = orderbook.get_order(1);
        assert!(remaining_sell_order.is_some());
        assert_eq!(remaining_sell_order.unwrap().quantity, 50);
    }

    #[test]
    fn test_amm_functionality() {
        let mut amm = ConstantProductAMM::new(30); // 0.3% fee
        let token_a = "BTC".to_string();
        let token_b = "USD".to_string();

        // Add liquidity
        let liquidity_tokens = amm
            .add_liquidity(
                token_a.clone(),
                1000,
                token_b.clone(),
                50000000, // 50,000,000 USD (assuming 1 BTC = 50,000 USD)
            )
            .unwrap();

        assert!(liquidity_tokens > 0);
        assert_eq!(amm.total_supply, liquidity_tokens);
        assert_eq!(*amm.reserves.get(&token_a).unwrap(), 1000);
        assert_eq!(*amm.reserves.get(&token_b).unwrap(), 50000000);

        // Get price
        let price = amm.get_price(&token_a, &token_b).unwrap();
        assert_eq!(price, 50000.0);

        // Swap tokens
        let amount_out = amm
            .swap(token_b.clone(), token_a.clone(), 10000000)
            .unwrap(); // Swap 10,000 USD for BTC
        assert!(amount_out > 0);

        // Check that reserves were updated
        assert!(*amm.reserves.get(&token_b).unwrap() > 50000000);
        assert!(*amm.reserves.get(&token_a).unwrap() < 1000);
    }

    #[test]
    fn test_trade_struct() {
        let trade = Trade {
            id: 1,
            maker_order_id: 100,
            taker_order_id: 200,
            base_token: "BTC".to_string(),
            quote_token: "USD".to_string(),
            price: 50000,
            quantity: 100,
            timestamp: 1234567890,
        };

        assert_eq!(trade.id, 1);
        assert_eq!(trade.maker_order_id, 100);
        assert_eq!(trade.taker_order_id, 200);
        assert_eq!(trade.base_token, "BTC");
        assert_eq!(trade.quote_token, "USD");
        assert_eq!(trade.price, 50000);
        assert_eq!(trade.quantity, 100);
        assert_eq!(trade.timestamp, 1234567890);
    }
}
// Force refresh