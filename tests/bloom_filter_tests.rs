//! Tests for the Bloom Filter implementation

use dex_core::security::BloomFilter;

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