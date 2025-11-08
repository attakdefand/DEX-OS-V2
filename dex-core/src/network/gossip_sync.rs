//! Gossip protocol implementation for off-chain sync
//!
//! This module implements the Priority 3 feature from DEX-OS-V2.csv:
//! - Security,Security,Security,Gossip Protocol,Off-chain Sync,Medium

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time::sleep;

/// Configuration for the gossip sync protocol
#[derive(Debug, Clone)]
pub struct GossipSyncConfig {
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
    /// Sync interval in milliseconds
    pub sync_interval_ms: u64,
}

impl Default for GossipSyncConfig {
    fn default() -> Self {
        Self {
            node_id: "node-0".to_string(),
            node_address: "127.0.0.1:8000".parse().unwrap(),
            initial_peers: vec![],
            gossip_interval_ms: 1000,
            node_timeout_ms: 5000,
            sync_interval_ms: 2000,
        }
    }
}

/// Information about data that needs to be synced
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncData {
    /// Unique identifier for the data
    pub id: String,
    /// Data payload
    pub payload: Vec<u8>,
    /// Timestamp when data was created/modified
    pub timestamp: u64,
    /// Origin node ID
    pub origin: String,
    /// Data type/category
    pub data_type: String,
}

/// Gossip sync message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipSyncMessage {
    /// Ping message to check if node is alive
    Ping { node_id: String },
    /// Pong message in response to ping
    Pong { node_id: String },
    /// Gossip message containing node information
    Gossip { nodes: Vec<NodeInfo> },
    /// Sync request for specific data
    SyncRequest { data_ids: Vec<String> },
    /// Sync response with requested data
    SyncResponse { data: Vec<SyncData> },
    /// Broadcast new data to all nodes
    DataBroadcast { data: SyncData },
    /// Request for latest data updates
    DataUpdateRequest,
    /// Response with latest data updates
    DataUpdateResponse { data: Vec<SyncData> },
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

/// Gossip sync node implementation
pub struct GossipSyncNode {
    /// Node configuration
    config: GossipSyncConfig,
    /// Known nodes in the network
    nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,
    /// Nodes that we know are alive
    alive_nodes: Arc<RwLock<HashSet<String>>>,
    /// Data that needs to be synced
    sync_data: Arc<RwLock<HashMap<String, SyncData>>>,
    /// Gossip interval
    gossip_interval: Duration,
    /// Node timeout
    node_timeout: Duration,
    /// Sync interval
    sync_interval: Duration,
}

impl GossipSyncNode {
    /// Create a new gossip sync node
    pub fn new(config: GossipSyncConfig) -> Self {
        let nodes = Arc::new(RwLock::new(HashMap::new()));
        let alive_nodes = Arc::new(RwLock::new(HashSet::new()));
        let sync_data = Arc::new(RwLock::new(HashMap::new()));
        let gossip_interval = Duration::from_millis(config.gossip_interval_ms);
        let node_timeout = Duration::from_millis(config.node_timeout_ms);
        let sync_interval = Duration::from_millis(config.sync_interval_ms);

        Self {
            config,
            nodes,
            alive_nodes,
            sync_data,
            gossip_interval,
            node_timeout,
            sync_interval,
        }
    }

    /// Start the gossip sync node
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

        // Start gossip and sync loops
        let gossip_node = self.clone();
        let sync_node = self.clone();

        tokio::spawn(async move {
            loop {
                gossip_node.gossip().await;
                sleep(gossip_node.gossip_interval).await;
            }
        });

        tokio::spawn(async move {
            loop {
                sync_node.sync_data_with_peers().await;
                sleep(sync_node.sync_interval).await;
            }
        });
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

    /// Sync data with other nodes
    async fn sync_data_with_peers(&self) {
        // Get a list of peers to sync with
        let peers = self.get_gossip_peers().await;

        if peers.is_empty() {
            return;
        }

        // For now, sync with the first peer
        // In a real implementation, we would sync with multiple peers
        let peer = &peers[0];
        
        // Request data updates from peer
        self.request_data_updates(peer).await;
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
        let message = GossipSyncMessage::Gossip { nodes };

        // Simulate network communication
        self.handle_gossip_message(message).await;
    }

    /// Request data updates from a peer
    async fn request_data_updates(&self, peer: &NodeInfo) {
        println!(
            "Requesting data updates from node {} at {}",
            peer.id, peer.address
        );

        // In a real implementation, we would send this request over the network
        let message = GossipSyncMessage::DataUpdateRequest;

        // Simulate network communication
        self.handle_gossip_message(message).await;
    }

    /// Handle incoming gossip message
    pub async fn handle_gossip_message(&self, message: GossipSyncMessage) {
        match message {
            GossipSyncMessage::Ping { node_id } => {
                println!("Received ping from node {}", node_id);
                // In a real implementation, we would send a pong response
            }
            GossipSyncMessage::Pong { node_id } => {
                println!("Received pong from node {}", node_id);
                // Update last seen time for this node
                let mut nodes = self.nodes.write().await;
                if let Some(node) = nodes.get_mut(&node_id) {
                    node.last_seen = Self::current_time();
                }
            }
            GossipSyncMessage::Gossip { nodes } => {
                println!("Received gossip message with {} nodes", nodes.len());
                self.update_node_list(nodes).await;
            }
            GossipSyncMessage::SyncRequest { data_ids } => {
                println!("Received sync request for {} data items", data_ids.len());
                self.handle_sync_request(data_ids).await;
            }
            GossipSyncMessage::SyncResponse { data } => {
                println!("Received sync response with {} data items", data.len());
                self.handle_sync_response(data).await;
            }
            GossipSyncMessage::DataBroadcast { data } => {
                println!("Received data broadcast for item {}", data.id);
                self.handle_data_broadcast(data).await;
            }
            GossipSyncMessage::DataUpdateRequest => {
                println!("Received data update request");
                self.handle_data_update_request().await;
            }
            GossipSyncMessage::DataUpdateResponse { data } => {
                println!("Received data update response with {} items", data.len());
                self.handle_data_update_response(data).await;
            }
        }
    }

