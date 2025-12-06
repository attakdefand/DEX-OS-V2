//! Gossip protocol implementation for node discovery
//!
//! This module implements a gossip protocol for discovering and maintaining
//! a list of active nodes in the network.
//! Implements the Priority 3 feature from DEX-OS-V2.csv:
//! - Infrastructure,Network,Network,Gossip Protocol,Node Discovery,Medium

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time::sleep;

/// Gossip node configuration
#[derive(Debug, Clone)]
pub struct GossipConfig {
    /// Node ID
    pub node_id: String,
    /// Node address
    pub node_address: SocketAddr,
    /// List of initial peer addresses
    pub initial_peers: Vec<SocketAddr>,
    /// Gossip interval in milliseconds
    pub gossip_interval_ms: u64,
    /// Node timeout in milliseconds (nodes not seen for this long are considered dead)
    pub node_timeout_ms: u64,
}

impl Default for GossipConfig {
    fn default() -> Self {
        Self {
            node_id: "node-0".to_string(),
            node_address: "127.0.0.1:8000".parse().unwrap(),
            initial_peers: vec![],
            gossip_interval_ms: 1000,
            node_timeout_ms: 5000,
        }
    }
}

/// Information about a node in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Node ID
    pub id: String,
    /// Node address
    pub address: SocketAddr,
    /// Last time we heard from this node
    pub last_seen: u64,
}

/// Gossip message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    /// Ping message to check if node is alive
    Ping { node_id: String },
    /// Pong message in response to ping
    Pong { node_id: String },
    /// Gossip message containing node information
    Gossip { nodes: Vec<NodeInfo> },
}

/// Gossip node implementation
pub struct GossipNode {
    /// Node configuration
    config: GossipConfig,
    /// Known nodes in the network
    nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,
    /// Nodes that we know are alive
    alive_nodes: Arc<RwLock<HashSet<String>>>,
    /// Gossip interval
    gossip_interval: Duration,
    /// Node timeout
    node_timeout: Duration,
}

impl GossipNode {
    /// Create a new gossip node
    pub fn new(config: GossipConfig) -> Self {
        let nodes = Arc::new(RwLock::new(HashMap::new()));
        let alive_nodes = Arc::new(RwLock::new(HashSet::new()));
        let gossip_interval = Duration::from_millis(config.gossip_interval_ms);
        let node_timeout = Duration::from_millis(config.node_timeout_ms);

        Self {
            config,
            nodes,
            alive_nodes,
            gossip_interval,
            node_timeout,
        }
    }

    /// Start the gossip node
    pub async fn start(&self) {
        // Add ourselves to the node list
        let self_info = NodeInfo {
            id: self.config.node_id.clone(),
            address: self.config.node_address,
            last_seen: Self::current_time(),
        };

        {
            let mut nodes = self.nodes.write().await;
            nodes.insert(self.config.node_id.clone(), self_info);
        }

        // Add initial peers
        for peer_addr in &self.config.initial_peers {
            // In a real implementation, we would try to connect to these peers
            // and get their node IDs
            println!("Would connect to peer at {}", peer_addr);
        }

        // Start gossip loop
        loop {
            self.gossip().await;
            sleep(self.gossip_interval).await;
        }
    }

    /// Get current time in milliseconds since UNIX epoch
    fn current_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Perform gossip with other nodes
    async fn gossip(&self) {
        // Get a list of nodes to gossip with
        let peers = self.get_gossip_peers().await;

        if peers.is_empty() {
            return;
        }

        // Select a random peer to gossip with
        let peer = &peers[0]; // In a real implementation, we would select randomly

        // Send our node information to the peer
        self.send_gossip_message(peer, self.get_node_info().await)
            .await;

        // In a real implementation, we would also receive gossip messages
        // from other nodes and update our node list accordingly
    }

    /// Get list of peers to gossip with
    async fn get_gossip_peers(&self) -> Vec<NodeInfo> {
        let nodes = self.nodes.read().await;
        let alive_nodes = self.alive_nodes.read().await;

        nodes
            .values()
            .filter(|node| {
                node.id != self.config.node_id
                    && alive_nodes.contains(&node.id)
                    && Self::current_time() - node.last_seen < self.node_timeout.as_millis() as u64
            })
            .cloned()
            .collect()
    }

    /// Get our own node information
    async fn get_node_info(&self) -> Vec<NodeInfo> {
        let nodes = self.nodes.read().await;
        nodes.values().cloned().collect()
    }

    /// Send a gossip message to a peer
    async fn send_gossip_message(&self, peer: &NodeInfo, nodes: Vec<NodeInfo>) {
        println!(
            "Sending gossip message to node {} at {}",
            peer.id, peer.address
        );

        // In a real implementation, we would serialize the message and send it
        // over the network using TCP/UDP or another protocol
        let message = GossipMessage::Gossip { nodes };

        // Simulate network communication
        self.handle_gossip_message(message).await;
    }

