//! Dashboard query engine for on-chain analytics
//!
//! Implements the Priority 5 feature from DEX-OS-V2.csv:
//! "5,Analytics & Oracles,On-Chain Analytics,On-Chain Analytics,Dashboard Queries,Dashboard Querying,Medium {Security: Layer 4 - Application Security}"
//!
//! The engine provides:
//! - Input validation and output encoding to align with application security requirements.
//! - Query registration and execution with allow-listed aggregations.
//! - Result caching with TTL to avoid repeated computation for dashboards.
//! - Bounded storage and filter controls to prevent unbounded data growth.

use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use crate::network::MessageBroker;

/// Analytics event captured for dashboard queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    /// Unique identifier for the event
    pub id: String,
    /// Event type (e.g., "swap", "transfer", "liquidity")
    pub event_type: String,
    /// Tags attached to the event for filtering
    pub tags: Vec<String>,
    /// Numerical value associated with the event (volume, fee, etc.)
    pub value: f64,
    /// Event timestamp (seconds since epoch)
    pub timestamp: u64,
    /// Additional metadata (must be simple key/value strings)
    pub metadata: HashMap<String, String>,
}

/// Supported dashboard query aggregations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DashboardQueryKind {
    /// Count matching events
    CountEvents,
    /// Sum the `value` field of matching events
    SumValue,
    /// Average of the `value` field of matching events
    AverageValue,
    /// Return counts per tag (limited)
    TopTags { max: usize },
    /// Return recent events (limited)
    RecentEvents,
}

/// Filter to apply before aggregation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryFilter {
    /// Event types to include (empty means all)
    pub event_types: Vec<String>,
    /// Required tags (all must be present)
    pub tags: Vec<String>,
    /// Excluded tags (any present excludes the event)
    pub exclude_tags: Vec<String>,
    /// Optional time range (inclusive start, inclusive end)
    pub time_range: Option<(u64, u64)>,
    /// Minimum value to include
    pub min_value: Option<f64>,
    /// Maximum value to include
    pub max_value: Option<f64>,
    /// Cap for number of records returned in detail-oriented queries
    pub max_results: Option<usize>,
}

impl Default for QueryFilter {
    fn default() -> Self {
        Self {
            event_types: Vec::new(),
            tags: Vec::new(),
            exclude_tags: Vec::new(),
            time_range: None,
            min_value: None,
            max_value: None,
            max_results: None,
        }
    }
}

/// Dashboard query definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashboardQuery {
    /// Unique identifier for the query
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what the query returns
    pub description: String,
    /// Query kind / aggregation
    pub kind: DashboardQueryKind,
    /// Filter to apply
    pub filter: QueryFilter,
    /// Whether cached results can be used
    pub allow_cached: bool,
    /// Cache TTL in seconds (uses engine default when None)
    pub ttl_seconds: Option<u64>,
}

