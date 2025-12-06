//! Pub-Sub message broker for distributed systems.
//!
//! Implements the Priority 3 feature from DEX-OS-V2.csv:
//! - Distributed Systems,Distributed Systems,Distributed Systems,Pub-Sub,Message Brokers,Medium

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::{broadcast, RwLock};

/// Configuration for the message broker.
#[derive(Debug, Clone)]
pub struct PubSubConfig {
    /// Maximum buffered messages per topic before older ones are dropped.
    pub channel_capacity: usize,
    /// Maximum distinct topics the broker will create.
    pub max_topics: usize,
}

impl Default for PubSubConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 64,
            max_topics: 128,
        }
    }
}

/// Standard message shape delivered through the broker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PubSubMessage {
    /// Topic the message belongs to.
    pub topic: String,
    /// JSON payload for flexibility.
    pub payload: serde_json::Value,
    /// Milliseconds since Unix epoch when the message was published.
    pub timestamp_ms: u64,
}

/// Broker-level statistics for a single topic.
#[derive(Debug, Default, Clone)]
pub struct TopicStats {
    /// Number of active subscribers on the topic.
    pub subscribers: usize,
    /// Messages successfully published to the topic.
    pub published: u64,
    /// Messages dropped because no subscribers were present.
    pub dropped: u64,
}

/// Errors that can occur during publish/subscribe operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PubSubError {
    #[error("topic name cannot be empty")]
    EmptyTopic,
    #[error("maximum topics reached: {0}")]
    TopicLimitReached(usize),
    #[error("no subscribers for topic {0}")]
    NoSubscribers(String),
    #[error("channel closed for topic {0}")]
    ChannelClosed(String),
}

#[derive(Clone, Debug)]
struct TopicState {
    sender: broadcast::Sender<PubSubMessage>,
    subscribers: Arc<AtomicUsize>,
    published: Arc<AtomicU64>,
    dropped: Arc<AtomicU64>,
}

impl TopicState {
    fn new(sender: broadcast::Sender<PubSubMessage>) -> Self {
        Self {
            sender,
            subscribers: Arc::new(AtomicUsize::new(0)),
            published: Arc::new(AtomicU64::new(0)),
            dropped: Arc::new(AtomicU64::new(0)),
        }
    }
}

/// In-memory message broker with per-topic fan-out semantics.
#[derive(Clone)]
pub struct MessageBroker {
    config: PubSubConfig,
    topics: Arc<RwLock<HashMap<String, TopicState>>>,
}

impl std::fmt::Debug for MessageBroker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageBroker")
            .field("config", &self.config)
            .finish()
    }
}

impl MessageBroker {
    /// Create a broker with the provided configuration.
    pub fn new(config: PubSubConfig) -> Self {
        Self {
            config,
            topics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to a topic, creating it if needed.
    pub async fn subscribe(&self, topic: impl Into<String>) -> Result<Subscription, PubSubError> {
        let topic_name = topic.into();
        if topic_name.trim().is_empty() {
            return Err(PubSubError::EmptyTopic);
        }

        let topic_state = {
            let mut topics = self.topics.write().await;
            if !topics.contains_key(&topic_name) {
                if topics.len() >= self.config.max_topics {
                    return Err(PubSubError::TopicLimitReached(self.config.max_topics));
                }
                let (sender, _receiver) = broadcast::channel(self.config.channel_capacity);
                topics.insert(topic_name.clone(), TopicState::new(sender));
            }
            topics
                .get(&topic_name)
                .cloned()
                .expect("topic must exist after insertion")
        };

        topic_state.subscribers.fetch_add(1, Ordering::SeqCst);
        Ok(Subscription {
            topic: topic_name,
            receiver: topic_state.sender.subscribe(),
            subscribers: topic_state.subscribers.clone(),
        })
    }

    /// Publish a JSON payload to the given topic.
    ///
    /// Returns the number of subscribers that received the message.
    pub async fn publish(
        &self,
        topic: impl Into<String>,
        payload: serde_json::Value,
    ) -> Result<usize, PubSubError> {
        let topic_name = topic.into();
        if topic_name.trim().is_empty() {
            return Err(PubSubError::EmptyTopic);
        }

        let topic_state = {
            let mut topics = self.topics.write().await;
            if !topics.contains_key(&topic_name) {
                if topics.len() >= self.config.max_topics {
                    return Err(PubSubError::TopicLimitReached(self.config.max_topics));
                }
                let (sender, _receiver) = broadcast::channel(self.config.channel_capacity);
                topics.insert(topic_name.clone(), TopicState::new(sender));
            }
            topics
                .get(&topic_name)
                .cloned()
                .expect("topic must exist after insertion")
        };

        let message = PubSubMessage {
            topic: topic_name.clone(),
            payload,
            timestamp_ms: current_time_ms(),
        };

        match topic_state.sender.send(message) {
            Ok(receivers) => {
                if receivers == 0 {
                    topic_state.dropped.fetch_add(1, Ordering::SeqCst);
                    return Err(PubSubError::NoSubscribers(topic_name));
                }
                topic_state.published.fetch_add(1, Ordering::SeqCst);
                Ok(receivers)
            }
            Err(_) => {
                topic_state.dropped.fetch_add(1, Ordering::SeqCst);
                Err(PubSubError::ChannelClosed(topic_name))
            }
        }
    }

    /// Retrieve statistics for a specific topic.
    pub async fn topic_stats(&self, topic: &str) -> Option<TopicStats> {
        let topics = self.topics.read().await;
        topics.get(topic).map(|state| TopicStats {
            subscribers: state.subscribers.load(Ordering::SeqCst),
            published: state.published.load(Ordering::SeqCst),
            dropped: state.dropped.load(Ordering::SeqCst),
        })
    }

    /// Snapshot of all known topics and their stats.
    pub async fn all_stats(&self) -> HashMap<String, TopicStats> {
        let topics = self.topics.read().await;
        topics
            .iter()
            .map(|(topic, state)| {
                (
                    topic.clone(),
                    TopicStats {
                        subscribers: state.subscribers.load(Ordering::SeqCst),
                        published: state.published.load(Ordering::SeqCst),
                        dropped: state.dropped.load(Ordering::SeqCst),
                    },
                )
            })
            .collect()
    }
}

/// Stream of messages for a single subscriber.
pub struct Subscription {
    topic: String,
    receiver: broadcast::Receiver<PubSubMessage>,
    subscribers: Arc<AtomicUsize>,
}

impl Subscription {
    /// Get the topic this subscription listens to.
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// Receive the next message for this subscription.
    pub async fn next(&mut self) -> Result<PubSubMessage, PubSubError> {
        match self.receiver.recv().await {
            Ok(msg) => Ok(msg),
            Err(broadcast::error::RecvError::Closed) => {
                Err(PubSubError::ChannelClosed(self.topic.clone()))
            }
            Err(broadcast::error::RecvError::Lagged(_)) => {
                // Dropped messages because subscriber lagged; treat as dropped.
                Err(PubSubError::ChannelClosed(self.topic.clone()))
            }
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.subscribers.fetch_sub(1, Ordering::SeqCst);
    }
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
