// Database Infrastructure Module
// Security: Layer 9 - Database Security
// Implements B+ Tree indexing and ACID transactions

pub mod bplus_tree;
pub mod transaction_manager;
pub mod connection_pool;

pub use bplus_tree::{BPlusTree, BPlusTreeIndex};
pub use transaction_manager::{
    TransactionManager, Transaction, TransactionId, IsolationLevel,
    TransactionState, LockManager, DeadlockDetector
};
pub use connection_pool::ConnectionPool;

use std::sync::Arc;
use parking_lot::RwLock;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// B+ tree order (branching factor)
    pub btree_order: usize,
    /// Maximum number of concurrent transactions
    pub max_transactions: usize,
    /// Default isolation level
    pub default_isolation: IsolationLevel,
    /// Enable deadlock detection
    pub enable_deadlock_detection: bool,
    /// Connection pool size
    pub pool_size: usize,
    /// Transaction timeout in milliseconds
    pub transaction_timeout_ms: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            btree_order: 128,
            max_transactions: 10000,
            default_isolation: IsolationLevel::Serializable,
            enable_deadlock_detection: true,
            pool_size: 100,
            transaction_timeout_ms: 30000,
        }
    }
}

/// Main database instance
pub struct Database {
    config: DatabaseConfig,
    transaction_manager: Arc<RwLock<TransactionManager>>,
    indexes: Arc<RwLock<std::collections::HashMap<String, Arc<RwLock<BPlusTreeIndex>>>>>,
    connection_pool: Arc<ConnectionPool>,
}

impl Database {
    /// Create a new database instance
    pub fn new(config: DatabaseConfig) -> Self {
        let transaction_manager = Arc::new(RwLock::new(
            TransactionManager::new(
                config.max_transactions,
                config.default_isolation,
                config.enable_deadlock_detection,
                config.transaction_timeout_ms,
            )
        ));

        let connection_pool = Arc::new(ConnectionPool::new(config.pool_size));

        Self {
            config: config.clone(),
            transaction_manager,
            indexes: Arc::new(RwLock::new(std::collections::HashMap::new())),
            connection_pool,
        }
    }

    /// Create a new index
    pub fn create_index(&self, name: String) -> Result<(), DatabaseError> {
        let mut indexes = self.indexes.write();
        if indexes.contains_key(&name) {
            return Err(DatabaseError::IndexAlreadyExists(name));
        }

        let index = Arc::new(RwLock::new(BPlusTreeIndex::new(self.config.btree_order)));
        indexes.insert(name, index);
        Ok(())
    }

    /// Get an index by name
    pub fn get_index(&self, name: &str) -> Result<Arc<RwLock<BPlusTreeIndex>>, DatabaseError> {
        let indexes = self.indexes.read();
        indexes.get(name)
            .cloned()
            .ok_or_else(|| DatabaseError::IndexNotFound(name.to_string()))
    }

    /// Begin a new transaction
    pub fn begin_transaction(&self, isolation: Option<IsolationLevel>) -> Result<TransactionId, DatabaseError> {
        let mut tm = self.transaction_manager.write();
        tm.begin_transaction(isolation)
            .map_err(DatabaseError::TransactionError)
    }

    /// Commit a transaction
    pub fn commit_transaction(&self, txn_id: TransactionId) -> Result<(), DatabaseError> {
        let mut tm = self.transaction_manager.write();
        tm.commit_transaction(txn_id)
            .map_err(DatabaseError::TransactionError)
    }

    /// Rollback a transaction
    pub fn rollback_transaction(&self, txn_id: TransactionId) -> Result<(), DatabaseError> {
        let mut tm = self.transaction_manager.write();
        tm.rollback_transaction(txn_id)
            .map_err(DatabaseError::TransactionError)
    }

    /// Get transaction manager
    pub fn transaction_manager(&self) -> Arc<RwLock<TransactionManager>> {
        Arc::clone(&self.transaction_manager)
    }

    /// Get connection pool
    pub fn connection_pool(&self) -> Arc<ConnectionPool> {
        Arc::clone(&self.connection_pool)
    }

    /// Get database statistics
    pub fn get_stats(&self) -> DatabaseStats {
        let tm = self.transaction_manager.read();
        let indexes = self.indexes.read();

        DatabaseStats {
            active_transactions: tm.active_transaction_count(),
            total_indexes: indexes.len(),
            pool_size: self.config.pool_size,
            pool_active: self.connection_pool.active_connections(),
        }
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub active_transactions: usize,
    pub total_indexes: usize,
    pub pool_size: usize,
    pub pool_active: usize,
}

/// Database errors
#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseError {
    IndexNotFound(String),
    IndexAlreadyExists(String),
    TransactionError(String),
    ConnectionPoolExhausted,
    InvalidConfiguration(String),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::IndexNotFound(name) => write!(f, "Index not found: {}", name),
            DatabaseError::IndexAlreadyExists(name) => write!(f, "Index already exists: {}", name),
            DatabaseError::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
            DatabaseError::ConnectionPoolExhausted => write!(f, "Connection pool exhausted"),
            DatabaseError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
        }
    }
}

impl std::error::Error for DatabaseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let config = DatabaseConfig::default();
        let db = Database::new(config);
        let stats = db.get_stats();
        assert_eq!(stats.active_transactions, 0);
        assert_eq!(stats.total_indexes, 0);
    }

    #[test]
    fn test_index_creation() {
        let config = DatabaseConfig::default();
        let db = Database::new(config);
        
        assert!(db.create_index("test_index".to_string()).is_ok());
        assert!(db.get_index("test_index").is_ok());
        
        // Duplicate index should fail
        assert!(db.create_index("test_index".to_string()).is_err());
    }

    #[test]
    fn test_transaction_lifecycle() {
        let config = DatabaseConfig::default();
        let db = Database::new(config);
        
        let txn_id = db.begin_transaction(None).unwrap();
        assert_eq!(db.get_stats().active_transactions, 1);
        
        db.commit_transaction(txn_id).unwrap();
        assert_eq!(db.get_stats().active_transactions, 0);
    }
}
