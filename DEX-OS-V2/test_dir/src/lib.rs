//! Bloom Filter implementation for Access Control

use sha3::{Digest, Sha3_256};

/// Bloom filter for efficient probabilistic set membership testing
#[derive(Debug, Clone)]
pub struct BloomFilter {
    /// Bit array for the filter
    bits: Vec<bool>,
    /// Number of hash functions
    num_hash_functions: usize,
    /// Size of the bit array
    size: usize,
}

impl BloomFilter {
    /// Create a new Bloom filter with the specified size and number of hash functions
    pub fn new(size: usize, num_hash_functions: usize) -> Self {
        Self {
            bits: vec![false; size],
            num_hash_functions,
            size,
        }
    }

    /// Add an item to the Bloom filter
    pub fn add(&mut self, item: &str) {
        for i in 0..self.num_hash_functions {
            let hash = self.hash(item, i);
            let index = hash % self.size;
            self.bits[index] = true;
        }
    }

    /// Check if an item might be in the set (with possible false positives)
    pub fn might_contain(&self, item: &str) -> bool {
        for i in 0..self.num_hash_functions {
            let hash = self.hash(item, i);
            let index = hash % self.size;
            if !self.bits[index] {
                return false;
            }
        }
        true
    }

    /// Simple hash function using SHA3-256
    fn hash(&self, item: &str, seed: usize) -> usize {
        let mut hasher = Sha3_256::new();
        hasher.update(item.as_bytes());
        hasher.update(&[seed as u8]);
        let result = hasher.finalize();
        
        // Convert first 8 bytes to usize
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&result[..8]);
        usize::from_le_bytes(bytes)
    }
}

impl Default for BloomFilter {
    fn default() -> Self {
        // Default to a reasonably sized filter with 3 hash functions
        Self::new(1000, 3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_filter_basic_functionality() {
        let mut filter = BloomFilter::new(100, 3);
        
        // Test adding and checking items
        filter.add("test_user_1");
        filter.add("test_user_2");
        
        assert!(filter.might_contain("test_user_1"));
        assert!(filter.might_contain("test_user_2"));
        assert!(!filter.might_contain("test_user_3")); // Should definitely not contain this
        
        // Test with larger dataset
        for i in 0..50 {
            filter.add(&format!("user_{}", i));
        }
        
        // All added items should be found
        for i in 0..50 {
            assert!(filter.might_contain(&format!("user_{}", i)));
        }
        
        // Some non-added items might have false positives, but most should be negative
        let false_positives = (50..100)
            .filter(|i| filter.might_contain(&format!("user_{}", i)))
            .count();
        
        // With a well-sized filter, false positives should be relatively rare
        assert!(false_positives < 10); // Less than 20% false positive rate
    }

    #[test]
    fn test_bloom_filter_default() {
        let filter = BloomFilter::default();
        
        // Should be able to create and use default filter
        assert!(!filter.might_contain("any_item"));
        
        // Size should be reasonable
        assert!(filter.size > 0);
        assert!(filter.num_hash_functions > 0);
    }
}