/// Result of a dashboard query
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashboardQueryResult {
    /// Query identifier
    pub query_id: String,
    /// Time the result was generated
    pub generated_at: u64,
    /// Encoded summary (HTML-escaped) for UI display
    pub summary: String,
    /// Aggregated data payload
    pub data: serde_json::Value,
    /// Metadata for downstream use
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct QueryCacheEntry {
    result: DashboardQueryResult,
    cached_at: u64,
}

/// Validates queries and events to enforce application security constraints
#[derive(Debug, Clone)]
struct DashboardQueryValidator {
    max_name_length: usize,
    max_description_length: usize,
    max_results: usize,
    max_tags: usize,
    max_id_length: usize,
}

impl DashboardQueryValidator {
    fn new() -> Self {
        Self {
            max_name_length: 120,
            max_description_length: 500,
            max_results: 500,
            max_tags: 20,
            max_id_length: 120,
        }
    }

    fn validate_query(&self, query: &DashboardQuery) -> Result<(), DashboardQueryError> {
        self.validate_identifier(&query.id, "query id")?;
        self.validate_safe_text(&query.name, self.max_name_length, "query name")?;
        self.validate_safe_text(
            &query.description,
            self.max_description_length,
            "description",
        )?;

        if let Some(ttl) = query.ttl_seconds {
            if ttl == 0 {
                return Err(DashboardQueryError::ValidationError(
                    "ttl_seconds must be greater than 0".to_string(),
                ));
            }
        }

        self.validate_filter(&query.filter)?;

        if let DashboardQueryKind::TopTags { max } = query.kind {
            if max == 0 || max > self.max_results {
                return Err(DashboardQueryError::ValidationError(format!(
                    "TopTags max must be between 1 and {}",
                    self.max_results
                )));
            }
        }

        Ok(())
    }

    fn validate_filter(&self, filter: &QueryFilter) -> Result<(), DashboardQueryError> {
        for et in &filter.event_types {
            self.validate_safe_text(et, self.max_name_length, "event_type")?;
        }
        self.validate_tags(&filter.tags)?;
        self.validate_tags(&filter.exclude_tags)?;

        if let Some((start, end)) = filter.time_range {
            if start > end {
                return Err(DashboardQueryError::ValidationError(
                    "time_range start must be <= end".to_string(),
                ));
            }
        }

        if let (Some(min), Some(max)) = (filter.min_value, filter.max_value) {
            if min > max {
                return Err(DashboardQueryError::ValidationError(
                    "min_value must be <= max_value".to_string(),
                ));
            }
        }

        if let Some(max_results) = filter.max_results {
            if max_results == 0 || max_results > self.max_results {
                return Err(DashboardQueryError::ValidationError(format!(
                    "max_results must be between 1 and {}",
                    self.max_results
                )));
            }
        }

        Ok(())
    }

    fn validate_event(&self, event: &AnalyticsEvent) -> Result<(), DashboardQueryError> {
        self.validate_identifier(&event.id, "event id")?;
        self.validate_safe_text(&event.event_type, self.max_name_length, "event_type")?;
        self.validate_tags(&event.tags)?;
        self.validate_metadata(&event.metadata)?;
        Ok(())
    }

    fn validate_identifier(&self, value: &str, field: &str) -> Result<(), DashboardQueryError> {
        self.validate_safe_text(value, self.max_id_length, field)
    }

    fn validate_tags(&self, tags: &[String]) -> Result<(), DashboardQueryError> {
        if tags.len() > self.max_tags {
            return Err(DashboardQueryError::ValidationError(format!(
                "too many tags (max {})",
                self.max_tags
            )));
        }

        for tag in tags {
            self.validate_safe_text(tag, 64, "tag")?;
        }

        Ok(())
    }

    fn validate_metadata(
        &self,
        metadata: &HashMap<String, String>,
    ) -> Result<(), DashboardQueryError> {
        if metadata.len() > 50 {
            return Err(DashboardQueryError::ValidationError(
                "metadata too large".to_string(),
            ));
        }

        for (k, v) in metadata {
            self.validate_safe_text(k, 64, "metadata key")?;
            self.validate_safe_text(v, 256, "metadata value")?;
        }

        Ok(())
    }

    fn validate_safe_text(
        &self,
        value: &str,
        max_len: usize,
        field: &str,
    ) -> Result<(), DashboardQueryError> {
        if value.is_empty() {
            return Err(DashboardQueryError::ValidationError(format!(
                "{} cannot be empty",
                field
            )));
        }

        if value.len() > max_len {
            return Err(DashboardQueryError::ValidationError(format!(
                "{} too long (max {})",
                field, max_len
            )));
        }

        // Allow alphanumeric, space, underscore, hyphen, and colon to keep outputs dashboard-safe
        if !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '-' | ':' | '.'))
        {
            return Err(DashboardQueryError::ValidationError(format!(
                "{} contains invalid characters",
                field
            )));
        }

        // Basic injection guardrails
        let lowered = value.to_lowercase();
        if lowered.contains("<script")
            || lowered.contains("javascript:")
            || lowered.contains("--")
            || lowered.contains(";")
        {
            return Err(DashboardQueryError::ValidationError(format!(
                "{} contains disallowed patterns",
                field
            )));
        }

        Ok(())
    }
}

/// Dashboard query engine
#[derive(Debug, Clone)]
pub struct DashboardQueryEngine {
    queries: HashMap<String, DashboardQuery>,
    events: Vec<AnalyticsEvent>,
    cache: HashMap<String, QueryCacheEntry>,
    max_events: usize,
    default_cache_ttl: u64,
    validator: DashboardQueryValidator,
    message_broker: Option<MessageBroker>,
    #[cfg(test)]
    time_override: Option<u64>,
}

