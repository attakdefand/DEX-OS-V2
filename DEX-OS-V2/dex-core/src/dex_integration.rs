//! DEX Aggregator API integrations across multiple venues.
//!
//! Implements Priority 4 feature:
//! - `4,Core Trading,DEX Aggregator,DEX Aggregator,API Integrations with Multiple DEXs,DEX Integration,High`
//!
//! The integration layer coordinates multiple venue adapters, keeps per-venue
//! health, caches quotes to avoid hammering external APIs, and can hydrate the
//! internal liquidity aggregator with fresh order books.

use crate::liquidity_aggregator::{AggregatorError, LiquidityAggregator, OrderLevel};
use crate::types::{OrderSide, TradingPair};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Configuration for the DEX integration gateway.
#[derive(Debug, Clone)]
pub struct DexIntegrationConfig {
    /// How long quotes remain valid in the cache (in milliseconds).
    pub quote_ttl_ms: u64,
    /// Number of consecutive failures before an adapter is marked unhealthy.
    pub max_failures_before_unhealthy: u8,
}

impl Default for DexIntegrationConfig {
    fn default() -> Self {
        Self {
            quote_ttl_ms: 5_000,
            max_failures_before_unhealthy: 3,
        }
    }
}

/// Adapter-level errors surfaced while talking to a venue API.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DexAdapterError {
    #[error("pair {0:?} is not supported by this venue")]
    UnsupportedPair(TradingPair),
    #[error("network error: {0}")]
    Network(String),
    #[error("timeout after {0} ms")]
    Timeout(u64),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    #[error("insufficient liquidity for pair {0:?}")]
    InsufficientLiquidity(TradingPair),
}

/// Errors produced by the integration gateway when orchestrating multiple adapters.
#[derive(Debug, Error, PartialEq)]
pub enum DexIntegrationError {
    #[error("no DEX adapters registered")]
    NoAdaptersRegistered,
    #[error("no adapters support pair {0:?}")]
    UnsupportedPair(TradingPair),
    #[error("all adapters failed: {0:?}")]
    AllAdaptersFailed(Vec<(String, DexAdapterError)>),
    #[error("liquidity aggregator error: {0}")]
    Aggregator(#[from] AggregatorError),
}

/// Resulting quote from a venue for a specific side and quantity.
#[derive(Debug, Clone, PartialEq)]
pub struct DexQuote {
    pub venue_id: String,
    pub pair: TradingPair,
    pub side: OrderSide,
    pub quantity: u64,
    pub average_price: f64,
    pub best_price: u64,
    pub worst_price: u64,
    pub slippage_bps: f64,
    pub estimated_gas_units: u64,
    pub liquidity_available: u64,
    pub fetched_at_ms: u64,
}

/// Venue order book snapshot returned by an adapter.
#[derive(Debug, Clone, PartialEq)]
pub struct DexOrderBook {
    pub pair: TradingPair,
    pub bids: Vec<OrderLevel>,
    pub asks: Vec<OrderLevel>,
    pub fetched_at_ms: u64,
}

impl DexOrderBook {
    /// Total liquidity available for a given side.
    pub fn total_available(&self, side: OrderSide) -> u64 {
        match side {
            OrderSide::Buy => self.asks.iter().map(|l| l.quantity).sum(),
            OrderSide::Sell => self.bids.iter().map(|l| l.quantity).sum(),
        }
    }
}

/// Trait each venue adapter must implement.
pub trait DexAdapter: Send + Sync {
    /// Stable identifier for the venue (e.g., "uniswap-v3").
    fn id(&self) -> &str;
    /// Whether the adapter supports quoting this trading pair.
    fn supports_pair(&self, pair: &TradingPair) -> bool;
    /// Fetch an order book snapshot for the pair.
    fn fetch_order_book(&self, pair: &TradingPair) -> Result<DexOrderBook, DexAdapterError>;
    /// Produce a quote for a given side and quantity.
    fn quote(
        &self,
        pair: &TradingPair,
        side: OrderSide,
        quantity: u64,
    ) -> Result<DexQuote, DexAdapterError>;
}

/// Health and failure tracking per adapter.
#[derive(Debug, Clone, PartialEq)]
pub struct AdapterHealth {
    pub healthy: bool,
    pub consecutive_failures: u8,
    pub last_error: Option<String>,
}

impl Default for AdapterHealth {
    fn default() -> Self {
        Self {
            healthy: true,
            consecutive_failures: 0,
            last_error: None,
        }
    }
}

impl AdapterHealth {
    fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.healthy = true;
        self.last_error = None;
    }

