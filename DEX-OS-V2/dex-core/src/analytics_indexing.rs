//! Graph-style indexing service for Event-driven On-Chain Analytics
//!
//! Implements the "Indexing Services (The Graph)" feature from DEX-OS-V2.csv by capturing
//! analytics events, storing them in an in-memory graph-style index, and exposing query helpers.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;
use uuid::Uuid;

/// Represents the analytics schema handled by the indexer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaDefinition {
    /// Name of the subgraph or schema.
    pub name: String,
    /// Human-readable version.
    pub version: String,
    /// Field names that the schema tracks.
    pub fields: Vec<String>,
}

/// Analytics event emitted from the core system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    /// Event type (e.g., "trade_executed", "account_update")
    pub event_type: String,
    /// Source subsystem (e.g., "orderbook", "bridge")
    pub source: String,
    /// Optional block number when the event occurred.
    pub block_number: Option<u64>,
    /// Payload data captured by the event.
    pub payload: Map<String, Value>,
    /// Optional timestamp (seconds since epoch). Defaults to when the indexer ingests it.
    pub timestamp: Option<u64>,
}

/// Indexed event stored by the service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexedEvent {
    /// Unique identifier assigned by the indexer.
    pub id: String,
    /// Bound schema name.
    pub schema_name: String,
    /// Schema version.
    pub schema_version: String,
    /// Event type, copied from the source event.
    pub event_type: String,
    /// Source subsystem name.
    pub source: String,
    /// Block number at indexing time.
    pub block_number: u64,
    /// Event payload.
    pub payload: Map<String, Value>,
    /// Timestamp at indexing time.
    pub timestamp: u64,
}

/// Indexing metrics exposed to analytics dashboards.
#[derive(Debug, Clone, PartialEq)]
pub struct IndexMetrics {
    pub total_events: u64,
    pub unique_event_types: usize,
    pub latest_block: u64,
}

/// Error types for indexing operations.
#[derive(Debug, Error)]
pub enum IndexingError {
    #[error("event payload missing required field: {0}")]
    MissingField(String),
    #[error("schema mismatch: expected field(s) {0:?}")]
    SchemaMismatch(Vec<String>),
    #[error("internal mutex poisoned")]
    LockPoisoned,
}

/// Filter used to query indexed events.
#[derive(Debug, Clone)]
pub struct EventQuery {
    pub event_type: Option<String>,
    pub source: Option<String>,
    pub min_block: Option<u64>,
    pub max_block: Option<u64>,
}

impl Default for EventQuery {
    fn default() -> Self {
        Self {
            event_type: None,
            source: None,
            min_block: None,
            max_block: None,
        }
    }
}

/// In-memory Graph-style Indexing Service.
#[derive(Debug, Clone)]
pub struct IndexingService {
    subgraph_name: String,
    schema: SchemaDefinition,
    events: Arc<Mutex<Vec<IndexedEvent>>>,
    event_counts: Arc<Mutex<HashMap<String, u64>>>,
    latest_block: Arc<AtomicU64>,
}

impl IndexingService {
    /// Construct a new indexing service bound to the provided subgraph and schema.
    pub fn new(subgraph_name: String, schema: SchemaDefinition) -> Self {
        Self {
            subgraph_name,
            schema,
            events: Arc::new(Mutex::new(Vec::new())),
            event_counts: Arc::new(Mutex::new(HashMap::new())),
            latest_block: Arc::new(AtomicU64::new(0)),
        }
    }

