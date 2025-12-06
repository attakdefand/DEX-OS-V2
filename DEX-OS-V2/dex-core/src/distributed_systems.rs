//! Distributed Systems sharding with hash and range partitioning.
//!
//! Implements Priority 3 feature:
//! "Distributed Systems,Distributed Systems,Distributed Systems,Sharding,Hash/Range Partitioning,Medium"

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Errors that can occur during sharding operations.
#[derive(Error, Debug, PartialEq)]
pub enum ShardingError {
    /// Invalid shard count for hash partitioning.
    #[error("Invalid shard count: {0}")]
    InvalidShardCount(u64),
    /// Invalid partition configuration for range partitioning.
    #[error("Invalid range partition configuration: {0}")]
    InvalidRangeConfig(String),
    /// No range partition covers the provided key.
    #[error("No range covers key: {0}")]
    RangeNotFound(i64),
}

/// Sharding strategy using hash-based partitioning.
#[derive(Debug, Clone)]
pub struct HashPartitioner {
    /// Total number of shards in the hash ring.
    num_shards: u64,
}

impl HashPartitioner {
    /// Create a new hash partitioner.
    pub fn new(num_shards: u64) -> Result<Self, ShardingError> {
        if num_shards == 0 {
            return Err(ShardingError::InvalidShardCount(num_shards));
        }
        Ok(Self { num_shards })
    }

    /// Determine the shard for an arbitrary key by hashing and modding by shard count.
    pub fn shard_for_key(&self, key: &[u8]) -> u64 {
        let digest = Sha3_256::digest(key);
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&digest[..8]); // take first 8 bytes for a u64
        let hash = u64::from_be_bytes(buf);
        hash % self.num_shards
    }
}

/// A single continuous range assigned to a shard (inclusive start, inclusive end).
#[derive(Debug, Clone, PartialEq)]
pub struct RangeShard {
    /// Inclusive start of the range.
    pub start: i64,
    /// Inclusive end of the range.
    pub end: i64,
    /// Shard identifier for this range.
    pub shard_id: u64,
}

/// Sharding strategy using explicit range partitions.
#[derive(Debug, Clone)]
pub struct RangePartitioner {
    /// Sorted, non-overlapping ranges mapped to shards.
    ranges: Vec<RangeShard>,
    /// Quick lookup for ranges by start boundary to allow binary-search semantics.
    starts: BTreeMap<i64, RangeShard>,
}

impl RangePartitioner {
    /// Create a new range partitioner from a list of ranges.
    pub fn new(mut ranges: Vec<RangeShard>) -> Result<Self, ShardingError> {
        if ranges.is_empty() {
            return Err(ShardingError::InvalidRangeConfig(
                "at least one range is required".to_string(),
            ));
        }

        // Validate ranges are well-formed and non-overlapping when sorted.
        ranges.sort_by_key(|r| r.start);
        let mut starts = BTreeMap::new();
        let mut previous_end: Option<i64> = None;

        for range in ranges.into_iter() {
            if range.start > range.end {
                return Err(ShardingError::InvalidRangeConfig(format!(
                    "start {} greater than end {} for shard {}",
                    range.start, range.end, range.shard_id
                )));
            }
            if let Some(prev_end) = previous_end {
                if range.start <= prev_end {
                    return Err(ShardingError::InvalidRangeConfig(format!(
                        "overlapping or adjacent ranges detected at start {}",
                        range.start
                    )));
                }
            }
            previous_end = Some(range.end);
            starts.insert(range.start, range);
        }

        let ranges = starts.values().cloned().collect();
        Ok(Self { ranges, starts })
    }

    /// Determine the shard for a numeric key by locating the covering range.
    pub fn shard_for_key(&self, key: i64) -> Result<u64, ShardingError> {
        // Find the greatest start that is <= key, then check containment.
        let candidate = self.starts.range(..=key).next_back();
        match candidate {
            Some((_, range)) if key <= range.end => Ok(range.shard_id),
            _ => Err(ShardingError::RangeNotFound(key)),
        }
    }

    /// Get a copy of the configured ranges (useful for observability/testing).
    pub fn ranges(&self) -> &[RangeShard] {
        &self.ranges
    }
}