    fn record_failure(&mut self, config: &DexIntegrationConfig, err: &DexAdapterError) {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        self.last_error = Some(err.to_string());
        if self.consecutive_failures >= config.max_failures_before_unhealthy {
            self.healthy = false;
        }
    }
}

struct AdapterState {
    adapter: Arc<dyn DexAdapter + Send + Sync>,
    health: AdapterHealth,
}

impl fmt::Debug for AdapterState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AdapterState")
            .field("health", &self.health)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct QuoteCacheKey {
    venue_id: String,
    pair: TradingPair,
    side: OrderSide,
    quantity: u64,
}

impl QuoteCacheKey {
    fn new(venue_id: &str, pair: &TradingPair, side: OrderSide, quantity: u64) -> Self {
        Self {
            venue_id: venue_id.to_string(),
            pair: pair.clone(),
            side,
            quantity,
        }
    }
}

#[derive(Debug, Clone)]
struct CachedQuote {
    quote: DexQuote,
    expires_at_ms: u64,
}

/// Gateway that coordinates multiple DEX adapters.
#[derive(Debug)]
pub struct DexIntegrationGateway {
    adapters: HashMap<String, AdapterState>,
    quote_cache: HashMap<QuoteCacheKey, CachedQuote>,
    config: DexIntegrationConfig,
}

impl DexIntegrationGateway {
    /// Create a gateway with the provided configuration.
    pub fn new(config: DexIntegrationConfig) -> Self {
        Self {
            adapters: HashMap::new(),
            quote_cache: HashMap::new(),
            config,
        }
    }

    /// Register or replace an adapter by its id.
    pub fn register_adapter(&mut self, adapter: Arc<dyn DexAdapter + Send + Sync>) {
        let id = adapter.id().to_string();
        self.adapters.insert(
            id,
            AdapterState {
                adapter,
                health: AdapterHealth::default(),
            },
        );
    }

    /// Remove an adapter from the registry.
    pub fn remove_adapter(&mut self, venue_id: &str) -> bool {
        self.adapters.remove(venue_id).is_some()
    }

    /// Inspect health for a given adapter.
    pub fn health(&self, venue_id: &str) -> Option<AdapterHealth> {
        self.adapters.get(venue_id).map(|a| a.health.clone())
    }

    /// Clear cached quotes (useful for tests or forced refresh).
    pub fn clear_cache(&mut self) {
        self.quote_cache.clear();
    }

