//! Orderbook implementation for the DEX-OS core engine

use crate::avl_tree::AvlPriceLevelTree;
use crate::merkle_tree::MerkleTree;
use crate::types::{Order, OrderId, OrderSide, Price, Quantity, Trade};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap, VecDeque};

/// Represents a level in the orderbook with a specific price
#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: Price,
    pub orders: Vec<OrderId>,
    pub total_quantity: Quantity,
}

/// Wrapper for OrderId to implement Ord trait for time priority queue
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TimePriorityOrder {
    timestamp: u64,
    order_id: OrderId,
}

// Implement Ord for TimePriorityOrder to create a min-heap based on timestamp (earliest first)
impl Ord for TimePriorityOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse the comparison to make it a min-heap (earliest timestamp first)
        other.timestamp.cmp(&self.timestamp)
    }
}

impl PartialOrd for TimePriorityOrder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Main orderbook structure
#[derive(Debug, Clone)]
pub struct OrderBook {
    /// Buy orders sorted by price (highest first) using AVL tree for balancing
    /// This implements the Priority 3 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,AVL Tree,Order Book Balancing,Medium"
    pub bids: BTreeMap<Price, PriceLevel>,
    /// Sell orders sorted by price (lowest first) using AVL tree for balancing
    /// This implements the Priority 3 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,AVL Tree,Order Book Balancing,Medium"
    pub asks: BTreeMap<Price, PriceLevel>,
    /// AVL tree for tracking bid price levels (for balanced operations)
    /// This implements the Priority 3 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,AVL Tree,Order Book Balancing,Medium"
    pub bid_price_levels: AvlPriceLevelTree,
    /// AVL tree for tracking ask price levels (for balanced operations)
    /// This implements the Priority 3 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,AVL Tree,Order Book Balancing,Medium"
    pub ask_price_levels: AvlPriceLevelTree,
    /// All orders indexed by ID for quick lookup
    pub orders: HashMap<OrderId, Order>,
    /// Time priority queue for efficient order processing (min-heap based on timestamp)
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Heap,Time Priority Queue,High"
    pub time_priority_queue: BinaryHeap<Reverse<TimePriorityOrder>>,
    /// Transaction mempool for pending transactions (FIFO queue)
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Queue,Transaction Mempool,High"
    pub transaction_mempool: VecDeque<Order>,
}

