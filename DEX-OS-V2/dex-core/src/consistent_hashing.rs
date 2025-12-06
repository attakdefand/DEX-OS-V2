//! Consistent hashing ring for distributed systems
//!
//! Implements the Priority 3 feature from DEX-OS-V2.csv:
//! - Distributed Systems,Distributed Systems,Distributed Systems,Consistent Hashing,Hash Ring,Medium

use sha3::{Digest, Sha3_256};
use std::collections::{BTreeMap, HashMap, HashSet};
use thiserror::Error;

/// Consistent hashing ring with virtual nodes
#[derive(Debug, Clone)]
pub struct HashRing {
    /// Number of virtual nodes (replicas) per physical node
    replicas: u32,
    /// Sorted ring mapping hash slots to node ids
    ring: BTreeMap<u64, String>,
    /// Set of active node ids
    nodes: HashSet<String>,
}

/// Errors produced by the hash ring
#[derive(Debug, Error, PartialEq)]
pub enum HashRingError {
    #[error("node already exists: {0}")]
    NodeExists(String),
    #[error("node not found: {0}")]
    NodeNotFound(String),
    #[error("no nodes in ring")]
    NoNodes,
}

impl HashRing {
    /// Create a new hash ring with the given replica count (minimum 1)
    pub fn new(replicas: u32) -> Self {
        let replica_count = replicas.max(1);
        Self {
            replicas: replica_count,
            ring: BTreeMap::new(),
            nodes: HashSet::new(),
        }
    }

    /// Add a new node to the ring
    pub fn add_node(&mut self, node_id: impl Into<String>) -> Result<(), HashRingError> {
        let id = node_id.into();
        if self.nodes.contains(&id) {
            return Err(HashRingError::NodeExists(id));
        }

        self.nodes.insert(id.clone());
        for replica in 0..self.replicas {
            let hash = self.hash_slot(&format!("{id}::{replica}"));
            self.ring.insert(hash, id.clone());
        }
        Ok(())
    }

    /// Remove a node from the ring
    pub fn remove_node(&mut self, node_id: &str) -> Result<(), HashRingError> {
        if !self.nodes.remove(node_id) {
            return Err(HashRingError::NodeNotFound(node_id.to_string()));
        }

        for replica in 0..self.replicas {
            let hash = self.hash_slot(&format!("{node_id}::{replica}"));
            self.ring.remove(&hash);
        }
        Ok(())
    }

    /// Get the node responsible for the given key
    pub fn get_node(&self, key: &str) -> Result<String, HashRingError> {
        if self.ring.is_empty() {
            return Err(HashRingError::NoNodes);
        }

        let hash = self.hash_slot(key);
        if let Some((_, node)) = self.ring.range(hash..).next() {
            return Ok(node.clone());
        }

        // Wrap around to the first entry
        let (_, node) = self.ring.iter().next().expect("ring not empty");
        Ok(node.clone())
    }

    /// Get up to `count` distinct nodes for the key (for replication)
    pub fn get_nodes(&self, key: &str, count: usize) -> Result<Vec<String>, HashRingError> {
        if count == 0 {
            return Ok(Vec::new());
        }
        if self.ring.is_empty() {
            return Err(HashRingError::NoNodes);
        }

        let mut result = Vec::new();
        let mut seen = HashSet::new();
        let hash = self.hash_slot(key);

        // Traverse the ring from the key's slot, wrapping around once
        for (_, node) in self
            .ring
            .range(hash..)
            .chain(self.ring.iter())
        {
            if seen.insert(node.clone()) {
                result.push(node.clone());
                if result.len() == count || result.len() == self.nodes.len() {
                    break;
                }
            }
        }

        Ok(result)
    }

    /// Current node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn hash_slot(&self, input: &str) -> u64 {
        let mut hasher = Sha3_256::new();
        hasher.update(input.as_bytes());
        let digest = hasher.finalize();
        u64::from_be_bytes([
            digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6],
            digest[7],
        ])
    }
}

impl Default for HashRing {
    fn default() -> Self {
        Self::new(128)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_and_looks_up_nodes() {
        let mut ring = HashRing::new(64);
        ring.add_node("node-a").unwrap();
        ring.add_node("node-b").unwrap();
        ring.add_node("node-c").unwrap();

        assert_eq!(ring.node_count(), 3);

        let node = ring.get_node("user-123").unwrap();
        assert!(["node-a", "node-b", "node-c"].contains(&node.as_str()));

        // Deterministic mapping for the same key
        let node_again = ring.get_node("user-123").unwrap();
        assert_eq!(node, node_again);
    }

    #[test]
    fn returns_distinct_nodes_for_replication() {
        let mut ring = HashRing::new(32);
        ring.add_node("n1").unwrap();
        ring.add_node("n2").unwrap();
        ring.add_node("n3").unwrap();

        let nodes = ring.get_nodes("order-42", 2).unwrap();
        assert_eq!(nodes.len(), 2);
        assert_ne!(nodes[0], nodes[1]);
    }

    #[test]
    fn redistributes_after_node_removal() {
        let mut ring = HashRing::new(64);
        ring.add_node("alpha").unwrap();
        ring.add_node("beta").unwrap();
        ring.add_node("gamma").unwrap();

        let key = "account-555";
        let initial = ring.get_node(key).unwrap();

        // Remove the mapped node and ensure we select a different remaining node
        ring.remove_node(&initial).unwrap();
        let reassigned = ring.get_node(key).unwrap();
        assert_ne!(initial, reassigned);
        assert!(["alpha", "beta", "gamma"].contains(&reassigned.as_str()));
    }

    #[test]
    fn provides_reasonable_distribution() {
        let mut ring = HashRing::new(128);
        ring.add_node("east").unwrap();
        ring.add_node("west").unwrap();
        ring.add_node("central").unwrap();

        let mut counts: HashMap<String, usize> = HashMap::new();
        for i in 0..300 {
            let key = format!("key-{i}");
            let node = ring.get_node(&key).unwrap();
            *counts.entry(node).or_insert(0) += 1;
        }

        assert_eq!(counts.len(), 3);
        let min = counts.values().min().copied().unwrap();
        let max = counts.values().max().copied().unwrap();
        // Allow up to ~20% spread across 300 samples
        assert!((max - min) <= 60, "distribution too skewed: {:?}", counts);
    }

    #[test]
    fn errors_when_ring_empty() {
        let ring = HashRing::new(16);
        assert_eq!(ring.get_node("k"), Err(HashRingError::NoNodes));
        assert_eq!(ring.get_nodes("k", 2), Err(HashRingError::NoNodes));
    }
}
