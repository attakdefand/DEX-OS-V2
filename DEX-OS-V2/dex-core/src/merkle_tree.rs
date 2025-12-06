//! Merkle Tree implementation for DEX-OS
//!
//! This module provides a Merkle tree implementation for generating batch order proofs,
//! as required for the Priority 2 feature in the DEX-OS-V1.csv requirements.
//!
//! The implementation follows the standard Merkle tree structure where:
//! - Leaf nodes contain hashed order data
//! - Internal nodes contain the hash of their children
//! - The root hash serves as a commitment to the entire batch of orders

use sha2::{Digest, Sha256};

/// Represents a node in the Merkle tree
#[derive(Debug, Clone, PartialEq)]
pub struct MerkleNode {
    /// The hash value of this node
    pub hash: Vec<u8>,
    /// Left child node (None for leaf nodes)
    pub left: Option<Box<MerkleNode>>,
    /// Right child node (None for leaf nodes)
    pub right: Option<Box<MerkleNode>>,
}

impl MerkleNode {
    /// Create a new leaf node with the given data
    pub fn new_leaf(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize().to_vec();

        MerkleNode {
            hash,
            left: None,
            right: None,
        }
    }

    /// Create a new internal node with the given children
    pub fn new_internal(left: Box<MerkleNode>, right: Box<MerkleNode>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(&left.hash);
        hasher.update(&right.hash);
        let hash = hasher.finalize().to_vec();

        MerkleNode {
            hash,
            left: Some(left),
            right: Some(right),
        }
    }

    /// Create a new internal node with duplicated child (for odd number of nodes)
    pub fn new_internal_duplicate(child: Box<MerkleNode>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(&child.hash);
        hasher.update(&child.hash);
        let hash = hasher.finalize().to_vec();

        MerkleNode {
            hash,
            left: Some(child.clone()),
            right: Some(child),
        }
    }
}

/// Merkle Tree for batch order verification
#[derive(Debug, Clone)]
pub struct MerkleTree {
    /// Root node of the tree
    pub root: Option<MerkleNode>,
    /// Number of leaf nodes
    pub leaf_count: usize,
}

impl MerkleTree {
    /// Create a new empty Merkle tree
    pub fn new() -> Self {
        MerkleTree {
            root: None,
            leaf_count: 0,
        }
    }

    /// Build a Merkle tree from a vector of data
    pub fn from_data(data: &[Vec<u8>]) -> Self {
        if data.is_empty() {
            return MerkleTree::new();
        }

        let mut nodes: Vec<MerkleNode> = data.iter().map(|d| MerkleNode::new_leaf(d)).collect();

        while nodes.len() > 1 {
            let mut next_level = Vec::new();

            // Process pairs of nodes
            let mut i = 0;
            while i < nodes.len() {
                if i + 1 < nodes.len() {
                    // Pair of nodes
                    let left = Box::new(nodes[i].clone());
                    let right = Box::new(nodes[i + 1].clone());
                    next_level.push(MerkleNode::new_internal(left, right));
                    i += 2;
                } else {
                    // Single node, duplicate it
                    let node = Box::new(nodes[i].clone());
                    next_level.push(MerkleNode::new_internal_duplicate(node));
                    i += 1;
                }
            }

            nodes = next_level;
        }

        MerkleTree {
            root: Some(nodes[0].clone()),
            leaf_count: data.len(),
        }
    }

    /// Get the root hash of the tree
    pub fn root_hash(&self) -> Option<Vec<u8>> {
        self.root.as_ref().map(|node| node.hash.clone())
    }

    /// Generate a proof for a specific leaf index
    /// Returns true if a proof can be generated for the given index, false otherwise
    pub fn can_generate_proof(&self, index: usize) -> bool {
        if self.root.is_none() || index >= self.leaf_count {
            return false;
        }
        true
    }