    /// Handle incoming gossip message
    pub async fn handle_gossip_message(&self, message: GossipMessage) {
        match message {
            GossipMessage::Ping { node_id } => {
                println!("Received ping from node {}", node_id);
                // In a real implementation, we would send a pong response
            }
            GossipMessage::Pong { node_id } => {
                println!("Received pong from node {}", node_id);
                // Update last seen time for this node
                let mut nodes = self.nodes.write().await;
                if let Some(node) = nodes.get_mut(&node_id) {
                    node.last_seen = Self::current_time();
                }
            }
            GossipMessage::Gossip { nodes } => {
                println!("Received gossip message with {} nodes", nodes.len());
                self.update_node_list(nodes).await;
            }
        }
    }

    /// Update our node list with information from gossip message
    async fn update_node_list(&self, received_nodes: Vec<NodeInfo>) {
        let mut nodes = self.nodes.write().await;
        let mut alive_nodes = self.alive_nodes.write().await;

        for node in received_nodes {
            // Update or insert node information
            if let Some(existing_node) = nodes.get_mut(&node.id) {
                // Only update if this information is more recent
                if node.last_seen > existing_node.last_seen {
                    *existing_node = node.clone();
                }
            } else {
                // New node, add it to our list
                nodes.insert(node.id.clone(), node.clone());
            }

            // Mark node as alive
            alive_nodes.insert(node.id);
        }

        // Clean up dead nodes
        self.cleanup_dead_nodes(&mut nodes, &mut alive_nodes).await;
    }

    /// Clean up nodes that haven't been seen for a while
    async fn cleanup_dead_nodes(
        &self,
        nodes: &mut HashMap<String, NodeInfo>,
        alive_nodes: &mut HashSet<String>,
    ) {
        let now = Self::current_time();
        let timeout = self.node_timeout.as_millis() as u64;

        nodes.retain(|id, node| {
            if now - node.last_seen > timeout {
                alive_nodes.remove(id);
                false
            } else {
                true
            }
        });
    }

    /// Get list of all known nodes
    pub async fn get_known_nodes(&self) -> Vec<NodeInfo> {
        let nodes = self.nodes.read().await;
        nodes.values().cloned().collect()
    }

    /// Get list of alive nodes
    pub async fn get_alive_nodes(&self) -> Vec<NodeInfo> {
        let nodes = self.nodes.read().await;
        let alive_nodes = self.alive_nodes.read().await;

        nodes
            .values()
            .filter(|node| alive_nodes.contains(&node.id))
            .cloned()
            .collect()
    }

    /// Add a new node to our list
    pub async fn add_node(&self, node_info: NodeInfo) {
        let mut nodes = self.nodes.write().await;
        nodes.insert(node_info.id.clone(), node_info);
    }

    /// Mark a node as alive
    pub async fn mark_node_alive(&self, node_id: &str) {
        let mut alive_nodes = self.alive_nodes.write().await;
        alive_nodes.insert(node_id.to_string());
    }

    /// Mark a node as dead
    pub async fn mark_node_dead(&self, node_id: &str) {
        let mut alive_nodes = self.alive_nodes.write().await;
        alive_nodes.remove(node_id);
    }
}

/// Errors that can occur in gossip protocol
#[derive(Debug, Error)]
pub enum GossipError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Timeout error")]
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[test]
    fn test_gossip_node_creation() {
        let config = GossipConfig::default();
        let node = GossipNode::new(config);

        assert_eq!(node.config.node_id, "node-0");
    }

    #[tokio::test]
    async fn test_add_node() {
        let config = GossipConfig::default();
        let node = GossipNode::new(config);

        let node_info = NodeInfo {
            id: "test-node".to_string(),
            address: "127.0.0.1:8001".parse().unwrap(),
            last_seen: GossipNode::current_time(),
        };

        node.add_node(node_info).await;

        let known_nodes = node.get_known_nodes().await;
        assert_eq!(known_nodes.len(), 1);
        assert_eq!(known_nodes[0].id, "test-node");
    }

    #[tokio::test]
    async fn test_mark_node_alive_dead() {
        let config = GossipConfig::default();
        let node = GossipNode::new(config);

        let node_info = NodeInfo {
            id: "test-node".to_string(),
            address: "127.0.0.1:8001".parse().unwrap(),
            last_seen: GossipNode::current_time(),
        };

        node.add_node(node_info).await;
        node.mark_node_alive("test-node").await;

        let alive_nodes = node.get_alive_nodes().await;
        assert_eq!(alive_nodes.len(), 1);
        assert_eq!(alive_nodes[0].id, "test-node");

        node.mark_node_dead("test-node").await;

        let alive_nodes = node.get_alive_nodes().await;
        assert_eq!(alive_nodes.len(), 0);
    }
}