    /// Fetch quotes from all supporting adapters, sorted by best execution.
    pub fn fetch_quotes(
        &mut self,
        pair: &TradingPair,
        side: OrderSide,
        quantity: u64,
    ) -> Result<Vec<DexQuote>, DexIntegrationError> {
        if self.adapters.is_empty() {
            return Err(DexIntegrationError::NoAdaptersRegistered);
        }

        let now = now_ms();
        let mut responses = Vec::new();
        let mut failures = Vec::new();
        let mut supported = false;

        // Iterate by adapter id to avoid borrow conflicts between cache lookups and adapter mutations.
        let adapter_ids: Vec<String> = self.adapters.keys().cloned().collect();
        for id in adapter_ids {
            let supports_pair = self
                .adapters
                .get(&id)
                .map(|state| state.adapter.supports_pair(pair))
                .unwrap_or(false);

            if !supports_pair {
                continue;
            }

            supported = true;
            let cache_key = QuoteCacheKey::new(&id, pair, side, quantity);
            if let Some(quote) = self.cached_quote(&cache_key, now) {
                responses.push(quote);
                continue;
            }

            let mut produced_quote: Option<DexQuote> = None;
            {
                let adapter_state = self
                    .adapters
                    .get_mut(&id)
                    .expect("adapter should exist during iteration");

                match adapter_state.adapter.quote(pair, side, quantity) {
                    Ok(mut quote) => {
                        quote.fetched_at_ms = now;
                        adapter_state.health.record_success();
                        produced_quote = Some(quote);
                    }
                    Err(err) => {
                        adapter_state.health.record_failure(&self.config, &err);
                        failures.push((id.clone(), err));
                    }
                }
            }

            if let Some(quote) = produced_quote {
                self.cache_quote(&cache_key, quote.clone());
                responses.push(quote);
            }
        }

        if !supported {
            return Err(DexIntegrationError::UnsupportedPair(pair.clone()));
        }

        if responses.is_empty() {
            return Err(DexIntegrationError::AllAdaptersFailed(failures));
        }

        responses.sort_by(|a, b| match side {
            OrderSide::Buy => a
                .average_price
                .partial_cmp(&b.average_price)
                .unwrap_or(Ordering::Equal),
            OrderSide::Sell => b
                .average_price
                .partial_cmp(&a.average_price)
                .unwrap_or(Ordering::Equal),
        });

        Ok(responses)
    }

    /// Pulls order books from all supporting venues and hydrates the liquidity aggregator.
    pub fn sync_into_aggregator(
        &mut self,
        pair: &TradingPair,
        aggregator: &mut LiquidityAggregator,
    ) -> Result<usize, DexIntegrationError> {
        if self.adapters.is_empty() {
            return Err(DexIntegrationError::NoAdaptersRegistered);
        }

        let mut updated = 0usize;
        let mut failures = Vec::new();
        let mut supported = false;

        for (id, adapter_state) in self.adapters.iter_mut() {
            if !adapter_state.adapter.supports_pair(pair) {
                continue;
            }

            supported = true;
            match adapter_state.adapter.fetch_order_book(pair) {
                Ok(book) => {
                    aggregator.upsert_venue_book(
                        id.clone(),
                        pair.clone(),
                        book.bids,
                        book.asks,
                    );
                    updated += 1;
                    adapter_state.health.record_success();
                }
                Err(err) => {
                    adapter_state.health.record_failure(&self.config, &err);
                    failures.push((id.clone(), err));
                }
            }
        }

        if !supported {
            return Err(DexIntegrationError::UnsupportedPair(pair.clone()));
        }

        if updated == 0 {
            return Err(DexIntegrationError::AllAdaptersFailed(failures));
        }

        Ok(updated)
    }

    fn cached_quote(&self, key: &QuoteCacheKey, now_ms: u64) -> Option<DexQuote> {
        self.quote_cache.get(key).and_then(|cached| {
            if cached.expires_at_ms >= now_ms {
                Some(cached.quote.clone())
            } else {
                None
            }
        })
    }

    fn cache_quote(&mut self, key: &QuoteCacheKey, quote: DexQuote) {
        let expires_at = now_ms().saturating_add(self.config.quote_ttl_ms);
        self.quote_cache.insert(
            key.clone(),
            CachedQuote {
                quote,
                expires_at_ms: expires_at,
            },
        );
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, RwLock};
    use std::thread;
    use std::time::Duration;

    #[derive(Clone)]
    struct StaticDexAdapter {
        id: String,
        books: Arc<RwLock<HashMap<TradingPair, DexOrderBook>>>,
        gas_units: u64,
        fail_next: Arc<RwLock<bool>>,
        call_count: Arc<RwLock<u32>>,
        permanent_error: Option<DexAdapterError>,
    }