    /// Generate a proof for a specific leaf index
    /// Note: This implementation requires the original data to generate proofs
    pub fn generate_proof_with_data(&self, data: &[Vec<u8>], index: usize) -> Option<Vec<Vec<u8>>> {
        if self.root.is_none() || index >= self.leaf_count || index >= data.len() {
            return None;
        }

        let mut proof = Vec::new();
        let mut current_level = self.build_leaf_nodes(data);
        let mut target_index = index;

        while current_level.len() > 1 {
            // For each level, we need to find the sibling of the target node
            let is_last_node_in_odd_level =
                current_level.len() % 2 == 1 && target_index == current_level.len() - 1;

            if is_last_node_in_odd_level {
                // This is the last node in an odd-length level, it's duplicated
                // Its "sibling" is itself, so we add its own hash to the proof
                proof.push(current_level[target_index].hash.clone());
            } else {
                // Normal case: find sibling
                let sibling_index = if target_index % 2 == 0 {
                    target_index + 1
                } else {
                    target_index - 1
                };

                if sibling_index < current_level.len() {
                    proof.push(current_level[sibling_index].hash.clone());
                }
            }

            // Build next level
            let mut next_level = Vec::new();
            let mut i = 0;
            while i < current_level.len() {
                if i + 1 < current_level.len() {
                    // Pair of nodes
                    let left = Box::new(current_level[i].clone());
                    let right = Box::new(current_level[i + 1].clone());
                    next_level.push(MerkleNode::new_internal(left, right));
                    i += 2;
                } else {
                    // Single node, duplicate it
                    let node = Box::new(current_level[i].clone());
                    next_level.push(MerkleNode::new_internal_duplicate(node));
                    i += 1;
                }
            }

            target_index /= 2;
            current_level = next_level;
        }

        Some(proof)
    }

    /// Verify a proof for a specific data item
    pub fn verify_proof(&self, data: &[u8], index: usize, proof: &[Vec<u8>]) -> bool {
        if index >= self.leaf_count {
            return false;
        }

        let mut hasher = Sha256::new();
        hasher.update(data);
        let mut current_hash = hasher.finalize().to_vec();

        let mut proof_index = index;
        for sibling_hash in proof {
            let mut hasher = Sha256::new();
            if proof_index % 2 == 0 {
                // Current node is left child
                hasher.update(&current_hash);
                hasher.update(sibling_hash);
            } else {
                // Current node is right child
                hasher.update(sibling_hash);
                hasher.update(&current_hash);
            }
            current_hash = hasher.finalize().to_vec();
            proof_index /= 2;
        }

        if let Some(root) = &self.root {
            current_hash == root.hash
        } else {
            false
        }
    }

    /// Build leaf nodes from the original data
    fn build_leaf_nodes(&self, data: &[Vec<u8>]) -> Vec<MerkleNode> {
        data.iter().map(|d| MerkleNode::new_leaf(d)).collect()
    }
}

/// Bridge Proof Verification using Merkle Tree
///
/// This implements the Priority 2 feature from DEX-OS-V1.csv:
/// "Core Trading,Bridge,Bridge,Merkle Tree,Proof Verification,High"
#[derive(Debug, Clone)]
pub struct BridgeProofVerifier {
    /// The Merkle tree containing the bridge transaction data
    tree: MerkleTree,
}

impl BridgeProofVerifier {
    /// Create a new bridge proof verifier with the given transaction data
    pub fn new(transaction_data: &[Vec<u8>]) -> Self {
        let tree = MerkleTree::from_data(transaction_data);
        Self { tree }
    }

    /// Create a new bridge proof verifier with an existing Merkle tree
    pub fn with_tree(tree: MerkleTree) -> Self {
        Self { tree }
    }

    /// Get the root hash of the Merkle tree
    pub fn root_hash(&self) -> Option<Vec<u8>> {
        self.tree.root_hash()
    }

    /// Generate a proof for a specific transaction
    pub fn generate_proof(
        &self,
        transaction_data: &[Vec<u8>],
        index: usize,
    ) -> Option<Vec<Vec<u8>>> {
        self.tree.generate_proof_with_data(transaction_data, index)
    }

    /// Verify a proof for a specific transaction
    pub fn verify_transaction(&self, transaction: &[u8], index: usize, proof: &[Vec<u8>]) -> bool {
        self.tree.verify_proof(transaction, index, proof)
    }