/// Manager that exposes both hash-based and range-based sharding in one place.
#[derive(Debug, Clone)]
pub struct ShardingManager {
    hash_partitioner: HashPartitioner,
    range_partitioner: RangePartitioner,
}

impl ShardingManager {
    /// Create a manager from both strategies.
    pub fn new(
        hash_partitioner: HashPartitioner,
        range_partitioner: RangePartitioner,
    ) -> Self {
        Self {
            hash_partitioner,
            range_partitioner,
        }
    }

    /// Resolve a shard using hash partitioning.
    pub fn shard_for_hash_key(&self, key: &[u8]) -> u64 {
        self.hash_partitioner.shard_for_key(key)
    }

    /// Resolve a shard using range partitioning.
    pub fn shard_for_range_key(&self, key: i64) -> Result<u64, ShardingError> {
        self.range_partitioner.shard_for_key(key)
    }
}

/// Gossip protocol for node discovery across distributed nodes.
///
/// Implements Priority 3 feature:
/// "Distributed Systems,Distributed Systems,Distributed Systems,Gossip Protocol,Node Discovery,Medium"
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
    /// Node is reachable and actively responding.
    Alive,
    /// Node has not been seen recently and should be probed.
    Suspect,
    /// Node is considered offline/unreachable.
    Dead,
}

/// Node metadata shared through gossip.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeState {
    /// Node identifier.
    pub id: String,
    /// Advertised network address.
    pub address: SocketAddr,
    /// Monotonic heartbeat counter from the node.
    pub heartbeat: u64,
    /// Last time this node was seen (ms since epoch).
    pub last_seen_ms: u64,
    /// Current liveness classification.
    pub status: NodeStatus,
}

/// Seed peer used to bootstrap the membership list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedPeer {
    /// Node identifier.
    pub id: String,
    /// Peer address.
    pub address: SocketAddr,
}

/// Gossip payload exchanged between nodes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GossipPayload {
    /// Sender identifier.
    pub from: String,
    /// Sender's latest heartbeat.
    pub heartbeat: u64,
    /// Timestamp when the payload was generated.
    pub generated_at_ms: u64,
    /// Known membership information.
    pub nodes: Vec<NodeState>,
}

/// Abstract time provider to allow deterministic testing.
pub trait TimeProvider: Send + Sync {
    /// Current time in milliseconds since UNIX epoch.
    fn now_ms(&self) -> u64;
}

/// Real clock implementation.
#[derive(Debug, Clone)]
pub struct SystemClock;

