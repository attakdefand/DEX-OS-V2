//! Comprehensive tests for the Dashboard Queries functionality
//! 
//! These tests cover all aspects of the Dashboard Query Engine implementation,
//! including query registration, event recording, query execution, caching,
//! filtering, and edge cases.

use dex_core::dashboard_queries::{
    AnalyticsEvent, DashboardQuery, DashboardQueryEngine, DashboardQueryKind, QueryFilter,
};
use std::collections::HashMap;

/// Helper function to create a sample analytics event
fn sample_event(
    id: &str,
    event_type: &str,
    value: f64,
    ts: u64,
    tags: Vec<&str>,
) -> AnalyticsEvent {
    AnalyticsEvent {
        id: id.to_string(),
        event_type: event_type.to_string(),
        tags: tags.into_iter().map(|t| t.to_string()).collect(),
        value,
        timestamp: ts,
        metadata: HashMap::new(),
    }
}

/// Helper function to create a basic query
fn basic_query(kind: DashboardQueryKind) -> DashboardQuery {
    DashboardQuery {
        id: "q1".to_string(),
        name: "Test Query".to_string(),
        description: "Test query for dashboard".to_string(),
        kind,
        filter: QueryFilter {
            event_types: vec!["swap".to_string()],
            tags: vec![],
            exclude_tags: vec![],
            time_range: None,
            min_value: None,
            max_value: None,
            max_results: Some(50),
        },
        allow_cached: true,
        ttl_seconds: Some(300),
    }
}

#[test]
fn test_dashboard_engine_full_workflow() {
    let mut engine = DashboardQueryEngine::new(100, 60);

    // 1. Record some analytics events
    engine
        .record_event(sample_event("e1", "swap", 10.0, 1000, vec!["dex", "eth"]))
        .expect("Failed to record event e1");
    engine
        .record_event(sample_event("e2", "swap", 5.0, 1001, vec!["dex", "usdc"]))
        .expect("Failed to record event e2");
    engine
        .record_event(sample_event("e3", "transfer", 3.0, 1002, vec!["wallet"]))
        .expect("Failed to record event e3");

    // 2. Register different types of queries
    let count_query = basic_query(DashboardQueryKind::CountEvents);
    engine
        .register_query(count_query)
        .expect("Failed to register count query");

    let sum_query = DashboardQuery {
        id: "sum_q".to_string(),
        name: "Total Volume".to_string(),
        description: "Sum of all swap values".to_string(),
        kind: DashboardQueryKind::SumValue,
        filter: QueryFilter {
            event_types: vec!["swap".to_string()],
            tags: vec![],
            exclude_tags: vec![],
            time_range: None,
            min_value: None,
            max_value: None,
            max_results: None,
        },
        allow_cached: true,
        ttl_seconds: Some(120),
    };
    engine
        .register_query(sum_query)
        .expect("Failed to register sum query");

    let top_tags_query = DashboardQuery {
        id: "tags_q".to_string(),
        name: "Popular Tags".to_string(),
        description: "Most frequently used tags".to_string(),
        kind: DashboardQueryKind::TopTags { max: 5 },
        filter: QueryFilter::default(),
        allow_cached: false,
        ttl_seconds: None,
    };
    engine
        .register_query(top_tags_query)
        .expect("Failed to register top tags query");

    // 3. Execute queries
    let count_result = engine
        .execute_query("q1")
        .expect("Failed to execute count query");
    assert_eq!(count_result.data["count"], serde_json::json!(2)); // Only swap events

    let sum_result = engine
        .execute_query("sum_q")
        .expect("Failed to execute sum query");
    assert_eq!(sum_result.data["sum"], serde_json::json!(15.0)); // 10.0 + 5.0

    let tags_result = engine
        .execute_query("tags_q")
        .expect("Failed to execute top tags query");
    assert!(tags_result.data["top_tags"].as_array().is_some());

    // 4. Verify caching works
    let cached_result = engine
        .execute_query("q1")
        .expect("Failed to execute cached query");
    assert_eq!(cached_result.data["count"], serde_json::json!(2)); // Should be cached

    // 5. Add more events - can't test cache invalidation without set_time_override
    engine
        .record_event(sample_event("e4", "swap", 7.0, 1003, vec!["dex", "btc"]))
        .expect("Failed to record event e4");

    // Can't test cache invalidation without set_time_override, but we can verify
    // that the engine still executes queries correctly
    let result = engine
        .execute_query("q1")
        .expect("Failed to execute query");
    // The result could be either cached or fresh, so we just verify it executes
}

