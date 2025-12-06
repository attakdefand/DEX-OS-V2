//! Ring Buffer Implementation for Security Layer 10
//!
//! A fixed-size circular buffer for high-performance data storage,
//! useful for rate limiting, audit logging, and buffering.

use std::sync::{Arc, RwLock};

/// A generic fixed-size ring buffer
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    buffer: Vec<Option<T>>,
    capacity: usize,
    head: usize,
    tail: usize,
    size: usize,
}

impl<T: Clone> RingBuffer<T> {
    /// Create a new ring buffer with specified capacity
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(None);
        }

        Self {
            buffer,
            capacity,
            head: 0,
            tail: 0,
            size: 0,
        }
    }

    /// Push an item to the buffer (overwriting oldest if full)
    pub fn push(&mut self, item: T) {
        self.buffer[self.head] = Some(item);
        self.head = (self.head + 1) % self.capacity;

        if self.size < self.capacity {
            self.size += 1;
        } else {
            self.tail = (self.tail + 1) % self.capacity;
        }
    }

    /// Pop an item from the buffer (oldest first)
    pub fn pop(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }

        let item = self.buffer[self.tail].take();
        self.tail = (self.tail + 1) % self.capacity;
        self.size -= 1;

        item
    }

    /// Peek at the oldest item
    pub fn peek(&self) -> Option<&T> {
        if self.size == 0 {
            return None;
        }
        self.buffer[self.tail].as_ref()
    }

    /// Get all items in order (oldest to newest)
    pub fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.size);
        let mut current = self.tail;
        
        for _ in 0..self.size {
            if let Some(item) = &self.buffer[current] {
                result.push(item.clone());
            }
            current = (current + 1) % self.capacity;
        }
        
        result
    }

    /// Get current size
    pub fn len(&self) -> usize {
        self.size
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Check if full
    pub fn is_full(&self) -> bool {
        self.size == self.capacity
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        for i in 0..self.capacity {
            self.buffer[i] = None;
        }
        self.head = 0;
        self.tail = 0;
        self.size = 0;
    }
}

/// Thread-safe Ring Buffer Manager
#[derive(Debug, Clone)]
pub struct RingBufferManager<T> {
    buffer: Arc<RwLock<RingBuffer<T>>>,
}

impl<T: Clone + Send + Sync + 'static> RingBufferManager<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(RingBuffer::new(capacity))),
        }
    }

    pub fn push(&self, item: T) {
        let mut buf = self.buffer.write().unwrap();
        buf.push(item);
    }

    pub fn pop(&self) -> Option<T> {
        let mut buf = self.buffer.write().unwrap();
        buf.pop()
    }

    pub fn get_all(&self) -> Vec<T> {
        let buf = self.buffer.read().unwrap();
        buf.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let mut rb = RingBuffer::new(3);
        
        rb.push(1);
        rb.push(2);
        rb.push(3);
        
        assert_eq!(rb.len(), 3);
        assert!(rb.is_full());
        
        // Should overwrite 1
        rb.push(4);
        assert_eq!(rb.len(), 3);
        
        assert_eq!(rb.pop(), Some(2));
        assert_eq!(rb.pop(), Some(3));
        assert_eq!(rb.pop(), Some(4));
        assert_eq!(rb.pop(), None);
    }

    #[test]
    fn test_ring_buffer_order() {
        let mut rb = RingBuffer::new(5);
        for i in 0..5 {
            rb.push(i);
        }
        
        let vec = rb.to_vec();
        assert_eq!(vec, vec![0, 1, 2, 3, 4]);
    }
}
