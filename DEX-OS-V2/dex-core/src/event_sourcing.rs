//! Event sourcing append-only event store.
//!
//! Implements the Priority 3 feature from DEX-OS-V2.csv:
//! - Distributed Systems,Distributed Systems,Distributed Systems,Event Sourcing,Append-only Event Store,Medium

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Metadata associated with an event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EventMetadata {
    /// Correlation identifier for tracing a request.
    pub correlation_id: Option<String>,
    /// Causation identifier for upstream event linkage.
    pub causation_id: Option<String>,
    /// Optional source identifier (service, node, etc.).
    pub source: Option<String>,
    /// Arbitrary tags for filtering/observability.
    pub tags: Vec<String>,
}

/// A fully materialized event in the append-only log.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventRecord {
    /// Event identifier (unique across the store).
    pub id: String,
    /// Stream identifier (aggregate ID).
    pub stream_id: String,
    /// Global sequence number (monotonic across all streams).
    pub sequence: u64,
    /// Stream-specific version (1-based, monotonic per stream).
    pub version: u64,
    /// Application-specific event type.
    pub event_type: String,
    /// Event payload.
    pub payload: serde_json::Value,
    /// Event metadata.
    pub metadata: EventMetadata,
    /// Timestamp when the event was persisted (ms since UNIX epoch).
    pub timestamp: u64,
}

/// New event data prior to being committed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewEvent {
    /// Optional caller-provided event id (useful for idempotency).
    pub id: Option<String>,
    /// Application-specific event type.
    pub event_type: String,
    /// Event payload.
    pub payload: serde_json::Value,
    /// Event metadata.
    pub metadata: EventMetadata,
}

impl NewEvent {
    /// Create a new event with the provided type and payload.
    pub fn new(event_type: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: None,
            event_type: event_type.into(),
            payload,
            metadata: EventMetadata::default(),
        }
    }

    /// Attach metadata to the event.
    pub fn with_metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Assign a specific identifier to the event (for idempotency tests).
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// Snapshot of an aggregate stream for faster rehydration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    /// Stream identifier.
    pub stream_id: String,
    /// Stream version captured in the snapshot.
    pub version: u64,
    /// Serialized state payload.
    pub state: serde_json::Value,
    /// Snapshot metadata.
    pub metadata: SnapshotMetadata,
}

/// Metadata describing a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SnapshotMetadata {
    /// Timestamp when the snapshot was taken (ms since UNIX epoch).
    pub taken_at: u64,
    /// Optional identifier for who/what captured the snapshot.
    pub taken_by: Option<String>,
    /// Tags for classifying snapshots (e.g., "checkpoint", "recovery").
    pub tags: Vec<String>,
}

/// Errors raised by the append-only event store.
#[derive(Debug, Error, PartialEq)]
pub enum EventStoreError {
    /// The requested stream does not exist.
    #[error("stream not found: {0}")]
    StreamNotFound(String),
    /// The stream already exists but the caller expected none.
    #[error("stream already exists: {0}")]
    StreamAlreadyExists(String),
    /// Optimistic concurrency failure when appending to a stream.
    #[error(
        "concurrency conflict for stream {stream_id}: expected version {expected}, actual version {actual}"
    )]
    Concurrency {
        /// Stream identifier.
        stream_id: String,
        /// Expected version supplied by the caller.
        expected: u64,
        /// Actual version present in the store.
        actual: u64,
    },
    /// Duplicate event id detected (idempotency violation).
    #[error("duplicate event id: {0}")]
    DuplicateEventId(String),
    /// Snapshot version is older than the current snapshot.
    #[error(
        "snapshot version {attempted} is behind existing snapshot version {current} for stream {stream_id}"
    )]
    SnapshotBehind {
        /// Stream identifier.
        stream_id: String,
        /// Attempted snapshot version.
        attempted: u64,
        /// Current snapshot version.
        current: u64,
    },
    /// Snapshot version is ahead of the stream state.
    #[error(
        "snapshot version {attempted} is ahead of stream version {current} for stream {stream_id}"
    )]
    SnapshotInFuture {
        /// Stream identifier.
        stream_id: String,
        /// Attempted snapshot version.
        attempted: u64,
        /// Current stream version.
        current: u64,
    },
}