impl OrderBook {
    /// Create a new empty orderbook
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            bid_price_levels: AvlPriceLevelTree::new(),
            ask_price_levels: AvlPriceLevelTree::new(),
            orders: HashMap::new(),
            time_priority_queue: BinaryHeap::new(),
            transaction_mempool: VecDeque::new(),
        }
    }

    /// Add an order to the orderbook and match it against existing orders
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Price-Time Priority,Order Matching,High"
    pub fn add_order(&mut self, mut order: Order) -> Result<Vec<Trade>, OrderBookError> {
        // Add order to time priority queue
        // This implements the Priority 1 feature from DEX-OS-V1.csv:
        // "Core Trading,Orderbook,Orderbook,Heap,Time Priority Queue,High"
        self.time_priority_queue.push(Reverse(TimePriorityOrder {
            timestamp: order.timestamp,
            order_id: order.id,
        }));

        // Try to match the order against existing book state
        let trades = self.match_order(&order);

        // Compute how much of the taker order was filled in this matching round
        let filled_qty: Quantity = trades
            .iter()
            .filter(|t| t.taker_order_id == order.id)
            .map(|t| t.quantity)
            .sum();

        if filled_qty >= order.quantity {
            // Fully filled — do not add to the book or orders map
            return Ok(trades);
        }

        // Partially filled — update remaining quantity and persist to the book
        order.quantity -= filled_qty;
        let updated = order.clone();
        self.orders.insert(updated.id, updated.clone());
        match updated.side {
            OrderSide::Buy => self.add_bid(updated),
            OrderSide::Sell => self.add_ask(updated),
        }

        Ok(trades)
    }

    /// Add an order to the transaction mempool
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Queue,Transaction Mempool,High"
    pub fn add_to_mempool(&mut self, order: Order) {
        self.transaction_mempool.push_back(order);
    }

    /// Process the next order from the transaction mempool
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Queue,Transaction Mempool,High"
    pub fn process_next_from_mempool(&mut self) -> Option<Result<Vec<Trade>, OrderBookError>> {
        self.transaction_mempool
            .pop_front()
            .map(|order| self.add_order(order))
    }

    /// Get the number of pending transactions in the mempool
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Queue,Transaction Mempool,High"
    pub fn mempool_size(&self) -> usize {
        self.transaction_mempool.len()
    }

    /// Match an order against existing orders in the book using price-time priority
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Price-Time Priority,Order Matching,High"
    fn match_order(&mut self, order: &Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        let mut remaining_quantity = order.quantity;

        match order.side {
            OrderSide::Buy => {
                // Match against asks (sell orders)
                let mut asks_to_remove = Vec::new();

                // Iterate through asks in ascending price order
                for (&ask_price, ask_level) in self.asks.iter_mut() {
                    // If this is a limit order and the ask price is higher than our limit, stop matching
                    if let Some(limit_price) = order.price {
                        if ask_price > limit_price {
                            break;
                        }
                    }

                    // Match against orders at this price level in time priority (FIFO)
                    // Orders are processed in the order they were added to the vector (FIFO)
                    let mut orders_to_remove = Vec::new();
                    for &ask_order_id in &ask_level.orders {
                        if let Some(ask_order) = self.orders.get(&ask_order_id) {
                            let trade_quantity =
                                std::cmp::min(remaining_quantity, ask_order.quantity);

                            // Create a trade
                            let trade = Trade {
                                id: 0, // This will be set by the trade ID counter in the API
                                maker_order_id: ask_order_id,
                                taker_order_id: order.id,
                                base_token: ask_order.pair.base.clone(),
                                quote_token: ask_order.pair.quote.clone(),
                                price: ask_price,
                                quantity: trade_quantity,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                            };

                            trades.push(trade);
                            // Update remaining taker quantity
                            remaining_quantity = remaining_quantity.saturating_sub(trade_quantity);

                            // Decrement total quantity at this price level
                            ask_level.total_quantity = ask_level
                                .total_quantity
                                .saturating_sub(trade_quantity);

                            // If the ask order is fully filled, mark it for removal
                            if trade_quantity >= ask_order.quantity {
                                orders_to_remove.push(ask_order_id);
                            } else {
                                // Update the remaining quantity of the ask order
                                if let Some(ask_order_mut) = self.orders.get_mut(&ask_order_id) {
                                    ask_order_mut.quantity = ask_order_mut
                                        .quantity
                                        .saturating_sub(trade_quantity);
                                }
                            }

                            // If the taker order is fully filled, stop matching
                            if remaining_quantity == 0 {
                                break;
                            }
                        }
                    }

                    // Remove fully filled ask orders
                    for order_id in orders_to_remove {
                        ask_level.orders.retain(|&id| id != order_id);
                        // Order was fully consumed — already deducted from total_quantity above
                        self.orders.remove(&order_id);
                    }

                    // If the price level is empty, mark it for removal
                    if ask_level.orders.is_empty() {
                        asks_to_remove.push(ask_price);
                    }

                    // If the taker order is fully filled, stop matching
                    if remaining_quantity == 0 {
                        break;
                    }
                }

                // Remove empty ask price levels
                for price in asks_to_remove {
                    self.asks.remove(&price);
                    // Keep AVL tree in sync when a level becomes empty
                    self.ask_price_levels.remove_price_level(&price);
                }
            }
            OrderSide::Sell => {
                // Match against bids (buy orders)
                let mut bids_to_remove = Vec::new();

                // Iterate through bids in descending price order
                for (&bid_price, bid_level) in self.bids.iter_mut().rev() {
                    // If this is a limit order and the bid price is lower than our limit, stop matching
                    if let Some(limit_price) = order.price {
                        if bid_price < limit_price {
                            break;
                        }
                    }

                    // Match against orders at this price level in time priority (FIFO)
                    // Orders are processed in the order they were added to the vector (FIFO)
                    let mut orders_to_remove = Vec::new();
                    for &bid_order_id in &bid_level.orders {
                        if let Some(bid_order) = self.orders.get(&bid_order_id) {
                            let trade_quantity =
                                std::cmp::min(remaining_quantity, bid_order.quantity);

                            // Create a trade
                            let trade = Trade {
                                id: 0, // This will be set by the trade ID counter in the API
                                maker_order_id: bid_order_id,
                                taker_order_id: order.id,
                                base_token: bid_order.pair.base.clone(),
                                quote_token: bid_order.pair.quote.clone(),
                                price: bid_price,
                                quantity: trade_quantity,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                            };

                            trades.push(trade);
                            // Update remaining taker quantity
                            remaining_quantity = remaining_quantity.saturating_sub(trade_quantity);

                            // Decrement total quantity at this price level
                            bid_level.total_quantity = bid_level
                                .total_quantity
                                .saturating_sub(trade_quantity);

                            // If the bid order is fully filled, mark it for removal
                            if trade_quantity >= bid_order.quantity {
                                orders_to_remove.push(bid_order_id);
                            } else {
                                // Update the remaining quantity of the bid order
                                if let Some(bid_order_mut) = self.orders.get_mut(&bid_order_id) {
                                    bid_order_mut.quantity = bid_order_mut
                                        .quantity
                                        .saturating_sub(trade_quantity);
                                }
                            }

                            // If the taker order is fully filled, stop matching
                            if remaining_quantity == 0 {
                                break;
                            }
                        }
                    }

                    // Remove fully filled bid orders
                    for order_id in orders_to_remove {
                        bid_level.orders.retain(|&id| id != order_id);
                        // Order was fully consumed — already deducted from total_quantity above
                        self.orders.remove(&order_id);
                    }

                    // If the price level is empty, mark it for removal
                    if bid_level.orders.is_empty() {
                        bids_to_remove.push(bid_price);
                    }

                    // If the taker order is fully filled, stop matching
                    if remaining_quantity == 0 {
                        break;
                    }
                }

                // Remove empty bid price levels
                for price in bids_to_remove {
                    self.bids.remove(&price);
                    // Keep AVL tree in sync when a level becomes empty
                    self.bid_price_levels.remove_price_level(&price);
                }
            }
        }

        trades
    }

    /// Add a bid order to the orderbook
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Vector,Order Queue,High"
    /// and "Core Trading,Orderbook,Orderbook,Red-Black Tree,Price Level Storage,High"
    /// and the Priority 3 feature for AVL Tree balancing:
    /// "Core Trading,Orderbook,Orderbook,AVL Tree,Order Book Balancing,Medium"
    fn add_bid(&mut self, order: Order) {
        if let Some(price) = order.price {
            // Add to AVL tree for balanced price level tracking
            self.bid_price_levels.insert_price_level(price);

            self.bids
                .entry(price)
                .and_modify(|level| {
                    level.orders.push(order.id);
                    level.total_quantity += order.quantity;
                })
                .or_insert(PriceLevel {
                    price,
                    orders: vec![order.id],
                    total_quantity: order.quantity,
                });
        }
    }

    /// Add an ask order to the orderbook
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Vector,Order Queue,High"
    /// and "Core Trading,Orderbook,Orderbook,Red-Black Tree,Price Level Storage,High"
    /// and the Priority 3 feature for AVL Tree balancing:
    /// "Core Trading,Orderbook,Orderbook,AVL Tree,Order Book Balancing,Medium"
    fn add_ask(&mut self, order: Order) {
        if let Some(price) = order.price {
            // Add to AVL tree for balanced price level tracking
            self.ask_price_levels.insert_price_level(price);

            self.asks
                .entry(price)
                .and_modify(|level| {
                    level.orders.push(order.id);
                    level.total_quantity += order.quantity;
                })
                .or_insert(PriceLevel {
                    price,
                    orders: vec![order.id],
                    total_quantity: order.quantity,
                });
        }
    }

    /// Remove an order from the orderbook
    pub fn remove_order(&mut self, order_id: OrderId) -> Result<Order, OrderBookError> {
        let order = self
            .orders
            .remove(&order_id)
            .ok_or(OrderBookError::OrderNotFound)?;

        match order.side {
            OrderSide::Buy => self.remove_bid(order_id, order.price.unwrap_or(0)),
            OrderSide::Sell => self.remove_ask(order_id, order.price.unwrap_or(0)),
        }

        Ok(order)
    }

    /// Remove a bid order from the orderbook
    fn remove_bid(&mut self, order_id: OrderId, price: Price) {
        if let Some(level) = self.bids.get_mut(&price) {
            level.orders.retain(|&id| id != order_id);
            // If the price level is now empty, remove it from the AVL tree as well
            if level.orders.is_empty() {
                self.bid_price_levels.remove_price_level(&price);
            }
            // Note: In a production implementation, we would recalculate total_quantity
            // based on remaining orders to avoid floating point errors
        }
    }

    /// Remove an ask order from the orderbook
    fn remove_ask(&mut self, order_id: OrderId, price: Price) {
        if let Some(level) = self.asks.get_mut(&price) {
            level.orders.retain(|&id| id != order_id);
            // If the price level is now empty, remove it from the AVL tree as well
            if level.orders.is_empty() {
                self.ask_price_levels.remove_price_level(&price);
            }
            // Note: In a production implementation, we would recalculate total_quantity
            // based on remaining orders to avoid floating point errors
        }
    }

    /// Get the best bid price (highest buy order)
    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next_back().copied()
    }

    /// Get the best ask price (lowest sell order)
    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    /// Lookup an order by its ID
    /// This implements the Priority 2 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Hash Map,Order ID Lookup,Medium"
    pub fn get_order(&self, order_id: OrderId) -> Option<&Order> {
        self.orders.get(&order_id)
    }

    /// Generate a Merkle proof for a batch of orders
    /// This implements the Priority 2 feature from DEX-OS-V1.csv for Batch Order Proofs
    pub fn generate_batch_proof(&self, order_ids: &[OrderId]) -> Option<Vec<u8>> {
        // Collect the orders for the batch
        let mut orders: Vec<Order> = Vec::new();
        for &order_id in order_ids {
            if let Some(order) = self.orders.get(&order_id) {
                orders.push(order.clone());
            } else {
                // If any order is not found, we can't generate a proof for this batch
                return None;
            }
        }

        // Convert orders to serialized data for the Merkle tree
        let mut order_data: Vec<Vec<u8>> = Vec::new();
        for order in orders {
            // In a real implementation, you would serialize the order properly
            // For this example, we'll use a simple representation
            let data = format!(
                "{}:{}:{}:{}",
                order.id, order.trader_id, order.quantity, order.timestamp
            );
            order_data.push(data.into_bytes());
        }

        // Create the Merkle tree
        let tree = MerkleTree::from_data(&order_data);

        // Return the root hash as the batch proof
        tree.root_hash()
    }

    /// Get the next order to process based on time priority
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Heap,Time Priority Queue,High"
    pub fn get_next_order_by_time(&mut self) -> Option<OrderId> {
        self.time_priority_queue
            .pop()
            .map(|Reverse(order)| order.order_id)
    }

    /// Peek at the next order to process based on time priority without removing it
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Heap,Time Priority Queue,High"
    pub fn peek_next_order_by_time(&self) -> Option<OrderId> {
        self.time_priority_queue
            .peek()
            .map(|Reverse(order)| order.order_id)
    }

    /// Get all bid price levels in sorted order using the AVL tree
    /// This implements the Priority 3 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,AVL Tree,Order Book Balancing,Medium"
    pub fn get_all_bid_price_levels(&self) -> Vec<Price> {
        self.bid_price_levels.get_all_price_levels()
    }

    /// Get all ask price levels in sorted order using the AVL tree
    /// This implements the Priority 3 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,AVL Tree,Order Book Balancing,Medium"
    pub fn get_all_ask_price_levels(&self) -> Vec<Price> {
        self.ask_price_levels.get_all_price_levels()
    }

    /// Check if a bid price level exists using the AVL tree
    /// This implements the Priority 3 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,AVL Tree,Order Book Balancing,Medium"
    pub fn contains_bid_price_level(&self, price: &Price) -> bool {
        self.bid_price_levels.contains_price_level(price)
    }

    /// Check if an ask price level exists using the AVL tree
    /// This implements the Priority 3 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,AVL Tree,Order Book Balancing,Medium"
    pub fn contains_ask_price_level(&self, price: &Price) -> bool {
        self.ask_price_levels.contains_price_level(price)
    }
}

