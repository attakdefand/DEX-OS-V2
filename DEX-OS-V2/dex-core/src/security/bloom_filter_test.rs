//! Simple test for the BloomFilter implementation

use super::bloom_filter::BloomFilter;

#[test]
fn test_simple_bloom_filter() {
    let mut filter = BloomFilter::new(100, 3);
    filter.add("test");
    assert!(filter.might_contain("test"));
    assert!(!filter.might_contain("nonexistent"));
}

#[test]
fn test_bloom_filter_with_multiple_items() {
    let mut filter = BloomFilter::new(1000, 3);
    
    // Add multiple items
    filter.add("item1");
    filter.add("item2");
    filter.add("item3");
    
    // Check they exist
    assert!(filter.might_contain("item1"));
    assert!(filter.might_contain("item2"));
    assert!(filter.might_contain("item3"));
    
    // Check non-existent items
    assert!(!filter.might_contain("item4"));
    assert!(!filter.might_contain("item5"));
}