/// Expected version contract for optimistic concurrency control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedVersion {
    /// Accept whatever the current stream version is.
    Any,
    /// Expect the stream to be at a specific version.
    Exact(u64),
    /// Expect the stream to not exist yet.
    NoStream,
}

/// Append-only event store used for event sourcing.
#[derive(Debug, Default)]
pub struct AppendOnlyEventStore {
    streams: HashMap<String, Vec<EventRecord>>,
    global_log: Vec<EventRecord>,
    snapshots: HashMap<String, Snapshot>,
    seen_ids: HashSet<String>,
    last_sequence: u64,
}

impl AppendOnlyEventStore {
    /// Create a new in-memory append-only event store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append events to a stream while enforcing optimistic concurrency.
    pub fn append_to_stream(
        &mut self,
        stream_id: impl Into<String>,
        expected_version: ExpectedVersion,
        events: Vec<NewEvent>,
    ) -> Result<Vec<EventRecord>, EventStoreError> {
        let stream_id = stream_id.into();
        let stream_exists = self.streams.contains_key(&stream_id);
        let current_version = self
            .streams
            .get(&stream_id)
            .and_then(|s| s.last().map(|e| e.version))
            .unwrap_or(0);

        match expected_version {
            ExpectedVersion::NoStream if stream_exists => {
                return Err(EventStoreError::StreamAlreadyExists(stream_id))
            }
            ExpectedVersion::Exact(expected) if expected != current_version => {
                return Err(EventStoreError::Concurrency {
                    stream_id,
                    expected,
                    actual: current_version,
                })
            }
            _ => {}
        }

        let base_sequence = self.last_sequence;
        let now = current_time_millis();
        let mut prepared = Vec::with_capacity(events.len());
        let mut batch_ids: HashSet<String> = HashSet::new();

        for (offset, event) in events.into_iter().enumerate() {
            let version = current_version + offset as u64 + 1;
            let event_id = event
                .id
                .unwrap_or_else(|| format!("{}-{}-{}", stream_id, version, now + offset as u64));

            if !batch_ids.insert(event_id.clone()) || self.seen_ids.contains(&event_id) {
                return Err(EventStoreError::DuplicateEventId(event_id));
            }

            let record = EventRecord {
                id: event_id,
                stream_id: stream_id.clone(),
                sequence: base_sequence + offset as u64 + 1,
                version,
                event_type: event.event_type,
                payload: event.payload,
                metadata: event.metadata,
                timestamp: now,
            };
            prepared.push(record);
        }

        if prepared.is_empty() {
            return Ok(Vec::new());
        }

        self.last_sequence = base_sequence + prepared.len() as u64;
        let stream = self.streams.entry(stream_id.clone()).or_default();
        stream.extend(prepared.iter().cloned());
        self.global_log.extend(prepared.iter().cloned());
        for record in &prepared {
            self.seen_ids.insert(record.id.clone());
        }

        Ok(prepared)
    }

    /// Read all events for a stream (oldest first).
    pub fn read_stream(&self, stream_id: &str) -> Result<Vec<EventRecord>, EventStoreError> {
        self.streams
            .get(stream_id)
            .cloned()
            .ok_or_else(|| EventStoreError::StreamNotFound(stream_id.to_string()))
    }

    /// Read events for a stream from a specific version onward.
    pub fn read_stream_from(
        &self,
        stream_id: &str,
        from_version: u64,
    ) -> Result<Vec<EventRecord>, EventStoreError> {
        let stream = self
            .streams
            .get(stream_id)
            .ok_or_else(|| EventStoreError::StreamNotFound(stream_id.to_string()))?;

        Ok(stream
            .iter()
            .filter(|evt| evt.version >= from_version)
            .cloned()
            .collect())
    }

    /// Read all events from the global log starting at a given sequence.
    pub fn read_all(&self, from_sequence: u64) -> Vec<EventRecord> {
        self.global_log
            .iter()
            .filter(|evt| evt.sequence >= from_sequence)
            .cloned()
            .collect()
    }

    /// Return the current version for a stream (0 if it does not yet exist).
    pub fn stream_version(&self, stream_id: &str) -> u64 {
        self.streams
            .get(stream_id)
            .and_then(|s| s.last().map(|evt| evt.version))
            .unwrap_or(0)
    }

