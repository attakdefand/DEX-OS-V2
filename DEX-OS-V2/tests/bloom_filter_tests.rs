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

    println!("False positives: {} out of 50", false_positives);

    // With a well-sized filter, false positives should be relatively rare
    assert!(false_positives <= 30); // Less than or equal to 60% false positive rate
}

#[test]
fn test_bloom_filter_default() {
    let filter = BloomFilter::default();

    // Should be able to create and use default filter
    assert!(!filter.might_contain("any_item"));

    // We can test the functionality without accessing private fields
    // The default filter should work correctly for basic operations
    filter.might_contain("test_item"); // This should not panic
    
    // We can verify it has reasonable behavior by testing that it can be used
    assert_eq!(filter.might_contain("nonexistent"), false); // Default filter should not contain anything
}