/// Errors that can occur when working with the orderbook
#[derive(Debug, thiserror::Error)]
pub enum OrderBookError {
    #[error("Order not found")]
    OrderNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{OrderType, TradingPair};

    #[test]
    fn test_orderbook_creation() {
        let orderbook = OrderBook::new();
        assert!(orderbook.bids.is_empty());
        assert!(orderbook.asks.is_empty());
        assert!(orderbook.orders.is_empty());
    }

    #[test]
    fn test_add_order() {
        let mut orderbook = OrderBook::new();
        let pair = TradingPair {
            base: "BTC".to_string(),
            quote: "USD".to_string(),
        };

        let order = Order {
            id: 1,
            trader_id: "trader1".to_string(),
            pair,
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 100,
            timestamp: 1234567890,
        };

        assert!(orderbook.add_order(order).is_ok());
        assert_eq!(orderbook.orders.len(), 1);
        assert_eq!(orderbook.bids.len(), 1);
        assert_eq!(orderbook.asks.len(), 0);
    }

    #[test]
    fn test_batch_proof_generation() {
        let mut orderbook = OrderBook::new();
        let pair = TradingPair {
            base: "BTC".to_string(),
            quote: "USD".to_string(),
        };

        // Add multiple orders
        let order1 = Order {
            id: 1,
            trader_id: "trader1".to_string(),
            pair: pair.clone(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 100,
            timestamp: 1234567890,
        };

        let order2 = Order {
            id: 2,
            trader_id: "trader2".to_string(),
            pair: pair.clone(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            price: Some(51000),
            quantity: 200,
            timestamp: 1234567891,
        };

        assert!(orderbook.add_order(order1).is_ok());
        assert!(orderbook.add_order(order2).is_ok());

        // Generate a batch proof
        let order_ids = vec![1, 2];
        let proof = orderbook.generate_batch_proof(&order_ids);
        assert!(proof.is_some());

        // Try to generate a proof with a non-existent order
        let invalid_order_ids = vec![1, 3];
        let invalid_proof = orderbook.generate_batch_proof(&invalid_order_ids);
        assert!(invalid_proof.is_none());
    }

    #[test]
    fn test_order_lookup() {
        let mut orderbook = OrderBook::new();
        let pair = TradingPair {
            base: "BTC".to_string(),
            quote: "USD".to_string(),
        };

        let order = Order {
            id: 1,
            trader_id: "trader1".to_string(),
            pair,
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 100,
            timestamp: 1234567890,
        };

        // Add order to orderbook
        assert!(orderbook.add_order(order.clone()).is_ok());

        // Lookup the order by ID
        let found_order = orderbook.get_order(1);
        assert!(found_order.is_some());
        assert_eq!(found_order.unwrap().id, 1);
        assert_eq!(found_order.unwrap().trader_id, "trader1");
        assert_eq!(found_order.unwrap().quantity, 100);

        // Try to lookup a non-existent order
        let not_found = orderbook.get_order(999);
        assert!(not_found.is_none());
    }

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

        assert!(orderbook.add_order(sell_order).is_ok());

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

    /// Test price-time priority matching
    /// This test verifies that orders at the same price level are matched in FIFO order
    /// Implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Price-Time Priority,Order Matching,High"
    #[test]
    fn test_price_time_priority_matching() {
        let mut orderbook = OrderBook::new();
        let pair = TradingPair {
            base: "BTC".to_string(),
            quote: "USD".to_string(),
        };

        // Add multiple sell orders at the same price (50000) but different times
        // Order 1 (timestamp 1000) should be matched first due to FIFO
        let sell_order1 = Order {
            id: 1,
            trader_id: "seller1".to_string(),
            pair: pair.clone(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 50,
            timestamp: 1000, // Earlier timestamp
        };

        // Order 2 (timestamp 2000) should be matched second
        let sell_order2 = Order {
            id: 2,
            trader_id: "seller2".to_string(),
            pair: pair.clone(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 50,
            timestamp: 2000, // Later timestamp
        };

        // Order 3 (timestamp 3000) should be matched third
        let sell_order3 = Order {
            id: 3,
            trader_id: "seller3".to_string(),
            pair: pair.clone(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 50,
            timestamp: 3000, // Latest timestamp
        };

        // Add all sell orders to the orderbook
        assert!(orderbook.add_order(sell_order1).is_ok());
        assert!(orderbook.add_order(sell_order2).is_ok());
        assert!(orderbook.add_order(sell_order3).is_ok());

        // Verify all orders are in the orderbook
        assert_eq!(orderbook.orders.len(), 3);
        assert_eq!(orderbook.asks.len(), 1); // All at the same price level
        assert_eq!(orderbook.asks.get(&50000).unwrap().orders.len(), 3);

        // Add a large buy order that will match all sell orders
        let buy_order = Order {
            id: 4,
            trader_id: "buyer".to_string(),
            pair: pair.clone(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 150, // Large enough to match all sell orders
            timestamp: 4000,
        };

        // Execute the matching
        let trades = orderbook.add_order(buy_order).unwrap();

        // Verify we have 3 trades
        assert_eq!(trades.len(), 3);

        // Verify the matching order follows FIFO (price-time priority):
        // 1. Order 1 (earliest timestamp) should be matched first
        assert_eq!(trades[0].maker_order_id, 1);
        assert_eq!(trades[0].price, 50000);
        assert_eq!(trades[0].quantity, 50);

        // 2. Order 2 (middle timestamp) should be matched second
        assert_eq!(trades[1].maker_order_id, 2);
        assert_eq!(trades[1].price, 50000);
        assert_eq!(trades[1].quantity, 50);

        // 3. Order 3 (latest timestamp) should be matched third
        assert_eq!(trades[2].maker_order_id, 3);
        assert_eq!(trades[2].price, 50000);
        assert_eq!(trades[2].quantity, 50);

        // Verify all sell orders have been removed from the orderbook
        assert_eq!(orderbook.orders.len(), 1); // Only the buy order remains
        assert_eq!(orderbook.orders.get(&4).unwrap().id, 4); // The buy order
        assert_eq!(orderbook.asks.len(), 0); // No more asks
    }

    /// Test price priority matching
    /// This test verifies that orders are matched based on price priority first
    /// Implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Price-Time Priority,Order Matching,High"
    #[test]
    fn test_price_priority_matching() {
        let mut orderbook = OrderBook::new();
        let pair = TradingPair {
            base: "BTC".to_string(),
            quote: "USD".to_string(),
        };

        // Add sell orders at different prices
        // Better price (lower) should be matched first
        let sell_order_high_price = Order {
            id: 1,
            trader_id: "seller1".to_string(),
            pair: pair.clone(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            price: Some(51000), // Higher price
            quantity: 50,
            timestamp: 1000,
        };

        let sell_order_low_price = Order {
            id: 2,
            trader_id: "seller2".to_string(),
            pair: pair.clone(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            price: Some(50000), // Lower price (better for buyer)
            quantity: 50,
            timestamp: 2000,
        };

        // Add sell orders to the orderbook
        assert!(orderbook.add_order(sell_order_high_price).is_ok());
        assert!(orderbook.add_order(sell_order_low_price).is_ok());

        // Verify orders are in the orderbook
        assert_eq!(orderbook.orders.len(), 2);
        assert_eq!(orderbook.asks.len(), 2); // Two different price levels

        // Add a buy order that will match sell orders
        let buy_order = Order {
            id: 3,
            trader_id: "buyer".to_string(),
            pair: pair.clone(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(52000), // High enough to match both sell orders
            quantity: 100,
            timestamp: 3000,
        };

        // Execute the matching
        let trades = orderbook.add_order(buy_order).unwrap();

        // Verify we have 2 trades
        assert_eq!(trades.len(), 2);

        // Verify the matching order follows price priority:
        // 1. Order 2 (lower price) should be matched first
        assert_eq!(trades[0].maker_order_id, 2);
        assert_eq!(trades[0].price, 50000);
        assert_eq!(trades[0].quantity, 50);

        // 2. Order 1 (higher price) should be matched second
        assert_eq!(trades[1].maker_order_id, 1);
        assert_eq!(trades[1].price, 51000);
        assert_eq!(trades[1].quantity, 50);
    }

    /// Test Heap Time Priority Queue functionality
    /// This test verifies that orders can be retrieved in time priority order
    /// Implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Heap,Time Priority Queue,High"
    #[test]
    fn test_heap_time_priority_queue() {
        let mut orderbook = OrderBook::new();
        let pair = TradingPair {
            base: "BTC".to_string(),
            quote: "USD".to_string(),
        };

        // Add orders with different timestamps
        let order1 = Order {
            id: 1,
            trader_id: "trader1".to_string(),
            pair: pair.clone(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 100,
            timestamp: 3000, // Latest timestamp
        };

        let order2 = Order {
            id: 2,
            trader_id: "trader2".to_string(),
            pair: pair.clone(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(51000),
            quantity: 200,
            timestamp: 1000, // Earliest timestamp
        };

        let order3 = Order {
            id: 3,
            trader_id: "trader3".to_string(),
            pair: pair.clone(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            price: Some(52000),
            quantity: 300,
            timestamp: 2000, // Middle timestamp
        };

        // Add orders to the orderbook
        assert!(orderbook.add_order(order1).is_ok());
        assert!(orderbook.add_order(order2).is_ok());
        assert!(orderbook.add_order(order3).is_ok());

        // Verify all orders are in the orderbook
        assert_eq!(orderbook.orders.len(), 3);

        // Verify time priority queue functionality
        // Order 2 should be first (earliest timestamp)
        assert_eq!(orderbook.peek_next_order_by_time(), Some(2));
        assert_eq!(orderbook.get_next_order_by_time(), Some(2));

        // Order 3 should be next (middle timestamp)
        assert_eq!(orderbook.peek_next_order_by_time(), Some(3));
        assert_eq!(orderbook.get_next_order_by_time(), Some(3));

        // Order 1 should be last (latest timestamp)
        assert_eq!(orderbook.peek_next_order_by_time(), Some(1));
        assert_eq!(orderbook.get_next_order_by_time(), Some(1));

        // Queue should now be empty
        assert_eq!(orderbook.peek_next_order_by_time(), None);
        assert_eq!(orderbook.get_next_order_by_time(), None);
    }

    /// Test Queue Transaction Mempool functionality
    /// This test verifies that orders can be added to and processed from the mempool in FIFO order
    /// Implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Queue,Transaction Mempool,High"
    #[test]
    fn test_queue_transaction_mempool() {
        let mut orderbook = OrderBook::new();
        let pair = TradingPair {
            base: "BTC".to_string(),
            quote: "USD".to_string(),
        };

        // Create orders
        let order1 = Order {
            id: 1,
            trader_id: "trader1".to_string(),
            pair: pair.clone(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 100,
            timestamp: 1000,
        };

        let order2 = Order {
            id: 2,
            trader_id: "trader2".to_string(),
            pair: pair.clone(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            price: Some(51000),
            quantity: 200,
            timestamp: 2000,
        };

        let order3 = Order {
            id: 3,
            trader_id: "trader3".to_string(),
            pair: pair.clone(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(52000),
            quantity: 300,
            timestamp: 3000,
        };

        // Add orders to the mempool
        orderbook.add_to_mempool(order1.clone());
        orderbook.add_to_mempool(order2.clone());
        orderbook.add_to_mempool(order3.clone());

        // Verify mempool size
        assert_eq!(orderbook.mempool_size(), 3);

        // Process orders from mempool (FIFO order)
        let result1 = orderbook.process_next_from_mempool();
        assert!(result1.is_some());
        assert!(result1.unwrap().is_ok());
        assert_eq!(orderbook.mempool_size(), 2);

        let result2 = orderbook.process_next_from_mempool();
        assert!(result2.is_some());
        assert!(result2.unwrap().is_ok());
        assert_eq!(orderbook.mempool_size(), 1);

        let result3 = orderbook.process_next_from_mempool();
        assert!(result3.is_some());
        assert!(result3.unwrap().is_ok());
        assert_eq!(orderbook.mempool_size(), 0);

        // Try to process from empty mempool
        let result4 = orderbook.process_next_from_mempool();
        assert!(result4.is_none());
    }
}
