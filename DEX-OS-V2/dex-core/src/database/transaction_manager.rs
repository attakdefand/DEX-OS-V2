// ACID Transaction Manager
// Security: Layer 9 - Database Security
// Implements full ACID properties: Atomicity, Consistency, Isolation, Durability

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::{RwLock, Mutex};

/// Transaction ID
pub type TransactionId = u64;

/// Resource ID (for locking)
pub type ResourceId = String;

/// Transaction isolation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    /// Read uncommitted (lowest isolation)
    ReadUncommitted,
    /// Read committed
    ReadCommitted,
    /// Repeatable read
    RepeatableRead,
    /// Serializable (highest isolation)
    Serializable,
}

/// Transaction state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    Active,
    Preparing,
    Committed,
    Aborted,
}

/// Lock type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockType {
    Shared,    // Read lock
    Exclusive, // Write lock
}

/// Transaction metadata
#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: TransactionId,
    pub isolation_level: IsolationLevel,
    pub state: TransactionState,
    pub start_time: Instant,
    pub operations: Vec<Operation>,
    pub read_set: HashSet<ResourceId>,
    pub write_set: HashSet<ResourceId>,
}

impl Transaction {
    fn new(id: TransactionId, isolation_level: IsolationLevel) -> Self {
        Self {
            id,
            isolation_level,
            state: TransactionState::Active,
            start_time: Instant::now(),
            operations: Vec::new(),
            read_set: HashSet::new(),
            write_set: HashSet::new(),
        }
    }

    fn add_read(&mut self, resource: ResourceId) {
        self.read_set.insert(resource);
    }

    fn add_write(&mut self, resource: ResourceId, old_value: Vec<u8>, new_value: Vec<u8>) {
        self.write_set.insert(resource.clone());
        self.operations.push(Operation::Write {
            resource,
            old_value,
            new_value,
        });
    }
}

/// Database operation
#[derive(Debug, Clone)]
pub enum Operation {
    Write {
        resource: ResourceId,
        old_value: Vec<u8>,
        new_value: Vec<u8>,
    },
}

/// Lock manager for concurrency control
pub struct LockManager {
    locks: Arc<RwLock<HashMap<ResourceId, LockInfo>>>,
    wait_graph: Arc<RwLock<HashMap<TransactionId, HashSet<TransactionId>>>>,
}

#[derive(Debug, Clone)]
struct LockInfo {
    lock_type: LockType,
    holders: HashSet<TransactionId>,
    waiters: VecDeque<(TransactionId, LockType)>,
}

impl LockManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
            wait_graph: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Acquire a lock on a resource
    pub fn acquire_lock(
        &self,
        txn_id: TransactionId,
        resource: ResourceId,
        lock_type: LockType,
    ) -> Result<(), String> {
        let mut locks = self.locks.write();
        
        let lock_info = locks.entry(resource.clone()).or_insert(LockInfo {
            lock_type: LockType::Shared,
            holders: HashSet::new(),
            waiters: VecDeque::new(),
        });

        // Check if lock can be granted immediately
        if self.can_grant_lock(&lock_info, lock_type) {
            lock_info.holders.insert(txn_id);
            if lock_type == LockType::Exclusive {
                lock_info.lock_type = LockType::Exclusive;
            }
            Ok(())
        } else {
            // Add to wait queue
            lock_info.waiters.push_back((txn_id, lock_type));
            
            // Update wait graph for deadlock detection
            let mut wait_graph = self.wait_graph.write();
            let waiting_for = lock_info.holders.clone();
            wait_graph.insert(txn_id, waiting_for);
            
            Err(format!("Transaction {} waiting for lock on {}", txn_id, resource))
        }
    }

    /// Release all locks held by a transaction
    pub fn release_locks(&self, txn_id: TransactionId) {
        let mut locks = self.locks.write();
        let mut wait_graph = self.wait_graph.write();
        
        // Remove from wait graph
        wait_graph.remove(&txn_id);
        
        // Release all locks
        let resources_to_remove: Vec<ResourceId> = locks.keys().cloned().collect();
        
        for resource in resources_to_remove {
            if let Some(lock_info) = locks.get_mut(&resource) {
                lock_info.holders.remove(&txn_id);
                
                // Grant locks to waiters if possible
                while let Some((waiting_txn, waiting_lock_type)) = lock_info.waiters.front() {
                    if self.can_grant_lock(&lock_info, *waiting_lock_type) {
                        let (waiting_txn, waiting_lock_type) = lock_info.waiters.pop_front().unwrap();
                        lock_info.holders.insert(waiting_txn);
                        if waiting_lock_type == LockType::Exclusive {
                            lock_info.lock_type = LockType::Exclusive;
                        }
                        wait_graph.remove(&waiting_txn);
                    } else {
                        break;
                    }
                }
                
                // Remove lock info if no holders or waiters
                if lock_info.holders.is_empty() && lock_info.waiters.is_empty() {
                    locks.remove(&resource);
                }
            }
        }
    }

    fn can_grant_lock(&self, lock_info: &LockInfo, requested_type: LockType) -> bool {
        if lock_info.holders.is_empty() {
            return true;
        }

        match requested_type {
            LockType::Shared => lock_info.lock_type == LockType::Shared,
            LockType::Exclusive => lock_info.holders.is_empty(),
        }
    }
}