impl DashboardQueryEngine {
    /// Create a new engine with a maximum event capacity and default cache TTL
    pub fn new(max_events: usize, default_cache_ttl: u64) -> Self {
        Self {
            queries: HashMap::new(),
            events: Vec::new(),
            cache: HashMap::new(),
            max_events,
            default_cache_ttl,
            validator: DashboardQueryValidator::new(),
            message_broker: None,
            #[cfg(test)]
            time_override: None,
        }
    }

    /// Create a new engine with an attached message broker for Pub-Sub.
    pub fn new_with_pubsub(
        max_events: usize,
        default_cache_ttl: u64,
        message_broker: MessageBroker,
    ) -> Self {
        Self {
            queries: HashMap::new(),
            events: Vec::new(),
            cache: HashMap::new(),
            max_events,
            default_cache_ttl,
            validator: DashboardQueryValidator::new(),
            message_broker: Some(message_broker),
            #[cfg(test)]
            time_override: None,
        }
    }

    /// Attach or replace the message broker after initialization.
    pub fn set_message_broker(&mut self, message_broker: MessageBroker) {
        self.message_broker = Some(message_broker);
    }

    /// Register a new dashboard query (id must be unique)
    pub fn register_query(&mut self, query: DashboardQuery) -> Result<(), DashboardQueryError> {
        if self.queries.contains_key(&query.id) {
            return Err(DashboardQueryError::QueryAlreadyExists);
        }

        self.validator.validate_query(&query)?;
        self.queries.insert(query.id.clone(), query);
        Ok(())
    }

    /// Update an existing dashboard query
    pub fn update_query(&mut self, query: DashboardQuery) -> Result<(), DashboardQueryError> {
        if !self.queries.contains_key(&query.id) {
            return Err(DashboardQueryError::QueryNotFound);
        }

        self.validator.validate_query(&query)?;
        let query_id = query.id.clone();
        self.queries.insert(query_id.clone(), query);
        self.cache.remove(&query_id);
        Ok(())
    }

    /// Remove a dashboard query
    pub fn remove_query(&mut self, query_id: &str) -> Result<(), DashboardQueryError> {
        if self.queries.remove(query_id).is_none() {
            return Err(DashboardQueryError::QueryNotFound);
        }
        self.cache.remove(query_id);
        Ok(())
    }

    /// Record an analytics event for use in dashboard queries
    pub fn record_event(&mut self, event: AnalyticsEvent) -> Result<(), DashboardQueryError> {
        self.validator.validate_event(&event)?;
        self.events.push(event);
        self.enforce_max_events();

        if let Some(broker) = &self.message_broker {
            if let Some(latest) = self.events.last() {
                let broker = broker.clone();
                let payload = serde_json::to_value(latest).unwrap_or(json!({}));
                tokio::spawn(async move {
                    let _ = broker.publish("dashboard.analytics", payload).await;
                });
            }
        }
        Ok(())
    }

    /// Execute a dashboard query and return a result (uses cache when allowed)
    pub fn execute_query(
        &mut self,
        query_id: &str,
    ) -> Result<DashboardQueryResult, DashboardQueryError> {
        let query = self
            .queries
            .get(query_id)
            .ok_or(DashboardQueryError::QueryNotFound)?
            .clone();

        let now = self.now_seconds();
        let ttl = query.ttl_seconds.unwrap_or(self.default_cache_ttl);

        if query.allow_cached {
            if let Some(entry) = self.cache.get(query_id) {
                if now.saturating_sub(entry.cached_at) <= ttl {
                    return Ok(entry.result.clone());
                }
            }
        }

        let filtered = self.filter_events(&query.filter);
        let result = self.aggregate(&query, &filtered, now)?;

        if query.allow_cached {
            self.cache.insert(
                query.id.clone(),
                QueryCacheEntry {
                    result: result.clone(),
                    cached_at: now,
                },
            );
        } else {
            self.cache.remove(&query.id);
        }

        Ok(result)
    }

    fn filter_events(&self, filter: &QueryFilter) -> Vec<AnalyticsEvent> {
        self.events
            .iter()
            .filter(|event| self.event_matches_filter(event, filter))
            .cloned()
            .collect()
    }