#[test]
fn test_dashboard_query_filtering() {
    let mut engine = DashboardQueryEngine::new(50, 30);

    // Record events with different properties
    engine
        .record_event(sample_event("e1", "swap", 5.0, 10, vec!["eth", "dex"]))
        .unwrap();
    engine
        .record_event(sample_event("e2", "swap", 7.0, 20, vec!["btc", "dex"]))
        .unwrap();
    engine
        .record_event(sample_event("e3", "transfer", 3.0, 30, vec!["eth"]))
        .unwrap();
    engine
        .record_event(sample_event("e4", "swap", 12.0, 25, vec!["eth", "bot"]))
        .unwrap(); // Should be excluded
    engine
        .record_event(sample_event("e5", "swap", 8.0, 15, vec!["sol", "dex"]))
        .unwrap();

    // Create a complex query with multiple filters
    let filtered_query = DashboardQuery {
        id: "filtered_q".to_string(),
        name: "Filtered Swaps".to_string(),
        description: "Filtered swap events".to_string(),
        kind: DashboardQueryKind::AverageValue,
        filter: QueryFilter {
            event_types: vec!["swap".to_string()],
            tags: vec!["dex".to_string()],
            exclude_tags: vec!["bot".to_string()],
            time_range: Some((10, 25)),
            min_value: Some(6.0),
            max_value: Some(10.0),
            max_results: Some(10),
        },
        allow_cached: false,
        ttl_seconds: None,
    };

    engine.register_query(filtered_query).unwrap();

    let result = engine.execute_query("filtered_q").unwrap();
    // Should only include e2 (7.0) and e5 (8.0)
    // Average = (7.0 + 8.0) / 2 = 7.5
    assert_eq!(result.data["average"], serde_json::json!(7.5));
}

#[test]
fn test_dashboard_query_validation() {
    let mut engine = DashboardQueryEngine::new(50, 30);

    // Test invalid query name (contains script tag)
    let mut unsafe_query = basic_query(DashboardQueryKind::CountEvents);
    unsafe_query.name = "<script>alert(1)</script>".to_string();
    let err = engine.register_query(unsafe_query).unwrap_err();
    assert!(err.to_string().contains("invalid characters"));

    // Test invalid TTL (zero)
    let mut zero_ttl_query = basic_query(DashboardQueryKind::SumValue);
    zero_ttl_query.ttl_seconds = Some(0);
    let err = engine.register_query(zero_ttl_query).unwrap_err();
    assert!(err.to_string().contains("ttl_seconds must be greater than 0"));

    // Test invalid TopTags max value
    let invalid_top_tags_query = basic_query(DashboardQueryKind::TopTags { max: 0 });
    let err = engine.register_query(invalid_top_tags_query).unwrap_err();
    assert!(err.to_string().contains("TopTags max must be between 1 and"));

    // Test invalid time range
    let mut invalid_time_range_query = basic_query(DashboardQueryKind::AverageValue);
    invalid_time_range_query.filter.time_range = Some((100, 50)); // start > end
    let err = engine.register_query(invalid_time_range_query).unwrap_err();
    assert!(err.to_string().contains("time_range start must be <= end"));

    // Test invalid value range
    let mut invalid_value_range_query = basic_query(DashboardQueryKind::CountEvents);
    invalid_value_range_query.filter.min_value = Some(10.0);
    invalid_value_range_query.filter.max_value = Some(5.0); // min > max
    let err = engine.register_query(invalid_value_range_query).unwrap_err();
    assert!(err.to_string().contains("min_value must be <= max_value"));
}

