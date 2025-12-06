//! Simple test for the Bloom Filter implementation

use dex_core::security::BloomFilter;

#[test]
fn test_simple_bloom() {
    let mut filter = BloomFilter::new(100, 3);
    filter.add("test");
    assert!(filter.might_contain("test"));
    assert!(!filter.might_contain("nonexistent"));
}