    fn timestamp_or_now(&self, ts: Option<u64>) -> u64 {
        ts.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        })
    }

    fn require_fields(&self, payload: &Map<String, Value>) -> Result<(), IndexingError> {
        for field in self.schema.fields.iter() {
            if !payload.contains_key(field) {
                return Err(IndexingError::MissingField(field.clone()));
            }
        }
        Ok(())
    }

    /// Ingest an analytics event into the index.
    pub fn ingest_event(
        &self,
        event: AnalyticsEvent,
    ) -> Result<IndexedEvent, IndexingError> {
        self.require_fields(&event.payload)?;
        let block_number = event.block_number.unwrap_or_else(|| {
            self.latest_block.load(Ordering::Relaxed)
        });
        let timestamp = self.timestamp_or_now(event.timestamp);
        let id = format!("{}-{}", self.subgraph_name, Uuid::new_v4());

        let indexed = IndexedEvent {
            id,
            schema_name: self.schema.name.clone(),
            schema_version: self.schema.version.clone(),
            event_type: event.event_type.clone(),
            source: event.source.clone(),
            block_number,
            payload: event.payload.clone(),
            timestamp,
        };

        let mut events = self
            .events
            .lock()
            .map_err(|_| IndexingError::LockPoisoned)?;
        events.push(indexed.clone());

        let mut counts = self
            .event_counts
            .lock()
            .map_err(|_| IndexingError::LockPoisoned)?;
        *counts.entry(event.event_type).or_insert(0) += 1;

        if block_number > self.latest_block.load(Ordering::Relaxed) {
            self.latest_block.store(block_number, Ordering::Relaxed);
        }

        Ok(indexed)
    }

    /// Query the indexed events using a simple filter.
    pub fn query(&self, query: EventQuery) -> Result<Vec<IndexedEvent>, IndexingError> {
        let events = self
            .events
            .lock()
            .map_err(|_| IndexingError::LockPoisoned)?;

        let results = events
            .iter()
            .filter(|event| {
                if let Some(ref event_type) = query.event_type {
                    if &event.event_type != event_type {
                        return false;
                    }
                }
                if let Some(ref source) = query.source {
                    if &event.source != source {
                        return false;
                    }
                }
                if let Some(min_block) = query.min_block {
                    if event.block_number < min_block {
                        return false;
                    }
                }
                if let Some(max_block) = query.max_block {
                    if event.block_number > max_block {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        Ok(results)
    }

    /// Get metrics about the indexed data.
    pub fn metrics(&self) -> Result<IndexMetrics, IndexingError> {
        let counts = self
            .event_counts
            .lock()
            .map_err(|_| IndexingError::LockPoisoned)?;

        let events = self
            .events
            .lock()
            .map_err(|_| IndexingError::LockPoisoned)?;

        let total = events.len() as u64;

        Ok(IndexMetrics {
            total_events: total,
            unique_event_types: counts.len(),
            latest_block: self.latest_block.load(Ordering::Relaxed),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_schema() -> SchemaDefinition {
        SchemaDefinition {
            name: "trades".to_string(),
            version: "1.0".to_string(),
            fields: vec![
                "volume".to_string(),
                "price".to_string(),
                "trader".to_string(),
            ],
        }
    }

    fn sample_event() -> AnalyticsEvent {
        let mut payload = Map::new();
        payload.insert("volume".to_string(), json!(1000));
        payload.insert("price".to_string(), json!(42.0));
        payload.insert("trader".to_string(), json!("trader1"));

        AnalyticsEvent {
            event_type: "trade_executed".to_string(),
            source: "orderbook".to_string(),
            block_number: Some(1234),
            payload,
            timestamp: None,
        }
    }

    #[test]
    fn ingest_and_query_success() {
        let service = IndexingService::new("trades".to_string(), sample_schema());
        let indexed = service.ingest_event(sample_event()).unwrap();
        assert_eq!(indexed.event_type, "trade_executed");
        assert_eq!(indexed.schema_name, "trades");

        let metrics = service.metrics().unwrap();
        assert_eq!(metrics.total_events, 1);
        assert_eq!(metrics.unique_event_types, 1);

        let events = service
            .query(EventQuery {
                event_type: Some("trade_executed".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn missing_field_error() {
        let schema = sample_schema();
        let service = IndexingService::new("trades".to_string(), schema.clone());

        let mut payload = Map::new();
        payload.insert("volume".to_string(), json!(1000));

        let result = service.ingest_event(AnalyticsEvent {
            event_type: "trade_executed".to_string(),
            source: "orderbook".to_string(),
            block_number: Some(1),
            payload,
            timestamp: None,
        });

        assert!(matches!(result, Err(IndexingError::MissingField(field)) if field == "price"));
    }

    #[test]
    fn query_filters_block_range() {
        let service = IndexingService::new("trades".to_string(), sample_schema());
        let mut event = sample_event();
        event.block_number = Some(10);
        let _ = service.ingest_event(event.clone()).unwrap();

        event.block_number = Some(20);
        let _ = service.ingest_event(event).unwrap();

        let results = service
            .query(EventQuery {
                min_block: Some(15),
                max_block: Some(25),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].block_number >= 15);
    }
}