    /// Handle sync request from another node
    async fn handle_sync_request(&self, data_ids: Vec<String>) {
        let sync_data = self.sync_data.read().await;
        let mut response_data = Vec::new();

        for id in data_ids {
            if let Some(data) = sync_data.get(&id) {
                response_data.push(data.clone());
            }
        }

        // In a real implementation, we would send this response back to the requesting node
        let response = GossipSyncMessage::SyncResponse {
            data: response_data,
        };
        
        // Simulate handling the response
        self.handle_gossip_message(response).await;
    }

    /// Handle sync response from another node
    async fn handle_sync_response(&self, data: Vec<SyncData>) {
        let mut sync_data = self.sync_data.write().await;
        
        for item in data {
            // Only update if we don't have this data or if the received data is newer
            if let Some(existing) = sync_data.get(&item.id) {
                if item.timestamp > existing.timestamp {
                    sync_data.insert(item.id.clone(), item);
                }
            } else {
                sync_data.insert(item.id.clone(), item);
            }
        }
    }

    /// Handle data broadcast from another node
    async fn handle_data_broadcast(&self, data: SyncData) {
        let mut sync_data = self.sync_data.write().await;
        
        // Only update if we don't have this data or if the received data is newer
        if let Some(existing) = sync_data.get(&data.id) {
            if data.timestamp > existing.timestamp {
                sync_data.insert(data.id.clone(), data);
            }
        } else {
            sync_data.insert(data.id.clone(), data);
        }
    }

    /// Handle data update request from another node
    async fn handle_data_update_request(&self) {
        let sync_data = self.sync_data.read().await;
        let data: Vec<SyncData> = sync_data.values().cloned().collect();
        
        // In a real implementation, we would send this response back to the requesting node
        let response = GossipSyncMessage::DataUpdateResponse { data };
        
        // Simulate handling the response
        self.handle_gossip_message(response).await;
    }

    /// Handle data update response from another node
    async fn handle_data_update_response(&self, data: Vec<SyncData>) {
        let mut sync_data = self.sync_data.write().await;
        
        for item in data {
            // Only update if we don't have this data or if the received data is newer
            if let Some(existing) = sync_data.get(&item.id) {
                if item.timestamp > existing.timestamp {
                    sync_data.insert(item.id.clone(), item);
                }
            } else {
                sync_data.insert(item.id.clone(), item);
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

    /// Add data to be synced
    pub async fn add_sync_data(&self, data: SyncData) {
        let mut sync_data = self.sync_data.write().await;
        sync_data.insert(data.id.clone(), data);
    }

    /// Get all sync data
    pub async fn get_sync_data(&self) -> HashMap<String, SyncData> {
        let sync_data = self.sync_data.read().await;
        sync_data.clone()
    }

    /// Get specific data by ID
    pub async fn get_data_by_id(&self, id: &str) -> Option<SyncData> {
        let sync_data = self.sync_data.read().await;
        sync_data.get(id).cloned()
    }

    /// Remove data by ID
    pub async fn remove_data(&self, id: &str) {
        let mut sync_data = self.sync_data.write().await;
        sync_data.remove(id);
    }
}

impl Clone for GossipSyncNode {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            nodes: Arc::clone(&self.nodes),
            alive_nodes: Arc::clone(&self.alive_nodes),
            sync_data: Arc::clone(&self.sync_data),
            gossip_interval: self.gossip_interval,
            node_timeout: self.node_timeout,
            sync_interval: self.sync_interval,
        }
    }
}

/// Errors that can occur in gossip sync protocol
#[derive(Debug, Error)]
pub enum GossipSyncError {
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
    fn test_gossip_sync_node_creation() {
        let config = GossipSyncConfig::default();
        let node = GossipSyncNode::new(config);

        assert_eq!(node.config.node_id, "node-0");
    }

    #[tokio::test]
    async fn test_add_sync_data() {
        let config = GossipSyncConfig::default();
        let node = GossipSyncNode::new(config);

        let sync_data = SyncData {
            id: "test-data".to_string(),
            payload: vec![1, 2, 3, 4],
            timestamp: GossipSyncNode::current_time(),
            origin: "test-node".to_string(),
            data_type: "test".to_string(),
        };

        node.add_sync_data(sync_data.clone()).await;

        let retrieved_data = node.get_data_by_id("test-data").await;
        assert!(retrieved_data.is_some());
        assert_eq!(retrieved_data.unwrap().id, "test-data");
    }

    #[tokio::test]
    async fn test_add_node() {
        let config = GossipSyncConfig::default();
        let node = GossipSyncNode::new(config);

        let node_info = NodeInfo {
            id: "test-node".to_string(),
            address: "127.0.0.1:8001".parse().unwrap(),
            last_seen: GossipSyncNode::current_time(),
        };

        node.add_node(node_info).await;

        let known_nodes = node.get_known_nodes().await;
        assert_eq!(known_nodes.len(), 1);
        assert_eq!(known_nodes[0].id, "test-node");
    }

    #[tokio::test]
    async fn test_mark_node_alive_dead() {
        let config = GossipSyncConfig::default();
        let node = GossipSyncNode::new(config);

        let node_info = NodeInfo {
            id: "test-node".to_string(),
            address: "127.0.0.1:8001".parse().unwrap(),
            last_seen: GossipSyncNode::current_time(),
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