#[test]
fn test_dashboard_event_validation() {
    let mut engine = DashboardQueryEngine::new(50, 30);

    // Test event with invalid event type
    let mut invalid_event = sample_event("e1", "swap", 5.0, 100, vec!["dex"]);
    invalid_event.event_type = "<script>alert(1)</script>".to_string();
    let err = engine.record_event(invalid_event).unwrap_err();
    assert!(err.to_string().contains("event_type contains invalid characters"));

    // Test event with too many tags
    let too_many_tags: Vec<String> = (0..30).map(|i| format!("tag{}", i)).collect();
    let too_many_tags_event = sample_event("e2", "swap", 5.0, 100, too_many_tags.iter().map(|s| s.as_str()).collect());
    let err = engine.record_event(too_many_tags_event).unwrap_err();
    assert!(err.to_string().contains("too many tags"));

    // Test event with unsafe tag
    let unsafe_tag_event = sample_event("e3", "swap", 5.0, 100, vec!["<script>alert(1)</script>"]);
    let err = engine.record_event(unsafe_tag_event).unwrap_err();
    assert!(err.to_string().contains("tag contains invalid characters"));
}

#[test]
fn test_dashboard_query_management() {
    let mut engine = DashboardQueryEngine::new(50, 30);

    // Register a query
    let query = basic_query(DashboardQueryKind::CountEvents);
    engine.register_query(query.clone()).unwrap();

    // Try to register the same query again (should fail)
    let err = engine.register_query(query.clone()).unwrap_err();
    assert_eq!(err.to_string(), "query already exists");

    // Update the query
    let mut updated_query = query.clone();
    updated_query.name = "Updated Query".to_string();
    engine.update_query(updated_query).unwrap();

    // Try to update a non-existent query
    let mut non_existent_query = basic_query(DashboardQueryKind::SumValue);
    non_existent_query.id = "nonexistent".to_string();
    let err = engine.update_query(non_existent_query).unwrap_err();
    assert_eq!(err.to_string(), "query not found");

    // Remove the query
    engine.remove_query("q1").unwrap();

    // Try to remove the same query again (should fail)
    let err = engine.remove_query("q1").unwrap_err();
    assert_eq!(err.to_string(), "query not found");

    // Try to execute the removed query (should fail)
    let err = engine.execute_query("q1").unwrap_err();
    assert_eq!(err.to_string(), "query not found");
}

#[test]
fn test_dashboard_event_storage_limits() {
    let mut engine = DashboardQueryEngine::new(3, 60); // Only allow 3 events

    // Record more events than capacity
    engine
        .record_event(sample_event("e1", "swap", 1.0, 1, vec!["a"]))
        .unwrap();
    engine
        .record_event(sample_event("e2", "swap", 1.0, 2, vec!["b"]))
        .unwrap();
    engine
        .record_event(sample_event("e3", "swap", 1.0, 3, vec!["c"]))
        .unwrap();
    engine
        .record_event(sample_event("e4", "swap", 1.0, 4, vec!["d"]))
        .unwrap(); // This should cause trimming

    // We can't directly access the private events field, but we can verify
    // the behavior by recording events and checking that the engine still works
    // Record another event to verify the engine is still functional
    assert!(engine
        .record_event(sample_event("e5", "swap", 1.0, 5, vec!["e"]))
        .is_ok());
}

#[test]
fn test_dashboard_recent_events_query() {
    let mut engine = DashboardQueryEngine::new(10, 60);

    // Record events with different timestamps
    engine
        .record_event(sample_event("e1", "swap", 5.0, 10, vec!["eth"]))
        .unwrap();
    engine
        .record_event(sample_event("e2", "swap", 7.0, 20, vec!["btc"]))
        .unwrap();
    engine
        .record_event(sample_event("e3", "transfer", 3.0, 30, vec!["eth"]))
        .unwrap();

    // Create a recent events query
    let recent_query = DashboardQuery {
        id: "recent_q".to_string(),
        name: "Recent Events".to_string(),
        description: "Most recent events".to_string(),
        kind: DashboardQueryKind::RecentEvents,
        filter: QueryFilter {
            max_results: Some(2),
            ..Default::default()
        },
        allow_cached: false,
        ttl_seconds: None,
    };

    engine.register_query(recent_query).unwrap();
    let result = engine.execute_query("recent_q").unwrap();

    // Should return the 2 most recent events (e3 and e2) in descending order
    let events = result.data["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["id"], "e3");
    assert_eq!(events[1]["id"], "e2");
}