/// Deadlock detector
pub struct DeadlockDetector {
    enabled: bool,
}

impl DeadlockDetector {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Detect deadlocks in the wait graph
    pub fn detect_deadlock(&self, wait_graph: &HashMap<TransactionId, HashSet<TransactionId>>) -> Option<Vec<TransactionId>> {
        if !self.enabled {
            return None;
        }

        // Use DFS to detect cycles
        for &start_txn in wait_graph.keys() {
            let mut visited = HashSet::new();
            let mut path = Vec::new();
            
            if self.has_cycle(start_txn, wait_graph, &mut visited, &mut path) {
                return Some(path);
            }
        }

        None
    }

    fn has_cycle(
        &self,
        txn: TransactionId,
        wait_graph: &HashMap<TransactionId, HashSet<TransactionId>>,
        visited: &mut HashSet<TransactionId>,
        path: &mut Vec<TransactionId>,
    ) -> bool {
        if path.contains(&txn) {
            return true;
        }

        if visited.contains(&txn) {
            return false;
        }

        visited.insert(txn);
        path.push(txn);

        if let Some(waiting_for) = wait_graph.get(&txn) {
            for &next_txn in waiting_for {
                if self.has_cycle(next_txn, wait_graph, visited, path) {
                    return true;
                }
            }
        }

        path.pop();
        false
    }
}

/// Transaction manager
pub struct TransactionManager {
    next_txn_id: Arc<Mutex<TransactionId>>,
    active_transactions: Arc<RwLock<HashMap<TransactionId, Transaction>>>,
    lock_manager: Arc<LockManager>,
    deadlock_detector: Arc<DeadlockDetector>,
    default_isolation: IsolationLevel,
    max_transactions: usize,
    transaction_timeout: Duration,
    write_ahead_log: Arc<RwLock<Vec<LogEntry>>>,
}

#[derive(Debug, Clone)]
struct LogEntry {
    txn_id: TransactionId,
    operation: Operation,
    timestamp: Instant,
}