    /// Verify a batch of transactions
    pub fn verify_transaction_batch(
        &self,
        transactions: &[(usize, Vec<u8>, Vec<Vec<u8>>)],
    ) -> bool {
        transactions
            .iter()
            .all(|(index, transaction, proof)| self.verify_transaction(transaction, *index, proof))
    }

    /// Get the number of transactions in the tree
    pub fn transaction_count(&self) -> usize {
        self.tree.leaf_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tree() {
        let tree = MerkleTree::new();
        assert!(tree.root.is_none());
        assert_eq!(tree.leaf_count, 0);
        assert!(tree.root_hash().is_none());
    }

    #[test]
    fn test_single_leaf() {
        let data = vec![b"test".to_vec()];
        let tree = MerkleTree::from_data(&data);

        assert!(tree.root.is_some());
        assert_eq!(tree.leaf_count, 1);
        assert!(tree.root_hash().is_some());
    }

    #[test]
    fn test_multiple_leaves() {
        let data = vec![b"leaf1".to_vec(), b"leaf2".to_vec(), b"leaf3".to_vec()];
        let tree = MerkleTree::from_data(&data);

        assert!(tree.root.is_some());
        assert_eq!(tree.leaf_count, 3);
        assert!(tree.root_hash().is_some());
    }

    #[test]
    fn test_proof_generation() {
        let data = vec![b"leaf1".to_vec(), b"leaf2".to_vec(), b"leaf3".to_vec()];
        let tree = MerkleTree::from_data(&data);

        // Test proof generation possibility for valid indices
        assert!(tree.can_generate_proof(0));
        assert!(tree.can_generate_proof(1));
        assert!(tree.can_generate_proof(2));

        // Test proof generation possibility for invalid indices
        assert!(!tree.can_generate_proof(3));
        assert!(!tree.can_generate_proof(10));
    }

    #[test]
    fn test_bridge_proof_verifier() {
        // Create transaction data
        let transactions = vec![
            b"transaction1".to_vec(),
            b"transaction2".to_vec(),
            b"transaction3".to_vec(),
            b"transaction4".to_vec(),
        ];

        // Create verifier
        let verifier = BridgeProofVerifier::new(&transactions);

        assert!(verifier.root_hash().is_some());
        assert_eq!(verifier.transaction_count(), 4);

        // Generate proof for first transaction
        let proof = verifier.generate_proof(&transactions, 0).unwrap();
        assert!(!proof.is_empty());

        // Verify the proof
        assert!(verifier.verify_transaction(&transactions[0], 0, &proof));

        // Verify with incorrect data should fail
        assert!(!verifier.verify_transaction(b"wrong_data", 0, &proof));
    }

    #[test]
    fn test_batch_verification() {
        // Create transaction data
        let transactions = vec![
            b"transaction1".to_vec(),
            b"transaction2".to_vec(),
            b"transaction3".to_vec(),
        ];

        // Create verifier
        let verifier = BridgeProofVerifier::new(&transactions);

        // Generate proofs for all transactions
        let mut verification_data = Vec::new();
        for (i, transaction) in transactions.iter().enumerate() {
            let proof = verifier.generate_proof(&transactions, i).unwrap();
            verification_data.push((i, transaction.clone(), proof));
        }

        // Verify each transaction individually to debug
        for (i, transaction, proof) in &verification_data {
            let result = verifier.verify_transaction(transaction, *i, proof);
            assert!(result, "Verification failed for transaction at index {}", i);
        }

        // Verify batch
        assert!(verifier.verify_transaction_batch(&verification_data));
    }

    #[test]
    fn test_proof_verification_with_modified_data() {
        let transactions = vec![b"transaction1".to_vec(), b"transaction2".to_vec()];

        let verifier = BridgeProofVerifier::new(&transactions);
        let proof = verifier.generate_proof(&transactions, 0).unwrap();

        // Verification should pass with correct data
        assert!(verifier.verify_transaction(&transactions[0], 0, &proof));

        // Verification should fail with modified data
        assert!(!verifier.verify_transaction(b"modified_transaction".as_ref(), 0, &proof));
    }
}