    /// Upsert a snapshot, enforcing monotonic versioning.
    pub fn upsert_snapshot(&mut self, snapshot: Snapshot) -> Result<(), EventStoreError> {
        let stream_id = snapshot.stream_id.clone();
        let stream_version = self.stream_version(&stream_id);

        if snapshot.version > stream_version {
            return Err(EventStoreError::SnapshotInFuture {
                stream_id,
                attempted: snapshot.version,
                current: stream_version,
            });
        }

        if let Some(existing) = self.snapshots.get(&snapshot.stream_id) {
            if snapshot.version < existing.version {
                return Err(EventStoreError::SnapshotBehind {
                    stream_id,
                    attempted: snapshot.version,
                    current: existing.version,
                });
            }
        }

        self.snapshots.insert(snapshot.stream_id.clone(), snapshot);
        Ok(())
    }

    /// Retrieve the latest snapshot for a stream, if any.
    pub fn latest_snapshot(&self, stream_id: &str) -> Option<Snapshot> {
        self.snapshots.get(stream_id).cloned()
    }

    /// Return lightweight statistics for observability.
    pub fn stats(&self) -> EventStoreStats {
        EventStoreStats {
            stream_count: self.streams.len(),
            event_count: self.global_log.len(),
            snapshot_count: self.snapshots.len(),
            last_sequence: self.last_sequence,
        }
    }
}

/// Event store statistics snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventStoreStats {
    /// Number of active streams.
    pub stream_count: usize,
    /// Number of persisted events.
    pub event_count: usize,
    /// Number of stored snapshots.
    pub snapshot_count: usize,
    /// Last issued global sequence number.
    pub last_sequence: u64,
}

fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn appends_assign_versions_and_sequences_per_stream() {
        let mut store = AppendOnlyEventStore::new();
        let appended = store
            .append_to_stream(
                "orders-1",
                ExpectedVersion::NoStream,
                vec![
                    NewEvent::new("OrderCreated", json!({"id": 1})),
                    NewEvent::new("OrderUpdated", json!({"id": 1, "status": "open"})),
                ],
            )
            .unwrap();

        assert_eq!(appended.len(), 2);
        assert_eq!(appended[0].version, 1);
        assert_eq!(appended[1].version, 2);
        assert_eq!(appended[0].sequence + 1, appended[1].sequence);

        let stream = store.read_stream("orders-1").unwrap();
        assert_eq!(stream.len(), 2);
        assert_eq!(stream[1].event_type, "OrderUpdated");
    }

    #[test]
    fn optimistic_concurrency_prevents_conflicts() {
        let mut store = AppendOnlyEventStore::new();
        store
            .append_to_stream(
                "orders-2",
                ExpectedVersion::NoStream,
                vec![NewEvent::new("OrderCreated", json!({"id": 2}))],
            )
            .unwrap();
        store
            .append_to_stream(
                "orders-2",
                ExpectedVersion::Exact(1),
                vec![NewEvent::new("OrderConfirmed", json!({"id": 2}))],
            )
            .unwrap();

        let err = store
            .append_to_stream(
                "orders-2",
                ExpectedVersion::Exact(1),
                vec![NewEvent::new("OrderCancelled", json!({"id": 2}))],
            )
            .unwrap_err();

        match err {
            EventStoreError::Concurrency {
                expected, actual, ..
            } => {
                assert_eq!(expected, 1);
                assert_eq!(actual, 2);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn duplicate_ids_are_rejected_to_keep_log_idempotent() {
        let mut store = AppendOnlyEventStore::new();
        let duplicate_id = "evt-123".to_string();

        store
            .append_to_stream(
                "accounts-1",
                ExpectedVersion::NoStream,
                vec![NewEvent::new("AccountOpened", json!({ "id": 1 }))
                    .with_id(duplicate_id.clone())],
            )
            .unwrap();

        let err = store
            .append_to_stream(
                "accounts-1",
                ExpectedVersion::Any,
                vec![NewEvent::new("AccountCredited", json!({ "id": 1, "amount": 50 }))
                    .with_id(duplicate_id.clone())],
            )
            .unwrap_err();

        match err {
            EventStoreError::DuplicateEventId(id) => assert_eq!(id, duplicate_id),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn global_log_preserves_cross_stream_ordering() {
        let mut store = AppendOnlyEventStore::new();

        store
            .append_to_stream(
                "orders-3",
                ExpectedVersion::NoStream,
                vec![NewEvent::new("OrderCreated", json!({"id": 3}))],
            )
            .unwrap();
        store
            .append_to_stream(
                "payments-3",
                ExpectedVersion::NoStream,
                vec![
                    NewEvent::new("PaymentInitiated", json!({"order_id": 3})),
                    NewEvent::new("PaymentCaptured", json!({"order_id": 3})),
                ],
            )
            .unwrap();

        let all_events = store.read_all(1);
        assert_eq!(all_events.len(), 3);
        assert_eq!(all_events[0].stream_id, "orders-3");
        assert_eq!(all_events[1].stream_id, "payments-3");
        assert!(all_events[0].sequence < all_events[1].sequence);
        assert_eq!(all_events[1].sequence + 1, all_events[2].sequence);
    }

    #[test]
    fn snapshots_enforce_monotonic_versions() {
        let mut store = AppendOnlyEventStore::new();
        store
            .append_to_stream(
                "portfolio-1",
                ExpectedVersion::NoStream,
                vec![
                    NewEvent::new("AssetAdded", json!({"symbol": "ETH", "amount": 1.0})),
                    NewEvent::new("AssetAdded", json!({"symbol": "BTC", "amount": 0.5})),
                ],
            )
            .unwrap();

        let snapshot = Snapshot {
            stream_id: "portfolio-1".to_string(),
            version: 2,
            state: json!({"ETH": 1.0, "BTC": 0.5}),
            metadata: SnapshotMetadata {
                taken_at: 1_700_000_000_000,
                taken_by: Some("node-a".to_string()),
                tags: vec!["checkpoint".to_string()],
            },
        };
        store.upsert_snapshot(snapshot).unwrap();

        let behind_err = store
            .upsert_snapshot(Snapshot {
                stream_id: "portfolio-1".to_string(),
                version: 1,
                state: json!({"ETH": 1.0}),
                metadata: SnapshotMetadata::default(),
            })
            .unwrap_err();
        assert!(matches!(behind_err, EventStoreError::SnapshotBehind { current: 2, .. }));

        let future_err = store
            .upsert_snapshot(Snapshot {
                stream_id: "portfolio-1".to_string(),
                version: 5,
                state: json!({"ETH": 1.0, "BTC": 1.0}),
                metadata: SnapshotMetadata::default(),
            })
            .unwrap_err();
        assert!(matches!(
            future_err,
            EventStoreError::SnapshotInFuture {
                attempted: 5,
                current: 2,
                ..
            }
        ));

        let latest = store.latest_snapshot("portfolio-1").unwrap();
        assert_eq!(latest.version, 2);
        assert_eq!(latest.metadata.taken_by.unwrap(), "node-a");
    }

    #[test]
    fn read_stream_from_supports_partial_rehydration() {
        let mut store = AppendOnlyEventStore::new();
        store
            .append_to_stream(
                "positions-1",
                ExpectedVersion::NoStream,
                vec![
                    NewEvent::new("PositionOpened", json!({"id": 1, "qty": 10})),
                    NewEvent::new("PositionIncreased", json!({"id": 1, "qty": 5})),
                    NewEvent::new("PositionClosed", json!({"id": 1})),
                ],
            )
            .unwrap();

        let partial = store.read_stream_from("positions-1", 2).unwrap();
        assert_eq!(partial.len(), 2);
        assert_eq!(partial[0].version, 2);
        assert_eq!(partial[1].event_type, "PositionClosed");
    }

    #[test]
    fn no_stream_expectation_blocks_existing_streams() {
        let mut store = AppendOnlyEventStore::new();
        store
            .append_to_stream(
                "wallet-1",
                ExpectedVersion::NoStream,
                vec![NewEvent::new("WalletOpened", json!({"id": 1}))],
            )
            .unwrap();

        let err = store
            .append_to_stream(
                "wallet-1",
                ExpectedVersion::NoStream,
                vec![NewEvent::new("WalletCredited", json!({"id": 1, "amount": 10}))],
            )
            .unwrap_err();

        assert!(matches!(err, EventStoreError::StreamAlreadyExists(_)));
    }
}