    impl StaticDexAdapter {
        fn new(
            id: &str,
            pair: TradingPair,
            bids: Vec<OrderLevel>,
            asks: Vec<OrderLevel>,
            gas_units: u64,
        ) -> Self {
            let mut books = HashMap::new();
            books.insert(
                pair.clone(),
                DexOrderBook {
                    pair,
                    bids,
                    asks,
                    fetched_at_ms: now_ms(),
                },
            );

            Self {
                id: id.to_string(),
                books: Arc::new(RwLock::new(books)),
                gas_units,
                fail_next: Arc::new(RwLock::new(false)),
                call_count: Arc::new(RwLock::new(0)),
                permanent_error: None,
            }
        }

        fn with_error(id: &str, error: DexAdapterError) -> Self {
            Self {
                id: id.to_string(),
                books: Arc::new(RwLock::new(HashMap::new())),
                gas_units: 0,
                fail_next: Arc::new(RwLock::new(false)),
                call_count: Arc::new(RwLock::new(0)),
                permanent_error: Some(error),
            }
        }

        fn set_fail_next(&self) {
            if let Ok(mut flag) = self.fail_next.write() {
                *flag = true;
            }
        }

        fn calls(&self) -> u32 {
            *self.call_count.read().unwrap()
        }

        fn build_quote(
            &self,
            book: &DexOrderBook,
            side: OrderSide,
            quantity: u64,
        ) -> Result<DexQuote, DexAdapterError> {
            let levels = match side {
                OrderSide::Buy => &book.asks,
                OrderSide::Sell => &book.bids,
            };

            let best_price = levels.first().map(|l| l.price).unwrap_or(0);
            if best_price == 0 {
                return Err(DexAdapterError::InsufficientLiquidity(book.pair.clone()));
            }

            let mut remaining = quantity;
            let mut cost: u128 = 0;
            let mut filled: u64 = 0;
            let mut worst_price = best_price;

            for level in levels {
                if remaining == 0 {
                    break;
                }
                let take = remaining.min(level.quantity);
                remaining -= take;
                filled += take;
                cost += (level.price as u128) * (take as u128);
                worst_price = level.price;
            }

            if remaining > 0 {
                return Err(DexAdapterError::InsufficientLiquidity(book.pair.clone()));
            }

            let average_price = cost as f64 / filled as f64;
            let raw_slippage = match side {
                OrderSide::Buy => (average_price - best_price as f64) / best_price as f64,
                OrderSide::Sell => (best_price as f64 - average_price) / best_price as f64,
            };

            Ok(DexQuote {
                venue_id: self.id.clone(),
                pair: book.pair.clone(),
                side,
                quantity,
                average_price,
                best_price,
                worst_price,
                slippage_bps: raw_slippage * 10_000.0,
                estimated_gas_units: self.gas_units,
                liquidity_available: book.total_available(side),
                fetched_at_ms: now_ms(),
            })
        }
    }

    impl DexAdapter for StaticDexAdapter {
        fn id(&self) -> &str {
            &self.id
        }

        fn supports_pair(&self, pair: &TradingPair) -> bool {
            self.books.read().unwrap().contains_key(pair)
        }

        fn fetch_order_book(&self, pair: &TradingPair) -> Result<DexOrderBook, DexAdapterError> {
            if let Some(err) = &self.permanent_error {
                return Err(err.clone());
            }

            let mut fail_flag = self.fail_next.write().unwrap();
            if *fail_flag {
                *fail_flag = false;
                return Err(DexAdapterError::Network("transient".into()));
            }
            drop(fail_flag);

            self.books
                .read()
                .unwrap()
                .get(pair)
                .cloned()
                .ok_or_else(|| DexAdapterError::UnsupportedPair(pair.clone()))
        }