    fn event_matches_filter(&self, event: &AnalyticsEvent, filter: &QueryFilter) -> bool {
        if !filter.event_types.is_empty() && !filter.event_types.contains(&event.event_type) {
            return false;
        }

        if let Some((start, end)) = filter.time_range {
            if event.timestamp < start || event.timestamp > end {
                return false;
            }
        }

        if let Some(min) = filter.min_value {
            if event.value < min {
                return false;
            }
        }

        if let Some(max) = filter.max_value {
            if event.value > max {
                return false;
            }
        }

        for exclude in &filter.exclude_tags {
            if event.tags.contains(exclude) {
                return false;
            }
        }

        for required in &filter.tags {
            if !event.tags.contains(required) {
                return false;
            }
        }

        true
    }

    fn aggregate(
        &self,
        query: &DashboardQuery,
        events: &[AnalyticsEvent],
        now: u64,
    ) -> Result<DashboardQueryResult, DashboardQueryError> {
        let data = match &query.kind {
            DashboardQueryKind::CountEvents => json!({ "count": events.len() }),
            DashboardQueryKind::SumValue => {
                let sum: f64 = events.iter().map(|e| e.value).sum();
                json!({ "sum": sum })
            }
            DashboardQueryKind::AverageValue => {
                let sum: f64 = events.iter().map(|e| e.value).sum();
                let average = if events.is_empty() {
                    0.0
                } else {
                    sum / events.len() as f64
                };
                json!({ "average": average })
            }
            DashboardQueryKind::TopTags { max } => {
                let limit = (*max).min(self.validator.max_results);
                let mut counts: HashMap<String, usize> = HashMap::new();
                for event in events {
                    for tag in &event.tags {
                        *counts.entry(tag.clone()).or_insert(0) += 1;
                    }
                }

                let mut pairs: Vec<(String, usize)> = counts.into_iter().collect();
                pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                let trimmed = pairs.into_iter().take(limit).collect::<Vec<_>>();
                json!({ "top_tags": trimmed })
            }
            DashboardQueryKind::RecentEvents => {
                let limit = query
                    .filter
                    .max_results
                    .unwrap_or(self.validator.max_results)
                    .min(self.validator.max_results);

                let mut sorted = events.to_vec();
                sorted.sort_by_key(|e| e.timestamp);
                sorted.reverse();
                let trimmed: Vec<AnalyticsEvent> = sorted.into_iter().take(limit).collect();
                json!({ "events": trimmed })
            }
        };

        let summary = escape_html(&format!(
            "{} ({})",
            query.name,
            match query.kind {
                DashboardQueryKind::CountEvents => "count",
                DashboardQueryKind::SumValue => "sum",
                DashboardQueryKind::AverageValue => "average",
                DashboardQueryKind::TopTags { .. } => "top_tags",
                DashboardQueryKind::RecentEvents => "recent_events",
            }
        ));

        Ok(DashboardQueryResult {
            query_id: query.id.clone(),
            generated_at: now,
            summary,
            data,
            metadata: HashMap::new(),
        })
    }

    fn enforce_max_events(&mut self) {
        if self.events.len() > self.max_events {
            let excess = self.events.len() - self.max_events;
            self.events.drain(0..excess);
        }
    }

