//! Database migrations for the DEX-OS core engine
//!
//! This module provides functionality for database schema evolution.

use sqlx::{query, Row};
use sqlx::PgPool;

/// Represents a database migration
pub struct Migration {
    pub version: i32,
    pub description: &'static str,
    pub sql: &'static str,
}

/// Get all migrations
pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "Create initial schema with orders table",
            sql: r#"
                CREATE TABLE IF NOT EXISTS orders (
                    id BIGINT PRIMARY KEY,
                    trader_id TEXT NOT NULL,
                    base_token TEXT NOT NULL,
                    quote_token TEXT NOT NULL,
                    side TEXT NOT NULL,
                    order_type TEXT NOT NULL,
                    price BIGINT,
                    quantity BIGINT NOT NULL,
                    timestamp BIGINT NOT NULL
                )
            "#,
        },
        Migration {
            version: 2,
            description: "Create trades table",
            sql: r#"
                CREATE TABLE IF NOT EXISTS trades (
                    id BIGINT PRIMARY KEY,
                    maker_order_id BIGINT NOT NULL,
                    taker_order_id BIGINT NOT NULL,
                    base_token TEXT NOT NULL,
                    quote_token TEXT NOT NULL,
                    price BIGINT NOT NULL,
                    quantity BIGINT NOT NULL,
                    timestamp BIGINT NOT NULL
                )
            "#,
        },
        Migration {
            version: 3,
            description: "Add index on orders table for trader_id",
            sql: r#"
                CREATE INDEX IF NOT EXISTS idx_orders_trader_id ON orders (trader_id)
            "#,
        },
        Migration {
            version: 4,
            description: "Add index on trades table for maker_order_id and taker_order_id",
            sql: r#"
                CREATE INDEX IF NOT EXISTS idx_trades_maker_order_id ON trades (maker_order_id);
                CREATE INDEX IF NOT EXISTS idx_trades_taker_order_id ON trades (taker_order_id)
            "#,
        },
    ]
}

/// Run all pending migrations
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Create migrations table if it doesn't exist
    query(
        r#"
        CREATE TABLE IF NOT EXISTS migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TIMESTAMP NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Get the current schema version
    let current_version: i32 = match query("SELECT MAX(version) FROM migrations")
        .fetch_optional(pool)
        .await?
    {
        Some(row) => row.get::<Option<i32>, _>(0).unwrap_or(0),
        None => 0,
    };

    // Get all migrations
    let migrations = get_migrations();

    // Run pending migrations
    for migration in migrations {
        if migration.version > current_version {
            println!(
                "Running migration {}: {}",
                migration.version, migration.description
            );

            // Run the migration SQL
            query(migration.sql).execute(pool).await?;

            // Record the migration
            query("INSERT INTO migrations (version, description) VALUES ($1, $2)")
                .bind(migration.version)
                .bind(migration.description)
                .execute(pool)
                .await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_migrations() {
        let migrations = get_migrations();
        assert!(!migrations.is_empty());
        assert_eq!(migrations[0].version, 1);
    }
}