        fn quote(
            &self,
            pair: &TradingPair,
            side: OrderSide,
            quantity: u64,
        ) -> Result<DexQuote, DexAdapterError> {
            *self.call_count.write().unwrap() += 1;

            if let Some(err) = &self.permanent_error {
                return Err(err.clone());
            }

            let mut fail_flag = self.fail_next.write().unwrap();
            if *fail_flag {
                *fail_flag = false;
                return Err(DexAdapterError::Timeout(100));
            }
            drop(fail_flag);

            let books = self.books.read().unwrap();
            let book = books
                .get(pair)
                .ok_or_else(|| DexAdapterError::UnsupportedPair(pair.clone()))?;

            self.build_quote(book, side, quantity)
        }
    }

    fn sample_pair() -> TradingPair {
        TradingPair {
            base: "ETH".to_string(),
            quote: "USDC".to_string(),
        }
    }

    fn bids(level: u64, qty: u64) -> OrderLevel {
        OrderLevel {
            price: level,
            quantity: qty,
        }
    }

    fn asks(level: u64, qty: u64) -> OrderLevel {
        OrderLevel {
            price: level,
            quantity: qty,
        }
    }

    #[test]
    fn selects_best_quote_across_multiple_venues() {
        let pair = sample_pair();
        let adapter_a = Arc::new(StaticDexAdapter::new(
            "dex-a",
            pair.clone(),
            vec![bids(101, 50)],
            vec![asks(100, 50)],
            120_000,
        ));
        let adapter_b = Arc::new(StaticDexAdapter::new(
            "dex-b",
            pair.clone(),
            vec![bids(102, 100)],
            vec![asks(101, 100)],
            115_000,
        ));

        let mut gateway = DexIntegrationGateway::new(DexIntegrationConfig::default());
        gateway.register_adapter(adapter_a);
        gateway.register_adapter(adapter_b);

        let quotes = gateway
            .fetch_quotes(&pair, OrderSide::Buy, 25)
            .expect("quotes should be returned");
        assert_eq!(quotes.len(), 2);
        assert_eq!(quotes[0].venue_id, "dex-a");
        assert!(quotes[0].average_price < quotes[1].average_price);

        let sell_quotes = gateway
            .fetch_quotes(&pair, OrderSide::Sell, 20)
            .expect("quotes should be returned");
        assert_eq!(sell_quotes[0].venue_id, "dex-b");
        assert!(sell_quotes[0].average_price > sell_quotes[1].average_price);
    }

    #[test]
    fn caches_quotes_until_ttl_expires() {
        let pair = sample_pair();
        let adapter = Arc::new(StaticDexAdapter::new(
            "dex-cache",
            pair.clone(),
            vec![bids(101, 100)],
            vec![asks(100, 100)],
            100_000,
        ));

        let mut gateway = DexIntegrationGateway::new(DexIntegrationConfig {
            quote_ttl_ms: 5_000,
            max_failures_before_unhealthy: 3,
        });
        gateway.register_adapter(adapter.clone());

        let first = gateway.fetch_quotes(&pair, OrderSide::Buy, 10).unwrap();
        gateway.fetch_quotes(&pair, OrderSide::Buy, 10).unwrap();

        assert_eq!(first[0].venue_id, "dex-cache");
        assert_eq!(adapter.calls(), 1, "second call should hit cache");

        // Expire cache and ensure adapter is hit again.
        gateway.config.quote_ttl_ms = 1;
        thread::sleep(Duration::from_millis(2));
        let _ = gateway
            .fetch_quotes(&pair, OrderSide::Buy, 10)
            .unwrap();
        assert_eq!(adapter.calls(), 2);
    }

