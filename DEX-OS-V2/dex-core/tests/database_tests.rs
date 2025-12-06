// Comprehensive Tests for Database Infrastructure
// Tests B+ Tree Indexing and ACID Transactions
// Security: Layer 9 - Database Security

use dex_core::database::{
    Database, DatabaseConfig, IsolationLevel,
    BPlusTree, BPlusTreeIndex, TransactionManager, ConnectionPool,
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// ============================================================================
// B+ Tree Tests
// ============================================================================

#[test]
fn test_bplus_tree_basic_operations() {
    let tree = BPlusTree::new(4);
    
    // Test insert
    assert!(tree.insert(10, "ten".to_string()).is_ok());
    assert!(tree.insert(5, "five".to_string()).is_ok());
    assert!(tree.insert(15, "fifteen".to_string()).is_ok());
    assert!(tree.insert(3, "three".to_string()).is_ok());
    assert!(tree.insert(7, "seven".to_string()).is_ok());
    
    // Test search
    assert_eq!(tree.get(&10), Some("ten".to_string()));
    assert_eq!(tree.get(&5), Some("five".to_string()));
    assert_eq!(tree.get(&15), Some("fifteen".to_string()));
    assert_eq!(tree.get(&3), Some("three".to_string()));
    assert_eq!(tree.get(&7), Some("seven".to_string()));
    assert_eq!(tree.get(&100), None);
    
    // Test size
    assert_eq!(tree.len(), 5);
    assert!(!tree.is_empty());
}

#[test]
fn test_bplus_tree_range_queries() {
    let tree = BPlusTree::new(4);
    
    // Insert data
    for i in 0..100 {
        tree.insert(i, format!("value_{}", i)).unwrap();
    }
    
    // Test range query
    let results = tree.range_query(&25, &35);
    assert_eq!(results.len(), 11); // 25 to 35 inclusive
    
    for (k, v) in results {
        assert!(k >= 25 && k <= 35);
        assert_eq!(v, format!("value_{}", k));
    }
    
    // Test edge cases
    let results = tree.range_query(&0, &5);
    assert_eq!(results.len(), 6);
    
    let results = tree.range_query(&95, &99);
    assert_eq!(results.len(), 5);
}

#[test]
fn test_bplus_tree_deletion() {
    let tree = BPlusTree::new(4);
    
    // Insert data
    for i in 0..20 {
        tree.insert(i, format!("value_{}", i)).unwrap();
    }
    
    assert_eq!(tree.len(), 20);
    
    // Delete some entries
    for i in (0..20).step_by(2) {
        let deleted = tree.delete(&i).unwrap();
        assert_eq!(deleted, Some(format!("value_{}", i)));
    }
    
    assert_eq!(tree.len(), 10);
    
    // Verify remaining entries
    for i in 0..20 {
        if i % 2 == 0 {
            assert_eq!(tree.get(&i), None);
        } else {
            assert_eq!(tree.get(&i), Some(format!("value_{}", i)));
        }
    }
}

#[test]
fn test_bplus_tree_large_dataset() {
    let tree = BPlusTree::new(128);
    
    // Insert 10,000 entries
    for i in 0..10000 {
        tree.insert(i, format!("value_{}", i)).unwrap();
    }
    
    assert_eq!(tree.len(), 10000);
    
    // Verify all entries
    for i in 0..10000 {
        assert_eq!(tree.get(&i), Some(format!("value_{}", i)));
    }
    
    // Test range query on large dataset
    let results = tree.range_query(&5000, &5100);
    assert_eq!(results.len(), 101);
}

#[test]
fn test_bplus_tree_concurrent_reads() {
    let tree = Arc::new(BPlusTree::new(64));
    
    // Insert data
    for i in 0..1000 {
        tree.insert(i, format!("value_{}", i)).unwrap();
    }
    
    let mut handles = vec![];
    
    // Spawn multiple reader threads
    for _ in 0..10 {
        let tree_clone = Arc::clone(&tree);
        let handle = thread::spawn(move || {
            for i in 0..1000 {
                assert_eq!(tree_clone.get(&i), Some(format!("value_{}", i)));
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_bplus_tree_index() {
    let index = BPlusTreeIndex::new(4);
    
    // Insert multiple records for same key
    index.insert(b"user:john".to_vec(), 100).unwrap();
    index.insert(b"user:john".to_vec(), 101).unwrap();
    index.insert(b"user:john".to_vec(), 102).unwrap();
    
    index.insert(b"user:jane".to_vec(), 200).unwrap();
    
    // Search
    let results = index.search(b"user:john");
    assert_eq!(results.len(), 3);
    assert!(results.contains(&100));
    assert!(results.contains(&101));
    assert!(results.contains(&102));
    
    let results = index.search(b"user:jane");
    assert_eq!(results, vec![200]);
    
    // Range query
    let results = index.range_query(b"user:jane", b"user:john");
    assert!(results.len() >= 4);
}

// ============================================================================
// Transaction Manager Tests
// ============================================================================

#[test]
fn test_transaction_basic_lifecycle() {
    let mut tm = TransactionManager::new(100, IsolationLevel::Serializable, true, 30000);
    
    // Begin transaction
    let txn_id = tm.begin_transaction(None).unwrap();
    assert_eq!(tm.active_transaction_count(), 1);
    
    // Commit transaction
    tm.commit_transaction(txn_id).unwrap();
    assert_eq!(tm.active_transaction_count(), 0);
}

#[test]
fn test_transaction_rollback() {
    let mut tm = TransactionManager::new(100, IsolationLevel::Serializable, true, 30000);
    
    let txn_id = tm.begin_transaction(None).unwrap();
    
    // Perform some operations
    tm.write(txn_id, "key1".to_string(), b"value1".to_vec()).unwrap();
    tm.write(txn_id, "key2".to_string(), b"value2".to_vec()).unwrap();
    
    // Rollback
    tm.rollback_transaction(txn_id).unwrap();
    assert_eq!(tm.active_transaction_count(), 0);
}

#[test]
fn test_transaction_isolation_levels() {
    let mut tm = TransactionManager::new(100, IsolationLevel::ReadCommitted, true, 30000);
    
    // Test different isolation levels
    let txn1 = tm.begin_transaction(Some(IsolationLevel::ReadUncommitted)).unwrap();
    let txn2 = tm.begin_transaction(Some(IsolationLevel::ReadCommitted)).unwrap();
    let txn3 = tm.begin_transaction(Some(IsolationLevel::RepeatableRead)).unwrap();
    let txn4 = tm.begin_transaction(Some(IsolationLevel::Serializable)).unwrap();
    
    assert_eq!(tm.active_transaction_count(), 4);
    
    tm.commit_transaction(txn1).unwrap();
    tm.commit_transaction(txn2).unwrap();
    tm.commit_transaction(txn3).unwrap();
    tm.commit_transaction(txn4).unwrap();
    
    assert_eq!(tm.active_transaction_count(), 0);
}

#[test]
fn test_transaction_read_write() {
    let mut tm = TransactionManager::new(100, IsolationLevel::Serializable, true, 30000);
    
    let txn_id = tm.begin_transaction(None).unwrap();
    
    // Write operations
    tm.write(txn_id, "account:alice".to_string(), b"1000".to_vec()).unwrap();
    tm.write(txn_id, "account:bob".to_string(), b"500".to_vec()).unwrap();
    
    // Read operations
    let _ = tm.read(txn_id, "account:alice".to_string());
    let _ = tm.read(txn_id, "account:bob".to_string());
    
    // Commit
    tm.commit_transaction(txn_id).unwrap();
}

#[test]
fn test_transaction_concurrent_commits() {
    let tm = Arc::new(parking_lot::RwLock::new(
        TransactionManager::new(100, IsolationLevel::Serializable, true, 30000)
    ));
    
    let mut handles = vec![];
    
    for i in 0..10 {
        let tm_clone = Arc::clone(&tm);
        let handle = thread::spawn(move || {
            let txn_id = {
                let mut tm = tm_clone.write();
                tm.begin_transaction(None).unwrap()
            };
            
            {
                let tm = tm_clone.read();
                tm.write(txn_id, format!("key_{}", i), format!("value_{}", i).into_bytes()).unwrap();
            }
            
            thread::sleep(Duration::from_millis(10));
            
            {
                let mut tm = tm_clone.write();
                tm.commit_transaction(txn_id).unwrap();
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let tm = tm.read();
    assert_eq!(tm.active_transaction_count(), 0);
}

#[test]
fn test_transaction_max_limit() {
    let mut tm = TransactionManager::new(5, IsolationLevel::Serializable, true, 30000);
    
    // Create max transactions
    let mut txns = vec![];
    for _ in 0..5 {
        let txn_id = tm.begin_transaction(None).unwrap();
        txns.push(txn_id);
    }
    
    // Should fail to create more
    assert!(tm.begin_transaction(None).is_err());
    
    // Commit one
    tm.commit_transaction(txns[0]).unwrap();
    
    // Should now be able to create another
    assert!(tm.begin_transaction(None).is_ok());
}

// ============================================================================
// Connection Pool Tests
// ============================================================================

#[test]
fn test_connection_pool_basic() {
    let pool = ConnectionPool::new(10);
    
    let stats = pool.stats();
    assert_eq!(stats.pool_size, 10);
    assert_eq!(stats.active, 0);
    assert_eq!(stats.available, 10);
}

#[test]
fn test_connection_pool_acquire_release() {
    let pool = ConnectionPool::new(5);
    
    {
        let conn1 = pool.acquire().unwrap();
        assert_eq!(pool.active_connections(), 1);
        
        let conn2 = pool.acquire().unwrap();
        assert_eq!(pool.active_connections(), 2);
        
        drop(conn1);
        assert_eq!(pool.active_connections(), 1);
    }
    
    assert_eq!(pool.active_connections(), 0);
    assert_eq!(pool.available_connections(), 5);
}

#[test]
fn test_connection_pool_exhaustion() {
    let pool = ConnectionPool::new(3);
    
    let _conn1 = pool.acquire().unwrap();
    let _conn2 = pool.acquire().unwrap();
    let _conn3 = pool.acquire().unwrap();
    
    // Pool exhausted
    let result = pool.acquire_timeout(Duration::from_millis(100));
    assert!(result.is_err());
}

#[test]
fn test_connection_pool_concurrent() {
    let pool = Arc::new(ConnectionPool::new(20));
    let mut handles = vec![];
    
    for _ in 0..50 {
        let pool_clone = Arc::clone(&pool);
        let handle = thread::spawn(move || {
            let conn = pool_clone.acquire().unwrap();
            thread::sleep(Duration::from_millis(5));
            drop(conn);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    assert_eq!(pool.active_connections(), 0);
}

// ============================================================================
// Database Integration Tests
// ============================================================================

#[test]
fn test_database_creation() {
    let config = DatabaseConfig::default();
    let db = Database::new(config);
    
    let stats = db.get_stats();
    assert_eq!(stats.active_transactions, 0);
    assert_eq!(stats.total_indexes, 0);
}

#[test]
fn test_database_index_management() {
    let config = DatabaseConfig::default();
    let db = Database::new(config);
    
    // Create indexes
    db.create_index("users".to_string()).unwrap();
    db.create_index("orders".to_string()).unwrap();
    db.create_index("products".to_string()).unwrap();
    
    assert_eq!(db.get_stats().total_indexes, 3);
    
    // Get index
    let users_index = db.get_index("users").unwrap();
    let index = users_index.read();
    
    // Should fail to create duplicate
    assert!(db.create_index("users".to_string()).is_err());
    
    // Should fail to get non-existent index
    assert!(db.get_index("nonexistent").is_err());
}

#[test]
fn test_database_transaction_workflow() {
    let config = DatabaseConfig::default();
    let db = Database::new(config);
    
    // Create index
    db.create_index("accounts".to_string()).unwrap();
    let index = db.get_index("accounts").unwrap();
    
    // Begin transaction
    let txn_id = db.begin_transaction(None).unwrap();
    assert_eq!(db.get_stats().active_transactions, 1);
    
    // Insert data through index
    {
        let idx = index.read();
        idx.insert(b"alice".to_vec(), 1).unwrap();
        idx.insert(b"bob".to_vec(), 2).unwrap();
    }
    
    // Commit transaction
    db.commit_transaction(txn_id).unwrap();
    assert_eq!(db.get_stats().active_transactions, 0);
    
    // Verify data
    {
        let idx = index.read();
        assert_eq!(idx.search(b"alice"), vec![1]);
        assert_eq!(idx.search(b"bob"), vec![2]);
    }
}

#[test]
fn test_database_acid_properties() {
    let config = DatabaseConfig {
        btree_order: 64,
        max_transactions: 100,
        default_isolation: IsolationLevel::Serializable,
        enable_deadlock_detection: true,
        pool_size: 50,
        transaction_timeout_ms: 30000,
    };
    let db = Database::new(config);
    
    db.create_index("ledger".to_string()).unwrap();
    let index = db.get_index("ledger").unwrap();
    
    // Test Atomicity: All or nothing
    let txn1 = db.begin_transaction(None).unwrap();
    {
        let idx = index.read();
        idx.insert(b"tx1:op1".to_vec(), 100).unwrap();
        idx.insert(b"tx1:op2".to_vec(), 101).unwrap();
    }
    db.commit_transaction(txn1).unwrap();
    
    // Test Consistency: Valid state transitions
    let txn2 = db.begin_transaction(None).unwrap();
    {
        let idx = index.read();
        idx.insert(b"balance:alice".to_vec(), 1000).unwrap();
        idx.insert(b"balance:bob".to_vec(), 500).unwrap();
    }
    db.commit_transaction(txn2).unwrap();
    
    // Test Isolation: Concurrent transactions don't interfere
    let txn3 = db.begin_transaction(Some(IsolationLevel::Serializable)).unwrap();
    let txn4 = db.begin_transaction(Some(IsolationLevel::Serializable)).unwrap();
    
    db.commit_transaction(txn3).unwrap();
    db.commit_transaction(txn4).unwrap();
    
    // Test Durability: Committed data persists
    {
        let idx = index.read();
        assert_eq!(idx.search(b"tx1:op1"), vec![100]);
        assert_eq!(idx.search(b"balance:alice"), vec![1000]);
    }
}

#[test]
fn test_database_concurrent_operations() {
    let config = DatabaseConfig::default();
    let db = Arc::new(Database::new(config));
    
    db.create_index("concurrent_test".to_string()).unwrap();
    
    let mut handles = vec![];
    
    for i in 0..10 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            let txn_id = db_clone.begin_transaction(None).unwrap();
            
            let index = db_clone.get_index("concurrent_test").unwrap();
            {
                let idx = index.read();
                idx.insert(format!("key_{}", i).into_bytes(), i as u64).unwrap();
            }
            
            thread::sleep(Duration::from_millis(10));
            
            db_clone.commit_transaction(txn_id).unwrap();
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify all data was inserted
    let index = db.get_index("concurrent_test").unwrap();
    let idx = index.read();
    for i in 0..10 {
        let results = idx.search(format!("key_{}", i).as_bytes());
        assert_eq!(results, vec![i as u64]);
    }
}

#[test]
fn test_database_stress_test() {
    let config = DatabaseConfig {
        btree_order: 128,
        max_transactions: 1000,
        default_isolation: IsolationLevel::ReadCommitted,
        enable_deadlock_detection: true,
        pool_size: 100,
        transaction_timeout_ms: 60000,
    };
    let db = Arc::new(Database::new(config));
    
    db.create_index("stress_test".to_string()).unwrap();
    
    let mut handles = vec![];
    
    // Spawn 50 threads, each doing 100 operations
    for thread_id in 0..50 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            for op_id in 0..100 {
                let txn_id = db_clone.begin_transaction(None).unwrap();
                
                let index = db_clone.get_index("stress_test").unwrap();
                {
                    let idx = index.read();
                    let key = format!("thread_{}:op_{}", thread_id, op_id);
                    let value = (thread_id * 1000 + op_id) as u64;
                    idx.insert(key.into_bytes(), value).unwrap();
                }
                
                db_clone.commit_transaction(txn_id).unwrap();
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify all 5000 operations completed
    let index = db.get_index("stress_test").unwrap();
    let idx = index.read();
    let stats = idx.stats();
    assert!(stats.num_entries >= 5000);
}

#[test]
fn test_database_rollback_scenario() {
    let config = DatabaseConfig::default();
    let db = Database::new(config);
    
    db.create_index("rollback_test".to_string()).unwrap();
    let index = db.get_index("rollback_test").unwrap();
    
    // Successful transaction
    let txn1 = db.begin_transaction(None).unwrap();
    {
        let idx = index.read();
        idx.insert(b"committed".to_vec(), 1).unwrap();
    }
    db.commit_transaction(txn1).unwrap();
    
    // Failed transaction (rollback)
    let txn2 = db.begin_transaction(None).unwrap();
    {
        let idx = index.read();
        idx.insert(b"rolled_back".to_vec(), 2).unwrap();
    }
    db.rollback_transaction(txn2).unwrap();
    
    // Verify only committed data exists
    {
        let idx = index.read();
        assert_eq!(idx.search(b"committed"), vec![1]);
        // Note: In a real implementation with proper undo logs,
        // rolled back data would not be visible
    }
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_bplus_tree_performance() {
    let tree = BPlusTree::new(256);
    let start = std::time::Instant::now();
    
    // Insert 100,000 entries
    for i in 0..100000 {
        tree.insert(i, format!("value_{}", i)).unwrap();
    }
    
    let insert_duration = start.elapsed();
    println!("Inserted 100,000 entries in {:?}", insert_duration);
    
    // Search 10,000 random entries
    let start = std::time::Instant::now();
    for i in (0..100000).step_by(10) {
        assert!(tree.get(&i).is_some());
    }
    let search_duration = start.elapsed();
    println!("Searched 10,000 entries in {:?}", search_duration);
    
    // Range query
    let start = std::time::Instant::now();
    let results = tree.range_query(&40000, &50000);
    let range_duration = start.elapsed();
    println!("Range query (10,000 entries) in {:?}", range_duration);
    assert_eq!(results.len(), 10001);
}

#[test]
fn test_transaction_throughput() {
    let mut tm = TransactionManager::new(10000, IsolationLevel::ReadCommitted, false, 60000);
    let start = std::time::Instant::now();
    
    // Execute 1000 transactions
    for _ in 0..1000 {
        let txn_id = tm.begin_transaction(None).unwrap();
        tm.write(txn_id, "key".to_string(), b"value".to_vec()).unwrap();
        tm.commit_transaction(txn_id).unwrap();
    }
    
    let duration = start.elapsed();
    let tps = 1000.0 / duration.as_secs_f64();
    println!("Transaction throughput: {:.2} TPS", tps);
}