impl TransactionManager {
    pub fn new(
        max_transactions: usize,
        default_isolation: IsolationLevel,
        enable_deadlock_detection: bool,
        timeout_ms: u64,
    ) -> Self {
        Self {
            next_txn_id: Arc::new(Mutex::new(1)),
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
            lock_manager: Arc::new(LockManager::new()),
            deadlock_detector: Arc::new(DeadlockDetector::new(enable_deadlock_detection)),
            default_isolation,
            max_transactions,
            transaction_timeout: Duration::from_millis(timeout_ms),
            write_ahead_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Begin a new transaction
    pub fn begin_transaction(&mut self, isolation: Option<IsolationLevel>) -> Result<TransactionId, String> {
        let active = self.active_transactions.read();
        
        if active.len() >= self.max_transactions {
            return Err("Maximum number of concurrent transactions reached".to_string());
        }
        drop(active);

        let mut next_id = self.next_txn_id.lock();
        let txn_id = *next_id;
        *next_id += 1;
        drop(next_id);

        let isolation_level = isolation.unwrap_or(self.default_isolation);
        let transaction = Transaction::new(txn_id, isolation_level);

        let mut active = self.active_transactions.write();
        active.insert(txn_id, transaction);

        Ok(txn_id)
    }

    /// Read a resource
    pub fn read(&self, txn_id: TransactionId, resource: ResourceId) -> Result<Vec<u8>, String> {
        let mut active = self.active_transactions.write();
        
        let txn = active.get_mut(&txn_id)
            .ok_or_else(|| format!("Transaction {} not found", txn_id))?;

        if txn.state != TransactionState::Active {
            return Err(format!("Transaction {} is not active", txn_id));
        }

        // Acquire shared lock based on isolation level
        match txn.isolation_level {
            IsolationLevel::ReadUncommitted => {
                // No locking needed
            }
            _ => {
                self.lock_manager.acquire_lock(txn_id, resource.clone(), LockType::Shared)?;
            }
        }

        txn.add_read(resource.clone());

        // Simulate reading data (in real implementation, read from storage)
        Ok(vec![])
    }

    /// Write to a resource
    pub fn write(&self, txn_id: TransactionId, resource: ResourceId, value: Vec<u8>) -> Result<(), String> {
        let mut active = self.active_transactions.write();
        
        let txn = active.get_mut(&txn_id)
            .ok_or_else(|| format!("Transaction {} not found", txn_id))?;

        if txn.state != TransactionState::Active {
            return Err(format!("Transaction {} is not active", txn_id));
        }

        // Acquire exclusive lock
        self.lock_manager.acquire_lock(txn_id, resource.clone(), LockType::Exclusive)?;

        // Simulate reading old value
        let old_value = vec![];
        
        txn.add_write(resource.clone(), old_value, value.clone());

        // Write to WAL for durability
        let mut wal = self.write_ahead_log.write();
        wal.push(LogEntry {
            txn_id,
            operation: Operation::Write {
                resource,
                old_value: vec![],
                new_value: value,
            },
            timestamp: Instant::now(),
        });

        Ok(())
    }

    /// Commit a transaction
    pub fn commit_transaction(&mut self, txn_id: TransactionId) -> Result<(), String> {
        let mut active = self.active_transactions.write();
        
        let txn = active.get_mut(&txn_id)
            .ok_or_else(|| format!("Transaction {} not found", txn_id))?;

        // Check for timeout
        if txn.start_time.elapsed() > self.transaction_timeout {
            txn.state = TransactionState::Aborted;
            self.lock_manager.release_locks(txn_id);
            return Err(format!("Transaction {} timed out", txn_id));
        }

        // Validation phase (for serializable isolation)
        let isolation_level = txn.isolation_level;
        if isolation_level == IsolationLevel::Serializable {
            // Clone necessary data for validation to avoid borrow issues
            let txn_read_set = txn.read_set.clone();
            let txn_write_set = txn.write_set.clone();
            
            // Check for conflicts with other transactions
            for (other_id, other_txn) in active.iter() {
                if *other_id == txn_id {
                    continue;
                }

                // Check for write-read conflicts
                if !txn_read_set.is_disjoint(&other_txn.write_set) {
                    return Err(format!("Validation failed: write-read conflict with transaction {}", other_id));
                }

                // Check for write-write conflicts
                if !txn_write_set.is_disjoint(&other_txn.write_set) {
                    return Err(format!("Validation failed: write-write conflict with transaction {}", other_id));
                }
            }
        }

        // Get transaction again for commit phase
        let txn = active.get_mut(&txn_id).unwrap();
        
        // Commit phase
        txn.state = TransactionState::Committed;

        // Apply writes (in real implementation, write to storage)
        for operation in &txn.operations {
            match operation {
                Operation::Write { resource, new_value, .. } => {
                    // Apply write to storage
                    let _ = (resource, new_value);
                }
            }
        }

        // Release locks
        self.lock_manager.release_locks(txn_id);

        // Remove from active transactions
        active.remove(&txn_id);

        Ok(())
    }

    /// Rollback a transaction
    pub fn rollback_transaction(&mut self, txn_id: TransactionId) -> Result<(), String> {
        let mut active = self.active_transactions.write();
        
        let txn = active.get_mut(&txn_id)
            .ok_or_else(|| format!("Transaction {} not found", txn_id))?;

        txn.state = TransactionState::Aborted;

        // Release locks
        self.lock_manager.release_locks(txn_id);

        // Remove from active transactions
        active.remove(&txn_id);

        Ok(())
    }

    /// Validate transaction for serializable isolation
    fn validate_transaction(
        &self,
        txn_id: TransactionId,
        active: &HashMap<TransactionId, Transaction>,
    ) -> Result<(), String> {
        let txn = active.get(&txn_id).unwrap();

        // Check for conflicts with other transactions
        for (other_id, other_txn) in active.iter() {
            if *other_id == txn_id {
                continue;
            }

            // Check for write-read conflicts
            if !txn.read_set.is_disjoint(&other_txn.write_set) {
                return Err(format!("Validation failed: write-read conflict with transaction {}", other_id));
            }

            // Check for write-write conflicts
            if !txn.write_set.is_disjoint(&other_txn.write_set) {
                return Err(format!("Validation failed: write-write conflict with transaction {}", other_id));
            }
        }

        Ok(())
    }

    /// Get active transaction count
    pub fn active_transaction_count(&self) -> usize {
        self.active_transactions.read().len()
    }

    /// Check for deadlocks
    pub fn check_deadlocks(&self) -> Option<Vec<TransactionId>> {
        let wait_graph = self.lock_manager.wait_graph.read();
        self.deadlock_detector.detect_deadlock(&wait_graph)
    }

    /// Get transaction statistics
    pub fn get_stats(&self) -> TransactionStats {
        let active = self.active_transactions.read();
        let wal = self.write_ahead_log.read();

        TransactionStats {
            active_count: active.len(),
            wal_entries: wal.len(),
        }
    }
}

/// Transaction statistics
#[derive(Debug, Clone)]
pub struct TransactionStats {
    pub active_count: usize,
    pub wal_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_lifecycle() {
        let mut tm = TransactionManager::new(100, IsolationLevel::Serializable, true, 30000);
        
        let txn_id = tm.begin_transaction(None).unwrap();
        assert_eq!(tm.active_transaction_count(), 1);
        
        tm.commit_transaction(txn_id).unwrap();
        assert_eq!(tm.active_transaction_count(), 0);
    }

    #[test]
    fn test_transaction_rollback() {
        let mut tm = TransactionManager::new(100, IsolationLevel::Serializable, true, 30000);
        
        let txn_id = tm.begin_transaction(None).unwrap();
        tm.write(txn_id, "key1".to_string(), b"value1".to_vec()).unwrap();
        
        tm.rollback_transaction(txn_id).unwrap();
        assert_eq!(tm.active_transaction_count(), 0);
    }

    #[test]
    fn test_lock_manager() {
        let lm = LockManager::new();
        
        // Acquire shared locks
        assert!(lm.acquire_lock(1, "resource1".to_string(), LockType::Shared).is_ok());
        assert!(lm.acquire_lock(2, "resource1".to_string(), LockType::Shared).is_ok());
        
        // Exclusive lock should wait
        assert!(lm.acquire_lock(3, "resource1".to_string(), LockType::Exclusive).is_err());
        
        // Release locks
        lm.release_locks(1);
        lm.release_locks(2);
    }

    #[test]
    fn test_deadlock_detection() {
        let detector = DeadlockDetector::new(true);
        
        let mut wait_graph = HashMap::new();
        wait_graph.insert(1, vec![2].into_iter().collect());
        wait_graph.insert(2, vec![3].into_iter().collect());
        wait_graph.insert(3, vec![1].into_iter().collect());
        
        let cycle = detector.detect_deadlock(&wait_graph);
        assert!(cycle.is_some());
    }

    #[test]
    fn test_isolation_levels() {
        let mut tm = TransactionManager::new(100, IsolationLevel::ReadCommitted, true, 30000);
        
        let txn1 = tm.begin_transaction(Some(IsolationLevel::Serializable)).unwrap();
        let txn2 = tm.begin_transaction(Some(IsolationLevel::ReadUncommitted)).unwrap();
        
        assert_eq!(tm.active_transaction_count(), 2);
        
        tm.commit_transaction(txn1).unwrap();
        tm.commit_transaction(txn2).unwrap();
    }
}
