//! Database layer for the DEX-OS core engine
//!
//! This module provides database functionality for persisting orders,
//! trades, and other DEX-related data with sharding capabilities.

use dex_core::types::{Order, OrderId, Trade, TradeId, TraderId, TradingPair};
use sqlx::{query, Row};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::collections::HashMap;
use thiserror::Error;

pub mod migrations;

/// Database manager for the DEX with sharding support
#[derive(Clone)]
pub struct DatabaseManager {
    /// Primary connection pool for non-sharded data
    primary_pool: PgPool,
    /// Sharded connection pools for partitioned data
    shard_pools: HashMap<u64, PgPool>,
    /// Number of shards configured
    num_shards: u64,
}

impl DatabaseManager {
    /// Establish a new connection pool using the provided database URL.
    pub async fn connect(database_url: &str) -> Result<Self, DatabaseError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        Ok(Self {
            primary_pool: pool,
            shard_pools: HashMap::new(),
            num_shards: 1, // Default to 1 shard
        })
    }

    /// Establish a new connection with sharding support
    pub async fn connect_with_sharding(
        primary_database_url: &str,
        shard_urls: HashMap<u64, String>,
    ) -> Result<Self, DatabaseError> {
        let primary_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(primary_database_url)
            .await?;

        let mut shard_pools = HashMap::new();
        for (shard_id, url) in shard_urls {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&url)
                .await?;
            shard_pools.insert(shard_id, pool);
        }

        let num_shards = shard_pools.len() as u64;
        Ok(Self {
            primary_pool,
            shard_pools,
            num_shards,
        })
    }

    /// Create a new database manager with the provided connection pool
    pub fn new(pool: PgPool) -> Self {
        Self {
            primary_pool: pool,
            shard_pools: HashMap::new(),
            num_shards: 1,
        }
    }

    /// Create a new database manager with sharding support
    pub fn new_with_shards(primary_pool: PgPool, shard_pools: HashMap<u64, PgPool>) -> Self {
        let num_shards = shard_pools.len() as u64;
        Self {
            primary_pool,
            shard_pools,
            num_shards,
        }
    }

    /// Initialize the database schema
    pub async fn initialize(&self) -> Result<(), DatabaseError> {
        // Run migrations on primary database
        migrations::run_migrations(&self.primary_pool).await?;

        // Run migrations on each shard
        for (_, pool) in &self.shard_pools {
            migrations::run_migrations(pool).await?;
        }

        Ok(())
    }

    /// Get shard ID for a given key (simple hash-based sharding)
    fn get_shard_id(&self, key: u64) -> u64 {
        key % self.num_shards
    }

    /// Get the appropriate database pool for a given shard ID
    fn get_pool_for_shard(&self, shard_id: u64) -> &PgPool {
        self.shard_pools
            .get(&shard_id)
            .unwrap_or(&self.primary_pool)
    }

    /// Save an order to the database
    pub async fn save_order(&self, order: &Order) -> Result<(), DatabaseError> {
        // Determine which shard to use based on trader ID hash
        let shard_id = self.get_shard_id(order.trader_id.len() as u64);
        let pool = self.get_pool_for_shard(shard_id);

        query(
            r#"
            INSERT INTO orders (
                id, trader_id, base_token, quote_token, side, order_type, price, quantity, timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET
                trader_id = $2,
                base_token = $3,
                quote_token = $4,
                side = $5,
                order_type = $6,
                price = $7,
                quantity = $8,
                timestamp = $9
            "#,
        )
        .bind(order.id as i64)
        .bind(&order.trader_id)
        .bind(&order.pair.base)
        .bind(&order.pair.quote)
        .bind(match order.side {
            dex_core::types::OrderSide::Buy => "buy",
            dex_core::types::OrderSide::Sell => "sell",
        })
        .bind(match order.order_type {
            dex_core::types::OrderType::Limit => "limit",
            dex_core::types::OrderType::Market => "market",
        })
        .bind(order.price.map(|p| p as i64))
        .bind(order.quantity as i64)
        .bind(order.timestamp as i64)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Load an order from the database by ID
    pub async fn load_order(&self, order_id: OrderId) -> Result<Option<Order>, DatabaseError> {
        // Try to load from each shard until found
        for (_, pool) in &self.shard_pools {
            let row = query(
                r#"
                SELECT 
                    id, trader_id, base_token, quote_token, side, order_type, price, quantity, timestamp
                FROM orders
                WHERE id = $1
                "#,
            )
            .bind(order_id as i64)
            .fetch_optional(pool)
            .await?;

            if let Some(row) = row {
                let price: Option<i64> = row.get("price");
                let order = Order {
                    id: row.get::<i64, _>("id") as u64,
                    trader_id: row.get("trader_id"),
                    pair: TradingPair {
                        base: row.get("base_token"),
                        quote: row.get("quote_token"),
                    },
                    side: match row.get::<&str, _>("side") {
                        "buy" => dex_core::types::OrderSide::Buy,
                        "sell" => dex_core::types::OrderSide::Sell,
                        _ => return Err(DatabaseError::DataIntegrityError),
                    },
                    order_type: match row.get::<&str, _>("order_type") {
                        "limit" => dex_core::types::OrderType::Limit,
                        "market" => dex_core::types::OrderType::Market,
                        _ => return Err(DatabaseError::DataIntegrityError),
                    },
                    price: price.map(|p| p as u64),
                    quantity: row.get::<i64, _>("quantity") as u64,
                    timestamp: row.get::<i64, _>("timestamp") as u64,
                };
                return Ok(Some(order));
            }
        }

        // If not found in shards, try primary database
        let row = query(
            r#"
            SELECT 
                id, trader_id, base_token, quote_token, side, order_type, price, quantity, timestamp
            FROM orders
            WHERE id = $1
            "#,
        )
        .bind(order_id as i64)
        .fetch_optional(&self.primary_pool)
        .await?;

        if let Some(row) = row {
            let price: Option<i64> = row.get("price");
            let order = Order {
                id: row.get::<i64, _>("id") as u64,
                trader_id: row.get("trader_id"),
                pair: TradingPair {
                    base: row.get("base_token"),
                    quote: row.get("quote_token"),
                },
                side: match row.get::<&str, _>("side") {
                    "buy" => dex_core::types::OrderSide::Buy,
                    "sell" => dex_core::types::OrderSide::Sell,
                    _ => return Err(DatabaseError::DataIntegrityError),
                },
                order_type: match row.get::<&str, _>("order_type") {
                    "limit" => dex_core::types::OrderType::Limit,
                    "market" => dex_core::types::OrderType::Market,
                    _ => return Err(DatabaseError::DataIntegrityError),
                },
                price: price.map(|p| p as u64),
                quantity: row.get::<i64, _>("quantity") as u64,
                timestamp: row.get::<i64, _>("timestamp") as u64,
            };
            Ok(Some(order))
        } else {
            Ok(None)
        }
    }

    /// Delete an order from the database
    pub async fn delete_order(&self, order_id: OrderId) -> Result<bool, DatabaseError> {
        // Try to delete from each shard
        for (_, pool) in &self.shard_pools {
            let result = query("DELETE FROM orders WHERE id = $1")
                .bind(order_id as i64)
                .execute(pool)
                .await?;

            if result.rows_affected() > 0 {
                return Ok(true);
            }
        }

        // If not found in shards, try primary database
        let result = query("DELETE FROM orders WHERE id = $1")
            .bind(order_id as i64)
            .execute(&self.primary_pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Save a trade to the database
    pub async fn save_trade(&self, trade: &Trade) -> Result<(), DatabaseError> {
        // Determine which shard to use based on maker order ID
        let shard_id = self.get_shard_id(trade.maker_order_id);
        let pool = self.get_pool_for_shard(shard_id);

        query(
            r#"
            INSERT INTO trades (
                id, maker_order_id, taker_order_id, base_token, quote_token, price, quantity, timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(trade.id as i64)
        .bind(trade.maker_order_id as i64)
        .bind(trade.taker_order_id as i64)
        .bind(&trade.base_token)
        .bind(&trade.quote_token)
        .bind(trade.price as i64)
        .bind(trade.quantity as i64)
        .bind(trade.timestamp as i64)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Load a trade from the database by ID
    pub async fn load_trade(&self, trade_id: TradeId) -> Result<Option<Trade>, DatabaseError> {
        // Try to load from each shard until found
        for (_, pool) in &self.shard_pools {
            let row = query(
                r#"
                SELECT 
                    id, maker_order_id, taker_order_id, base_token, quote_token, price, quantity, timestamp
                FROM trades
                WHERE id = $1
                "#,
            )
            .bind(trade_id as i64)
            .fetch_optional(pool)
            .await?;

            if let Some(row) = row {
                let trade = Trade {
                    id: row.get::<i64, _>("id") as u64,
                    maker_order_id: row.get::<i64, _>("maker_order_id") as u64,
                    taker_order_id: row.get::<i64, _>("taker_order_id") as u64,
                    base_token: row.get("base_token"),
                    quote_token: row.get("quote_token"),
                    price: row.get::<i64, _>("price") as u64,
                    quantity: row.get::<i64, _>("quantity") as u64,
                    timestamp: row.get::<i64, _>("timestamp") as u64,
                };
                return Ok(Some(trade));
            }
        }

        // If not found in shards, try primary database
        let row = query(
            r#"
            SELECT 
                id, maker_order_id, taker_order_id, base_token, quote_token, price, quantity, timestamp
            FROM trades
            WHERE id = $1
            "#,
        )
        .bind(trade_id as i64)
        .fetch_optional(&self.primary_pool)
        .await?;

        if let Some(row) = row {
            let trade = Trade {
                id: row.get::<i64, _>("id") as u64,
                maker_order_id: row.get::<i64, _>("maker_order_id") as u64,
                taker_order_id: row.get::<i64, _>("taker_order_id") as u64,
                base_token: row.get("base_token"),
                quote_token: row.get("quote_token"),
                price: row.get::<i64, _>("price") as u64,
                quantity: row.get::<i64, _>("quantity") as u64,
                timestamp: row.get::<i64, _>("timestamp") as u64,
            };
            Ok(Some(trade))
        } else {
            Ok(None)
        }
    }

    /// Get all trades for a specific order
    pub async fn get_trades_for_order(
        &self,
        order_id: OrderId,
    ) -> Result<Vec<Trade>, DatabaseError> {
        let mut all_trades = Vec::new();

        // Search in all shards
        for (_, pool) in &self.shard_pools {
            let rows = query(
                r#"
                SELECT 
                    id, maker_order_id, taker_order_id, base_token, quote_token, price, quantity, timestamp
                FROM trades
                WHERE maker_order_id = $1 OR taker_order_id = $1
                ORDER BY timestamp ASC
                "#,
            )
            .bind(order_id as i64)
            .fetch_all(pool)
            .await?;

            for row in rows {
                let trade = Trade {
                    id: row.get::<i64, _>("id") as u64,
                    maker_order_id: row.get::<i64, _>("maker_order_id") as u64,
                    taker_order_id: row.get::<i64, _>("taker_order_id") as u64,
                    base_token: row.get("base_token"),
                    quote_token: row.get("quote_token"),
                    price: row.get::<i64, _>("price") as u64,
                    quantity: row.get::<i64, _>("quantity") as u64,
                    timestamp: row.get::<i64, _>("timestamp") as u64,
                };
                all_trades.push(trade);
            }
        }

        // Also search in primary database
        let rows = query(
            r#"
            SELECT 
                id, maker_order_id, taker_order_id, base_token, quote_token, price, quantity, timestamp
            FROM trades
            WHERE maker_order_id = $1 OR taker_order_id = $1
            ORDER BY timestamp ASC
            "#,
        )
        .bind(order_id as i64)
        .fetch_all(&self.primary_pool)
        .await?;

        for row in rows {
            let trade = Trade {
                id: row.get::<i64, _>("id") as u64,
                maker_order_id: row.get::<i64, _>("maker_order_id") as u64,
                taker_order_id: row.get::<i64, _>("taker_order_id") as u64,
                base_token: row.get("base_token"),
                quote_token: row.get("quote_token"),
                price: row.get::<i64, _>("price") as u64,
                quantity: row.get::<i64, _>("quantity") as u64,
                timestamp: row.get::<i64, _>("timestamp") as u64,
            };
            all_trades.push(trade);
        }

        // Sort by timestamp
        all_trades.sort_by_key(|trade| trade.timestamp);

        Ok(all_trades)
    }

    /// Get all trades for a specific trader
    pub async fn get_trades_for_trader(
        &self,
        trader_id: &TraderId,
    ) -> Result<Vec<Trade>, DatabaseError> {
        let mut all_trades = Vec::new();

        // Search in all shards
        for (_, pool) in &self.shard_pools {
            let rows = query(
                r#"
                SELECT 
                    t.id, t.maker_order_id, t.taker_order_id, t.base_token, t.quote_token, t.price, t.quantity, t.timestamp
                FROM trades t
                JOIN orders o1 ON t.maker_order_id = o1.id
                JOIN orders o2 ON t.taker_order_id = o2.id
                WHERE o1.trader_id = $1 OR o2.trader_id = $1
                ORDER BY t.timestamp ASC
                "#,
            )
            .bind(trader_id)
            .fetch_all(pool)
            .await?;

            for row in rows {
                let trade = Trade {
                    id: row.get::<i64, _>("id") as u64,
                    maker_order_id: row.get::<i64, _>("maker_order_id") as u64,
                    taker_order_id: row.get::<i64, _>("taker_order_id") as u64,
                    base_token: row.get("base_token"),
                    quote_token: row.get("quote_token"),
                    price: row.get::<i64, _>("price") as u64,
                    quantity: row.get::<i64, _>("quantity") as u64,
                    timestamp: row.get::<i64, _>("timestamp") as u64,
                };
                all_trades.push(trade);
            }
        }

        // Also search in primary database
        let rows = query(
            r#"
            SELECT 
                t.id, t.maker_order_id, t.taker_order_id, t.base_token, t.quote_token, t.price, t.quantity, t.timestamp
            FROM trades t
            JOIN orders o1 ON t.maker_order_id = o1.id
            JOIN orders o2 ON t.taker_order_id = o2.id
            WHERE o1.trader_id = $1 OR o2.trader_id = $1
            ORDER BY t.timestamp ASC
            "#,
        )
        .bind(trader_id)
        .fetch_all(&self.primary_pool)
        .await?;

        for row in rows {
            let trade = Trade {
                id: row.get::<i64, _>("id") as u64,
                maker_order_id: row.get::<i64, _>("maker_order_id") as u64,
                taker_order_id: row.get::<i64, _>("taker_order_id") as u64,
                base_token: row.get("base_token"),
                quote_token: row.get("quote_token"),
                price: row.get::<i64, _>("price") as u64,
                quantity: row.get::<i64, _>("quantity") as u64,
                timestamp: row.get::<i64, _>("timestamp") as u64,
            };
            all_trades.push(trade);
        }

        // Sort by timestamp
        all_trades.sort_by_key(|trade| trade.timestamp);

        Ok(all_trades)
    }

    /// Get shard statistics
    pub async fn get_shard_statistics(
        &self,
    ) -> Result<HashMap<u64, ShardStatistics>, DatabaseError> {
        let mut stats = HashMap::new();

        // Get stats for primary database
        let row = query(
            r#"
            SELECT 
                COUNT(*) as order_count,
                COUNT(DISTINCT trader_id) as trader_count
            FROM orders
            "#,
        )
        .fetch_one(&self.primary_pool)
        .await?;

        stats.insert(
            0, // Primary shard ID
            ShardStatistics {
                shard_id: 0,
                order_count: row.get::<i64, _>("order_count") as u64,
                trader_count: row.get::<i64, _>("trader_count") as u64,
                trade_count: 0, // Would need another query to get this
            },
        );

        // Get stats for each shard
        for (shard_id, pool) in &self.shard_pools {
            let row = query(
                r#"
                SELECT 
                    COUNT(*) as order_count,
                    COUNT(DISTINCT trader_id) as trader_count
                FROM orders
                "#,
            )
            .fetch_one(pool)
            .await?;

            stats.insert(
                *shard_id,
                ShardStatistics {
                    shard_id: *shard_id,
                    order_count: row.get::<i64, _>("order_count") as u64,
                    trader_count: row.get::<i64, _>("trader_count") as u64,
                    trade_count: 0, // Would need another query to get this
                },
            );
        }

        Ok(stats)
    }
}

/// Statistics for a database shard
#[derive(Debug, Clone)]
pub struct ShardStatistics {
    pub shard_id: u64,
    pub order_count: u64,
    pub trader_count: u64,
    pub trade_count: u64,
}

/// Errors that can occur when working with the database
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Data integrity error")]
    DataIntegrityError,
}

#[cfg(test)]
impl DatabaseManager {
    /// Create a database manager backed by a lazily connected pool. Useful in tests that do
    /// not exercise the database but need a handle for wiring filters.
    pub fn connect_lazy(database_url: &str) -> Result<Self, DatabaseError> {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy(database_url)?;
        Ok(Self {
            primary_pool: pool,
            shard_pools: HashMap::new(),
            num_shards: 1,
        })
    }
}