    fn now_seconds(&self) -> u64 {
        #[cfg(test)]
        if let Some(override_now) = self.time_override {
            return override_now;
        }

        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    #[cfg(test)]
    fn set_time_override(&mut self, value: Option<u64>) {
        self.time_override = value;
    }
}

/// Errors produced by the dashboard query engine
#[derive(Debug, Error, PartialEq)]
pub enum DashboardQueryError {
    #[error("query already exists")]
    QueryAlreadyExists,
    #[error("query not found")]
    QueryNotFound,
    #[error("validation error: {0}")]
    ValidationError(String),
}

fn escape_html(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn basic_query(kind: DashboardQueryKind) -> DashboardQuery {
        DashboardQuery {
            id: "q1".to_string(),
            name: "Volume".to_string(),
            description: "Counts trades".to_string(),
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
    fn registers_and_executes_count_query() {
        let mut engine = DashboardQueryEngine::new(100, 60);
        engine
            .record_event(sample_event("e1", "swap", 10.0, 1000, vec!["dex", "eth"]))
            .unwrap();
        engine
            .record_event(sample_event("e2", "swap", 5.0, 1001, vec!["dex", "usdc"]))
            .unwrap();

        let query = basic_query(DashboardQueryKind::CountEvents);
        engine.register_query(query).unwrap();

        engine.set_time_override(Some(1100));
        let result = engine.execute_query("q1").unwrap();
        assert_eq!(result.query_id, "q1");
        assert_eq!(result.generated_at, 1100);
        assert_eq!(result.data["count"], json!(2));
    }

    #[test]
    fn rejects_unsafe_query_names() {
        let mut engine = DashboardQueryEngine::new(50, 30);
        let mut query = basic_query(DashboardQueryKind::CountEvents);
        query.name = "<script>alert(1)</script>".to_string();
        let err = engine.register_query(query).unwrap_err();
        assert_eq!(
            err,
            DashboardQueryError::ValidationError(
                "query name contains invalid characters".to_string()
            )
        );
    }

    #[test]
    fn enforces_caching_and_ttl() {
        let mut engine = DashboardQueryEngine::new(100, 120);
        let mut query = basic_query(DashboardQueryKind::SumValue);
        query.ttl_seconds = Some(200);
        engine.register_query(query).unwrap();

        engine
            .record_event(sample_event("e1", "swap", 10.0, 1, vec!["dex"]))
            .unwrap();
        engine.set_time_override(Some(10));
        let first = engine.execute_query("q1").unwrap();
        assert_eq!(first.data["sum"], json!(10.0));

        // Add new event but keep time within TTL; cached result should be returned
        engine
            .record_event(sample_event("e2", "swap", 5.0, 20, vec!["dex"]))
            .unwrap();
        engine.set_time_override(Some(50));
        let cached = engine.execute_query("q1").unwrap();
        assert_eq!(cached.data["sum"], json!(10.0));

        // Move past TTL to force recompute
        engine.set_time_override(Some(400));
        let refreshed = engine.execute_query("q1").unwrap();
        assert_eq!(refreshed.data["sum"], json!(15.0));
    }

    #[test]
    fn respects_filters_and_limits() {
        let mut engine = DashboardQueryEngine::new(10, 60);
        engine
            .record_event(sample_event("e1", "swap", 5.0, 10, vec!["eth", "dex"]))
            .unwrap();
        engine
            .record_event(sample_event("e2", "swap", 7.0, 20, vec!["btc", "dex"]))
            .unwrap();
        engine
            .record_event(sample_event("e3", "transfer", 3.0, 30, vec!["eth"]))
            .unwrap();

        let query = DashboardQuery {
            id: "qtags".to_string(),
            name: "Top Tags".to_string(),
            description: "Top tags by frequency".to_string(),
            kind: DashboardQueryKind::TopTags { max: 2 },
            filter: QueryFilter {
                event_types: vec!["swap".to_string()],
                tags: vec!["dex".to_string()],
                exclude_tags: vec!["bot".to_string()],
                time_range: Some((0, 25)),
                min_value: Some(1.0),
                max_value: Some(10.0),
                max_results: Some(5),
            },
            allow_cached: false,
            ttl_seconds: None,
        };

        engine.register_query(query).unwrap();
        engine.set_time_override(Some(35));
        let result = engine.execute_query("qtags").unwrap();
        let top_tags = result.data["top_tags"].as_array().unwrap();
        assert_eq!(top_tags.len(), 2); // two tags returned (eth and dex)
    }

    #[test]
    fn trims_events_when_capacity_exceeded() {
        let mut engine = DashboardQueryEngine::new(2, 60);
        engine
            .record_event(sample_event("e1", "swap", 1.0, 1, vec!["a"]))
            .unwrap();
        engine
            .record_event(sample_event("e2", "swap", 1.0, 2, vec!["b"]))
            .unwrap();
        engine
            .record_event(sample_event("e3", "swap", 1.0, 3, vec!["c"]))
            .unwrap();

        assert_eq!(engine.events.len(), 2);
        assert!(engine.events.iter().all(|e| e.id != "e1"));
    }
}
