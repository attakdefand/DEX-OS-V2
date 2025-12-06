// Connection Pool for Database
// Security: Layer 9 - Database Security
// Manages database connections efficiently with resource pooling

use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::{Mutex, Condvar};
use std::collections::VecDeque;

/// Database connection
#[derive(Debug, Clone)]
pub struct Connection {
    id: usize,
    created_at: Instant,
    last_used: Instant,
}

impl Connection {
    fn new(id: usize) -> Self {
        let now = Instant::now();
        Self {
            id,
            created_at: now,
            last_used: now,
        }
    }

    fn update_last_used(&mut self) {
        self.last_used = Instant::now();
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    pub fn idle_time(&self) -> Duration {
        self.last_used.elapsed()
    }
}

/// Connection pool
pub struct ConnectionPool {
    pool_size: usize,
    available: Arc<Mutex<VecDeque<Connection>>>,
    in_use: Arc<Mutex<Vec<Connection>>>,
    condvar: Arc<Condvar>,
    next_id: Arc<Mutex<usize>>,
    max_idle_time: Duration,
    max_connection_age: Duration,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(pool_size: usize) -> Self {
        let mut available = VecDeque::new();
        
        // Pre-create connections
        for i in 0..pool_size {
            available.push_back(Connection::new(i));
        }

        Self {
            pool_size,
            available: Arc::new(Mutex::new(available)),
            in_use: Arc::new(Mutex::new(Vec::new())),
            condvar: Arc::new(Condvar::new()),
            next_id: Arc::new(Mutex::new(pool_size)),
            max_idle_time: Duration::from_secs(300), // 5 minutes
            max_connection_age: Duration::from_secs(3600), // 1 hour
        }
    }

    /// Acquire a connection from the pool
    pub fn acquire(&self) -> Result<PooledConnection, String> {
        self.acquire_timeout(Duration::from_secs(30))
    }

    /// Acquire a connection with timeout
    pub fn acquire_timeout(&self, timeout: Duration) -> Result<PooledConnection, String> {
        let start = Instant::now();
        
        loop {
            let mut available = self.available.lock();
            
            // Try to get an available connection
            if let Some(mut conn) = available.pop_front() {
                // Check if connection is still valid
                if self.is_connection_valid(&conn) {
                    conn.update_last_used();
                    
                    let mut in_use = self.in_use.lock();
                    in_use.push(conn.clone());
                    drop(in_use);
                    drop(available);
                    
                    return Ok(PooledConnection {
                        connection: Some(conn),
                        pool: self,
                    });
                } else {
                    // Create a new connection
                    let mut next_id = self.next_id.lock();
                    let new_conn = Connection::new(*next_id);
                    *next_id += 1;
                    drop(next_id);
                    
                    let mut in_use = self.in_use.lock();
                    in_use.push(new_conn.clone());
                    drop(in_use);
                    drop(available);
                    
                    return Ok(PooledConnection {
                        connection: Some(new_conn),
                        pool: self,
                    });
                }
            }
            
            // Check timeout
            if start.elapsed() >= timeout {
                return Err("Connection pool timeout".to_string());
            }
            
            // Wait for a connection to be released
            self.condvar.wait_for(&mut available, Duration::from_millis(100));
        }
    }

    /// Release a connection back to the pool
    fn release(&self, mut connection: Connection) {
        connection.update_last_used();
        
        let mut in_use = self.in_use.lock();
        in_use.retain(|c| c.id != connection.id);
        drop(in_use);
        
        let mut available = self.available.lock();
        available.push_back(connection);
        drop(available);
        
        self.condvar.notify_one();
    }

    /// Check if connection is valid
    fn is_connection_valid(&self, conn: &Connection) -> bool {
        conn.age() < self.max_connection_age && conn.idle_time() < self.max_idle_time
    }

    /// Get number of active connections
    pub fn active_connections(&self) -> usize {
        self.in_use.lock().len()
    }

    /// Get number of available connections
    pub fn available_connections(&self) -> usize {
        self.available.lock().len()
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            pool_size: self.pool_size,
            active: self.active_connections(),
            available: self.available_connections(),
        }
    }

    /// Cleanup idle connections
    pub fn cleanup_idle(&self) {
        let mut available = self.available.lock();
        available.retain(|conn| self.is_connection_valid(conn));
    }
}

/// Pooled connection with automatic return to pool
pub struct PooledConnection<'a> {
    connection: Option<Connection>,
    pool: &'a ConnectionPool,
}

impl<'a> PooledConnection<'a> {
    /// Get the underlying connection
    pub fn connection(&self) -> &Connection {
        self.connection.as_ref().unwrap()
    }

    /// Execute a query (placeholder)
    pub fn execute(&self, _query: &str) -> Result<(), String> {
        // In real implementation, execute query on connection
        Ok(())
    }
}

impl<'a> Drop for PooledConnection<'a> {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            self.pool.release(conn);
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub pool_size: usize,
    pub active: usize,
    pub available: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_pool_creation() {
        let pool = ConnectionPool::new(10);
        let stats = pool.stats();
        
        assert_eq!(stats.pool_size, 10);
        assert_eq!(stats.active, 0);
        assert_eq!(stats.available, 10);
    }

    #[test]
    fn test_acquire_release() {
        let pool = ConnectionPool::new(5);
        
        {
            let conn1 = pool.acquire().unwrap();
            assert_eq!(pool.active_connections(), 1);
            assert_eq!(pool.available_connections(), 4);
            
            let conn2 = pool.acquire().unwrap();
            assert_eq!(pool.active_connections(), 2);
            assert_eq!(pool.available_connections(), 3);
            
            drop(conn1);
            assert_eq!(pool.active_connections(), 1);
            assert_eq!(pool.available_connections(), 4);
        }
        
        assert_eq!(pool.active_connections(), 0);
        assert_eq!(pool.available_connections(), 5);
    }

    #[test]
    fn test_pool_exhaustion() {
        let pool = ConnectionPool::new(2);
        
        let _conn1 = pool.acquire().unwrap();
        let _conn2 = pool.acquire().unwrap();
        
        // Pool is exhausted, should timeout
        let result = pool.acquire_timeout(Duration::from_millis(100));
        assert!(result.is_err());
    }

    #[test]
    fn test_connection_reuse() {
        let pool = ConnectionPool::new(3);
        
        let conn_id = {
            let conn = pool.acquire().unwrap();
            conn.connection().id()
        };
        
        // Connection should be returned to pool
        let conn = pool.acquire().unwrap();
        assert_eq!(conn.connection().id(), conn_id);
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;
        use std::sync::Arc;
        
        let pool = Arc::new(ConnectionPool::new(10));
        let mut handles = vec![];
        
        for _ in 0..20 {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                let conn = pool_clone.acquire().unwrap();
                thread::sleep(Duration::from_millis(10));
                drop(conn);
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        assert_eq!(pool.active_connections(), 0);
    }
}