impl TimeProvider for SystemClock {
    fn now_ms(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

/// Configuration for gossip-driven node discovery.
#[derive(Debug, Clone)]
pub struct NodeDiscoveryConfig {
    /// Local node identifier.
    pub node_id: String,
    /// Local node address.
    pub node_address: SocketAddr,
    /// Seed peers used for initial gossip.
    pub initial_peers: Vec<SeedPeer>,
    /// Threshold before a node transitions from alive to suspect.
    pub suspect_timeout_ms: u64,
    /// Threshold before a node transitions from suspect to dead.
    pub dead_timeout_ms: u64,
    /// How many peers to target during a gossip round.
    pub gossip_fanout: usize,
}

impl Default for NodeDiscoveryConfig {
    fn default() -> Self {
        Self {
            node_id: "node-0".to_string(),
            node_address: "127.0.0.1:9000".parse().unwrap(),
            initial_peers: Vec::new(),
            suspect_timeout_ms: 3_000,
            dead_timeout_ms: 9_000,
            gossip_fanout: 3,
        }
    }
}

impl NodeDiscoveryConfig {
    fn normalized(mut self) -> Self {
        if self.dead_timeout_ms <= self.suspect_timeout_ms {
            self.dead_timeout_ms = self.suspect_timeout_ms + 1;
        }
        if self.gossip_fanout == 0 {
            self.gossip_fanout = 1;
        }
        self
    }
}

/// Gossip-driven node discovery and health tracking.
pub struct NodeDiscovery {
    config: NodeDiscoveryConfig,
    members: RwLock<HashMap<String, NodeState>>,
    time: Arc<dyn TimeProvider>,
    local_heartbeat: AtomicU64,
}

impl NodeDiscovery {
    /// Create a new node discovery instance using the system clock.
    pub fn new(config: NodeDiscoveryConfig) -> Self {
        Self::with_time_provider(config, Arc::new(SystemClock))
    }

    /// Create a new node discovery instance with a custom time provider.
    pub fn with_time_provider(
        config: NodeDiscoveryConfig,
        time: Arc<dyn TimeProvider>,
    ) -> Self {
        let config = config.normalized();
        let now = time.now_ms();
        let mut members = HashMap::new();

        members.insert(
            config.node_id.clone(),
            NodeState {
                id: config.node_id.clone(),
                address: config.node_address,
                heartbeat: 0,
                last_seen_ms: now,
                status: NodeStatus::Alive,
            },
        );

        for peer in &config.initial_peers {
            members.entry(peer.id.clone()).or_insert(NodeState {
                id: peer.id.clone(),
                address: peer.address,
                heartbeat: 0,
                last_seen_ms: now,
                status: NodeStatus::Suspect,
            });
        }

        Self {
            config,
            members: RwLock::new(members),
            time,
            local_heartbeat: AtomicU64::new(0),
        }
    }

    /// Increment the local heartbeat and refresh our own record.
    pub fn heartbeat(&self) {
        let now = self.time.now_ms();
        let hb = self.local_heartbeat.fetch_add(1, Ordering::SeqCst) + 1;
        if let Some(self_node) = self.members.write().unwrap().get_mut(&self.config.node_id) {
            self_node.heartbeat = hb;
            self_node.last_seen_ms = now;
            self_node.status = NodeStatus::Alive;
        }
    }

    /// Record a direct observation of a peer (e.g., after a successful ping).
    pub fn record_observation(
        &self,
        node_id: impl Into<String>,
        address: SocketAddr,
        heartbeat: u64,
    ) {
        let now = self.time.now_ms();
        let mut members = self.members.write().unwrap();
        let node_id = node_id.into();
        let entry = members.entry(node_id.clone()).or_insert(NodeState {
            id: node_id.clone(),
            address,
            heartbeat,
            last_seen_ms: now,
            status: NodeStatus::Alive,
        });

        if heartbeat >= entry.heartbeat {
            entry.address = address;
            entry.heartbeat = heartbeat;
            entry.last_seen_ms = now;
            entry.status = NodeStatus::Alive;
        }
    }

    /// Build a gossip payload with the current membership view.
    pub fn build_gossip_payload(&self) -> GossipPayload {
        self.run_health_check();
        let now = self.time.now_ms();
        let nodes = {
            let members = self.members.read().unwrap();
            members.values().cloned().collect()
        };

        GossipPayload {
            from: self.config.node_id.clone(),
            heartbeat: self.local_heartbeat.load(Ordering::SeqCst),
            generated_at_ms: now,
            nodes,
        }
    }

    /// Merge incoming gossip payload into the local membership view.
    pub fn merge_gossip(&self, payload: GossipPayload) {
        let now = self.time.now_ms();
        let mut members = self.members.write().unwrap();

        for node in payload.nodes {
            self.update_member(now, &mut members, node);
        }

        // Ensure the sender is tracked even if not present in the node list.
        let sender_address = members
            .get(&payload.from)
            .map(|node| node.address)
            .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 0)));
        let sender = NodeState {
            id: payload.from.clone(),
            address: sender_address,
            heartbeat: payload.heartbeat,
            last_seen_ms: now,
            status: NodeStatus::Alive,
        };
        self.update_member(now, &mut members, sender);
        drop(members);