    #[test]
    fn returns_quotes_when_one_adapter_fails_temporarily() {
        let pair = sample_pair();
        let flakey = Arc::new(StaticDexAdapter::new(
            "flakey",
            pair.clone(),
            vec![bids(100, 50)],
            vec![asks(101, 50)],
            100_000,
        ));
        let reliable = Arc::new(StaticDexAdapter::new(
            "reliable",
            pair.clone(),
            vec![bids(99, 50)],
            vec![asks(100, 50)],
            100_000,
        ));

        flakey.set_fail_next();

        let mut gateway = DexIntegrationGateway::new(DexIntegrationConfig::default());
        gateway.register_adapter(flakey.clone());
        gateway.register_adapter(reliable.clone());

        // First call: flakey fails, reliable succeeds.
        let quotes = gateway
            .fetch_quotes(&pair, OrderSide::Buy, 10)
            .expect("quotes should still be returned");
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].venue_id, "reliable");

        let health = gateway.health("flakey").unwrap();
        assert!(health.healthy);
        assert_eq!(health.consecutive_failures, 1);

        // Second call: flakey recovers and should be included.
        let quotes = gateway
            .fetch_quotes(&pair, OrderSide::Buy, 10)
            .expect("both venues should respond");
        assert_eq!(quotes.len(), 2);
        let recovered = gateway.health("flakey").unwrap();
        assert!(recovered.consecutive_failures == 0);
    }

    #[test]
    fn hydrates_liquidity_aggregator_from_multiple_adapters() {
        let pair = sample_pair();
        let adapter_a = Arc::new(StaticDexAdapter::new(
            "dex-a",
            pair.clone(),
            vec![bids(100, 10)],
            vec![asks(101, 10)],
            110_000,
        ));
        let adapter_b = Arc::new(StaticDexAdapter::new(
            "dex-b",
            pair.clone(),
            vec![bids(99, 30)],
            vec![asks(102, 30)],
            110_000,
        ));

        let mut gateway = DexIntegrationGateway::new(DexIntegrationConfig::default());
        gateway.register_adapter(adapter_a);
        gateway.register_adapter(adapter_b);

        let mut aggregator = LiquidityAggregator::new();
        let updated = gateway
            .sync_into_aggregator(&pair, &mut aggregator)
            .expect("should hydrate aggregator");
        assert_eq!(updated, 2);

        let book = aggregator.aggregated_book(&pair).expect("book present");
        assert_eq!(book.bids.len(), 2);
        assert_eq!(book.asks.len(), 2);
        assert_eq!(book.bids[0].price, 100);
        assert_eq!(book.asks[0].price, 101);
    }

    #[test]
    fn marks_adapter_unhealthy_after_repeated_failures() {
        let pair = sample_pair();
        let failing = Arc::new(StaticDexAdapter::with_error(
            "fail-venue",
            DexAdapterError::Network("offline".into()),
        ));

        let mut gateway = DexIntegrationGateway::new(DexIntegrationConfig {
            quote_ttl_ms: 0,
            max_failures_before_unhealthy: 1,
        });
        gateway.register_adapter(failing.clone());

        let result = gateway.fetch_quotes(&pair, OrderSide::Buy, 1);
        assert!(matches!(
            result,
            Err(DexIntegrationError::AllAdaptersFailed(_))
        ));

        let health = gateway.health("fail-venue").unwrap();
        assert!(!health.healthy);
        assert!(health.consecutive_failures >= 1);
        assert!(health.last_error.unwrap().contains("offline"));
    }

    #[test]
    fn errors_on_unsupported_pair() {
        let pair = sample_pair();
        let adapter = Arc::new(StaticDexAdapter::new(
            "dex-a",
            TradingPair {
                base: "BTC".into(),
                quote: "USDT".into(),
            },
            vec![bids(10, 10)],
            vec![asks(11, 10)],
            90_000,
        ));

        let mut gateway = DexIntegrationGateway::new(DexIntegrationConfig::default());
        gateway.register_adapter(adapter);

        let err = gateway
            .fetch_quotes(&pair, OrderSide::Buy, 1)
            .unwrap_err();
        assert!(matches!(err, DexIntegrationError::UnsupportedPair(_)));
    }
}