        self.run_health_check();
    }

    /// Run liveness evaluation to transition nodes between alive/suspect/dead.
    pub fn run_health_check(&self) {
        let now = self.time.now_ms();
        let mut members = self.members.write().unwrap();
        for node in members.values_mut() {
            if node.id == self.config.node_id {
                continue;
            }
            let elapsed = now.saturating_sub(node.last_seen_ms);
            if elapsed >= self.config.dead_timeout_ms {
                node.status = NodeStatus::Dead;
            } else if elapsed >= self.config.suspect_timeout_ms {
                if node.status == NodeStatus::Alive {
                    node.status = NodeStatus::Suspect;
                }
            } else {
                node.status = NodeStatus::Alive;
            }
        }
    }

    /// Remove nodes that have been dead beyond the configured timeout.
    pub fn prune_dead_nodes(&self) {
        let now = self.time.now_ms();
        let mut members = self.members.write().unwrap();
        members.retain(|id, node| {
            if id == &self.config.node_id {
                return true;
            }
            now.saturating_sub(node.last_seen_ms) <= self.config.dead_timeout_ms
        });
    }

    /// Select peers for the next gossip round, limited by gossip_fanout.
    pub fn select_gossip_peers(&self) -> Vec<NodeState> {
        self.run_health_check();
        let mut peers: Vec<NodeState> = self
            .members
            .read()
            .unwrap()
            .values()
            .filter(|node| node.id != self.config.node_id && node.status != NodeStatus::Dead)
            .cloned()
            .collect();
        peers.sort_by(|a, b| a.id.cmp(&b.id));
        peers.truncate(self.config.gossip_fanout);
        peers
    }

    /// Get the current membership snapshot sorted by node id.
    pub fn membership(&self) -> Vec<NodeState> {
        let mut view: Vec<NodeState> = self.members.read().unwrap().values().cloned().collect();
        view.sort_by(|a, b| a.id.cmp(&b.id));
        view
    }

    fn update_member(
        &self,
        now: u64,
        members: &mut HashMap<String, NodeState>,
        mut incoming: NodeState,
    ) {
        // Never allow remote state to override our self record.
        if incoming.id == self.config.node_id {
            if let Some(self_node) = members.get_mut(&self.config.node_id) {
                self_node.last_seen_ms = now;
                self_node.heartbeat = self_node
                    .heartbeat
                    .max(self.local_heartbeat.load(Ordering::SeqCst));
                self_node.status = NodeStatus::Alive;
            }
            return;
        }

        match members.entry(incoming.id.clone()) {
            Entry::Occupied(mut slot) => {
                let entry = slot.get_mut();
                let fresher = incoming.heartbeat > entry.heartbeat
                    || incoming.last_seen_ms > entry.last_seen_ms
                    || entry.status == NodeStatus::Dead;

                if fresher {
                    entry.address = incoming.address;
                    entry.heartbeat = incoming.heartbeat;
                    entry.last_seen_ms = now;
                    entry.status = if incoming.status == NodeStatus::Dead {
                        NodeStatus::Dead
                    } else {
                        NodeStatus::Alive
                    };
                }
            }
            Entry::Vacant(slot) => {
                let status = if incoming.status == NodeStatus::Dead {
                    NodeStatus::Suspect
                } else {
                    NodeStatus::Alive
                };
                slot.insert(NodeState {
                    id: incoming.id,
                    address: incoming.address,
                    heartbeat: incoming.heartbeat,
                    last_seen_ms: now,
                    status,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    #[test]
    fn hash_partitioning_is_deterministic_and_within_bounds() {
        let partitioner = HashPartitioner::new(8).unwrap();
        let shard_a = partitioner.shard_for_key(b"user-123");
        let shard_b = partitioner.shard_for_key(b"user-123");
        let shard_c = partitioner.shard_for_key(b"another-user");

        assert_eq!(shard_a, shard_b);
        assert!(shard_a < 8);
        assert!(shard_c < 8);
        assert_ne!(shard_a, shard_c); // high likelihood they differ with different keys
    }

    #[test]
    fn hash_partitioning_rejects_invalid_shard_count() {
        assert!(matches!(
            HashPartitioner::new(0),
            Err(ShardingError::InvalidShardCount(0))
        ));
    }

    #[test]
    fn range_partitioning_resolves_correct_shards() {
        let ranges = vec![
            RangeShard {
                start: 0,
                end: 99,
                shard_id: 0,
            },
            RangeShard {
                start: 100,
                end: 199,
                shard_id: 1,
            },
            RangeShard {
                start: 200,
                end: 10_000,
                shard_id: 2,
            },
        ];
        let partitioner = RangePartitioner::new(ranges).unwrap();

        assert_eq!(partitioner.shard_for_key(0).unwrap(), 0);
        assert_eq!(partitioner.shard_for_key(50).unwrap(), 0);
        assert_eq!(partitioner.shard_for_key(99).unwrap(), 0);
        assert_eq!(partitioner.shard_for_key(100).unwrap(), 1);
        assert_eq!(partitioner.shard_for_key(150).unwrap(), 1);
        assert_eq!(partitioner.shard_for_key(199).unwrap(), 1);
        assert_eq!(partitioner.shard_for_key(200).unwrap(), 2);
        assert_eq!(partitioner.shard_for_key(9_999).unwrap(), 2);
    }

    #[test]
    fn range_partitioning_errors_for_missing_range() {
        let ranges = vec![RangeShard {
            start: 10,
            end: 20,
            shard_id: 0,
        }];
        let partitioner = RangePartitioner::new(ranges).unwrap();
        assert_eq!(
            partitioner.shard_for_key(9),
            Err(ShardingError::RangeNotFound(9))
        );
        assert_eq!(
            partitioner.shard_for_key(21),
            Err(ShardingError::RangeNotFound(21))
        );
    }

    #[test]
    fn range_partitioning_rejects_overlapping_ranges() {
        let ranges = vec![
            RangeShard {
                start: 0,
                end: 50,
                shard_id: 0,
            },
            RangeShard {
                start: 50,
                end: 100,
                shard_id: 1,
            },
        ];
        let err = RangePartitioner::new(ranges).unwrap_err();
        assert!(matches!(err, ShardingError::InvalidRangeConfig(_)));
    }

    #[test]
    fn manager_supports_both_hash_and_range_partitioning() {
        let hash_partitioner = HashPartitioner::new(4).unwrap();
        let range_partitioner = RangePartitioner::new(vec![
            RangeShard {
                start: 0,
                end: 49,
                shard_id: 0,
            },
            RangeShard {
                start: 50,
                end: 99,
                shard_id: 1,
            },
        ])
        .unwrap();
        let manager = ShardingManager::new(hash_partitioner, range_partitioner);

        let hash_shard = manager.shard_for_hash_key(b"orders:12345");
        assert!(hash_shard < 4);

        let range_shard = manager.shard_for_range_key(75).unwrap();
        assert_eq!(range_shard, 1);
    }

    #[derive(Clone)]
    struct ManualClock {
        now: Arc<AtomicU64>,
    }

    impl ManualClock {
        fn new(start_ms: u64) -> Self {
            Self {
                now: Arc::new(AtomicU64::new(start_ms)),
            }
        }

        fn advance(&self, delta_ms: u64) {
            self.now.fetch_add(delta_ms, Ordering::SeqCst);
        }
    }

    impl TimeProvider for ManualClock {
        fn now_ms(&self) -> u64 {
            self.now.load(Ordering::SeqCst)
        }
    }

    fn addr(port: u16) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], port))
    }

    #[test]
    fn gossip_propagates_membership_between_peers() {
        let clock = Arc::new(ManualClock::new(0));

        let node_a = NodeDiscovery::with_time_provider(
            NodeDiscoveryConfig {
                node_id: "node-a".to_string(),
                node_address: addr(7000),
                initial_peers: Vec::new(),
                suspect_timeout_ms: 2_000,
                dead_timeout_ms: 5_000,
                gossip_fanout: 2,
            },
            clock.clone(),
        );

        let node_b = NodeDiscovery::with_time_provider(
            NodeDiscoveryConfig {
                node_id: "node-b".to_string(),
                node_address: addr(7001),
                initial_peers: vec![SeedPeer {
                    id: "node-a".to_string(),
                    address: addr(7000),
                }],
                suspect_timeout_ms: 2_000,
                dead_timeout_ms: 5_000,
                gossip_fanout: 2,
            },
            clock.clone(),
        );

        node_a.heartbeat();
        node_b.merge_gossip(node_a.build_gossip_payload());
        let b_view = node_b.membership();
        let a_seen = b_view.iter().find(|n| n.id == "node-a").unwrap();
        assert_eq!(a_seen.status, NodeStatus::Alive);

        node_b.heartbeat();
        node_a.merge_gossip(node_b.build_gossip_payload());
        let a_view = node_a.membership();
        let b_seen = a_view.iter().find(|n| n.id == "node-b").unwrap();
        assert_eq!(b_seen.status, NodeStatus::Alive);
        assert_eq!(b_seen.address, addr(7001));
    }

    #[test]
    fn gossip_marks_nodes_suspect_and_dead_after_timeouts() {
        let clock = Arc::new(ManualClock::new(0));
        let suspect_timeout = 2_000;
        let dead_timeout = 5_000;
        let discovery = NodeDiscovery::with_time_provider(
            NodeDiscoveryConfig {
                node_id: "observer".to_string(),
                node_address: addr(8000),
                initial_peers: Vec::new(),
                suspect_timeout_ms: suspect_timeout,
                dead_timeout_ms: dead_timeout,
                gossip_fanout: 3,
            },
            clock.clone(),
        );

        discovery.record_observation("peer-1", addr(8001), 1);
        discovery.run_health_check();
        let mut membership = discovery.membership();
        assert_eq!(
            membership.iter().find(|n| n.id == "peer-1").unwrap().status,
            NodeStatus::Alive
        );

        clock.advance(suspect_timeout + 1);
        discovery.run_health_check();
        membership = discovery.membership();
        assert_eq!(
            membership.iter().find(|n| n.id == "peer-1").unwrap().status,
            NodeStatus::Suspect
        );

        clock.advance(dead_timeout);
        discovery.run_health_check();
        membership = discovery.membership();
        assert_eq!(
            membership.iter().find(|n| n.id == "peer-1").unwrap().status,
            NodeStatus::Dead
        );

        discovery.prune_dead_nodes();
        assert!(!discovery
            .membership()
            .iter()
            .any(|n| n.id == "peer-1"));
    }

    #[test]
    fn gossip_prefers_newer_heartbeats_and_discards_stale_updates() {
        let clock = Arc::new(ManualClock::new(0));
        let discovery = NodeDiscovery::with_time_provider(
            NodeDiscoveryConfig {
                node_id: "observer".to_string(),
                node_address: addr(8100),
                initial_peers: Vec::new(),
                suspect_timeout_ms: 2_000,
                dead_timeout_ms: 5_000,
                gossip_fanout: 3,
            },
            clock.clone(),
        );

        discovery.record_observation("peer", addr(8101), 5);
        discovery.run_health_check();

        // Stale gossip with older heartbeat should be ignored.
        discovery.merge_gossip(GossipPayload {
            from: "other".to_string(),
            heartbeat: 0,
            generated_at_ms: clock.now_ms(),
            nodes: vec![NodeState {
                id: "peer".to_string(),
                address: addr(9000),
                heartbeat: 1,
                last_seen_ms: 0,
                status: NodeStatus::Alive,
            }],
        });

        let entry = discovery
            .membership()
            .into_iter()
            .find(|n| n.id == "peer")
            .unwrap();
        assert_eq!(entry.heartbeat, 5);
        assert_eq!(entry.address, addr(8101));

        // Fresher heartbeat updates the view.
        discovery.merge_gossip(GossipPayload {
            from: "other".to_string(),
            heartbeat: 0,
            generated_at_ms: clock.now_ms(),
            nodes: vec![NodeState {
                id: "peer".to_string(),
                address: addr(8102),
                heartbeat: 6,
                last_seen_ms: 0,
                status: NodeStatus::Alive,
            }],
        });

        let updated = discovery
            .membership()
            .into_iter()
            .find(|n| n.id == "peer")
            .unwrap();
        assert_eq!(updated.heartbeat, 6);
        assert_eq!(updated.address, addr(8102));
    }

    #[test]
    fn gossip_peer_selection_filters_dead_and_limits_fanout() {
        let clock = Arc::new(ManualClock::new(0));
        let suspect_timeout = 1_000;
        let dead_timeout = 3_000;
        let discovery = NodeDiscovery::with_time_provider(
            NodeDiscoveryConfig {
                node_id: "observer".to_string(),
                node_address: addr(8200),
                initial_peers: Vec::new(),
                suspect_timeout_ms: suspect_timeout,
                dead_timeout_ms: dead_timeout,
                gossip_fanout: 1,
            },
            clock.clone(),
        );

        discovery.record_observation("alive-peer", addr(8201), 1);
        discovery.record_observation("fading-peer", addr(8202), 1);

        // Keep alive-peer fresh right before health check and allow fading-peer to expire.
        clock.advance(dead_timeout + 100);
        discovery.record_observation("alive-peer", addr(8201), 2);
        discovery.run_health_check();

        let peers = discovery.select_gossip_peers();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].id, "alive-peer");
        assert_ne!(peers[0].status, NodeStatus::Dead);
    }
}
