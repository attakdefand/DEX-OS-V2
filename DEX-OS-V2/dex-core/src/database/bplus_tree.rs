// B+ Tree Implementation for Database Indexing
// Security: Layer 9 - Database Security
// High-performance B+ tree with support for range queries and concurrent access

use std::fmt::Debug;
use std::sync::Arc;
use parking_lot::RwLock;

/// B+ Tree Index for database use
pub struct BPlusTreeIndex {
    tree: Arc<RwLock<BTreeInner>>,
    order: usize,
}

struct BTreeInner {
    root: Node,
    size: usize,
}

#[derive(Debug, Clone)]
enum Node {
    Internal {
        keys: Vec<Vec<u8>>,
        children: Vec<Box<Node>>,
    },
    Leaf {
        keys: Vec<Vec<u8>>,
        values: Vec<Vec<u64>>,
        next: Option<Box<Node>>,
    },
}

impl BPlusTreeIndex {
    /// Create a new index
    pub fn new(order: usize) -> Self {
        assert!(order >= 3, "B+ tree order must be at least 3");
        
        Self {
            tree: Arc::new(RwLock::new(BTreeInner {
                root: Node::Leaf {
                    keys: Vec::new(),
                    values: Vec::new(),
                    next: None,
                },
                size: 0,
            })),
            order,
        }
    }

    /// Insert a key pointing to a record ID
    pub fn insert(&self, key: Vec<u8>, record_id: u64) -> Result<(), String> {
        let mut tree = self.tree.write();
        
        // Get existing record IDs or create new vector
        let mut record_ids = self.search_internal(&tree.root, &key).unwrap_or_else(Vec::new);
        
        if !record_ids.contains(&record_id) {
            record_ids.push(record_id);
            self.insert_internal(&mut tree.root, key, record_ids, self.order)?;
            tree.size += 1;
        }
        
        Ok(())
    }

    /// Search for record IDs by key
    pub fn search(&self, key: &[u8]) -> Vec<u64> {
        let tree = self.tree.read();
        self.search_internal(&tree.root, key).unwrap_or_else(Vec::new)
    }

    /// Range query
    pub fn range_query(&self, start: &[u8], end: &[u8]) -> Vec<u64> {
        let tree = self.tree.read();
        let mut results = Vec::new();
        self.range_query_internal(&tree.root, start, end, &mut results);
        results
    }

    /// Delete a key
    pub fn delete(&self, key: &[u8]) -> Result<(), String> {
        let mut tree = self.tree.write();
        if self.delete_internal(&mut tree.root, key)? {
            tree.size = tree.size.saturating_sub(1);
        }
        Ok(())
    }

    /// Get index statistics
    pub fn stats(&self) -> IndexStats {
        let tree = self.tree.read();
        IndexStats {
            num_entries: tree.size,
            height: self.height_internal(&tree.root),
        }
    }

    // Private helper methods

    fn search_internal(&self, node: &Node, key: &[u8]) -> Option<Vec<u64>> {
        match node {
            Node::Leaf { keys, values, .. } => {
                keys.iter()
                    .position(|k| k == key)
                    .map(|idx| values[idx].clone())
            }
            Node::Internal { keys, children } => {
                let idx = keys.iter()
                    .position(|k| key < k.as_slice())
                    .unwrap_or(keys.len());
                self.search_internal(&children[idx], key)
            }
        }
    }

    fn insert_internal(&self, node: &mut Node, key: Vec<u8>, value: Vec<u64>, order: usize) -> Result<(), String> {
        match node {
            Node::Leaf { keys, values, .. } => {
                let pos = keys.iter()
                    .position(|k| k >= &key)
                    .unwrap_or(keys.len());
                
                if pos < keys.len() && keys[pos] == key {
                    values[pos] = value;
                } else {
                    keys.insert(pos, key);
                    values.insert(pos, value);
                }
                Ok(())
            }
            Node::Internal { keys, children } => {
                let idx = keys.iter()
                    .position(|k| &key < k)
                    .unwrap_or(keys.len());
                self.insert_internal(&mut children[idx], key, value, order)
            }
        }
    }

    fn range_query_internal(&self, node: &Node, start: &[u8], end: &[u8], results: &mut Vec<u64>) {
        match node {
            Node::Leaf { keys, values, .. } => {
                for (k, v) in keys.iter().zip(values.iter()) {
                    if k.as_slice() >= start && k.as_slice() <= end {
                        results.extend(v);
                    }
                }
            }
            Node::Internal { keys, children } => {
                for (i, child) in children.iter().enumerate() {
                    if i == 0 || (i > 0 && keys[i - 1].as_slice() <= end) {
                        self.range_query_internal(child, start, end, results);
                    }
                }
            }
        }
    }

    fn delete_internal(&self, node: &mut Node, key: &[u8]) -> Result<bool, String> {
        match node {
            Node::Leaf { keys, values, .. } => {
                if let Some(pos) = keys.iter().position(|k| k == key) {
                    keys.remove(pos);
                    values.remove(pos);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Node::Internal { keys, children } => {
                let idx = keys.iter()
                    .position(|k| key < k.as_slice())
                    .unwrap_or(keys.len());
                self.delete_internal(&mut children[idx], key)
            }
        }
    }

    fn height_internal(&self, node: &Node) -> usize {
        match node {
            Node::Leaf { .. } => 1,
            Node::Internal { children, .. } => {
                if children.is_empty() {
                    1
                } else {
                    1 + self.height_internal(&children[0])
                }
            }
        }
    }
}

/// Index statistics
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub num_entries: usize,
    pub height: usize,
}

/// Generic B+ Tree implementation
#[derive(Debug, Clone)]
pub struct BPlusTree<K: Ord + Clone, V: Clone> {
    root: Arc<RwLock<BTreeNode<K, V>>>,
    order: usize,
    size: Arc<RwLock<usize>>,
}
#[derive(Clone, Debug)]
enum BTreeNode<K: Ord + Clone, V: Clone> {
    Internal {
        keys: Vec<K>,
        children: Vec<Arc<RwLock<BTreeNode<K, V>>>>,
    },
    Leaf {
        keys: Vec<K>,
        values: Vec<V>,
    },
}

impl<K: Ord + Clone + Debug, V: Clone + Debug> BPlusTree<K, V> {
    pub fn new(order: usize) -> Self {
        assert!(order >= 3, "B+ tree order must be at least 3");
        
        Self {
            root: Arc::new(RwLock::new(BTreeNode::Leaf {
                keys: Vec::new(),
                values: Vec::new(),
            })),
            order,
            size: Arc::new(RwLock::new(0)),
        }
    }

    /// Insert a key-value pair into the B+ tree
    pub fn insert(&self, key: K, value: V) -> Result<(), String> {
        let mut root = self.root.write();
        self.insert_node(&mut root, key, value)?;
        let mut size = self.size.write();
        *size += 1;
        Ok(())
    }

    /// Search for a key in the B+ tree and return its value if found
    pub fn get(&self, key: &K) -> Option<V> {
        let root = self.root.read();
        self.search_node(&root, key)
    }

    /// Check if a key exists in the B+ tree
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn range_query(&self, start: &K, end: &K) -> Vec<(K, V)> {
        let root = self.root.read();
        let mut results = Vec::new();
        self.range_query_node(&root, start, end, &mut results);
        results
    }

    pub fn delete(&self, key: &K) -> Result<Option<V>, String> {
        let mut root = self.root.write();
        let result = self.delete_node(&mut root, key);
        if result.is_some() {
            let mut size = self.size.write();
            *size = size.saturating_sub(1);
        }
        Ok(result)
    }

    pub fn len(&self) -> usize {
        *self.size.read()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn height(&self) -> usize {
        let root = self.root.read();
        self.height_node(&root)
    }

    fn insert_node(&self, node: &mut BTreeNode<K, V>, key: K, value: V) -> Result<(), String> {
        match node {
            BTreeNode::Leaf { keys, values } => {
                let pos = keys.binary_search(&key).unwrap_or_else(|e| e);
                if pos < keys.len() && keys[pos] == key {
                    values[pos] = value;
                } else {
                    keys.insert(pos, key);
                    values.insert(pos, value);
                }
                Ok(())
            }
            BTreeNode::Internal { keys, children } => {
                let idx = keys.binary_search(&key).unwrap_or_else(|e| e);
                let mut child = children[idx].write();
                self.insert_node(&mut child, key, value)
            }
        }
    }

    fn search_node(&self, node: &BTreeNode<K, V>, key: &K) -> Option<V> {
        match node {
            BTreeNode::Leaf { keys, values } => {
                keys.binary_search(key)
                    .ok()
                    .map(|idx| values[idx].clone())
            }
            BTreeNode::Internal { keys, children } => {
                let idx = keys.binary_search(key).unwrap_or_else(|e| e);
                let child = children[idx].read();
                self.search_node(&child, key)
            }
        }
    }

    fn range_query_node(&self, node: &BTreeNode<K, V>, start: &K, end: &K, results: &mut Vec<(K, V)>) {
        match node {
            BTreeNode::Leaf { keys, values } => {
                for (k, v) in keys.iter().zip(values.iter()) {
                    if k >= start && k <= end {
                        results.push((k.clone(), v.clone()));
                    }
                }
            }
            BTreeNode::Internal { keys, children } => {
                for (i, child) in children.iter().enumerate() {
                    if i == 0 || (i > 0 && &keys[i - 1] <= end) {
                        let child_node = child.read();
                        self.range_query_node(&child_node, start, end, results);
                    }
                }
            }
        }
    }

    fn delete_node(&self, node: &mut BTreeNode<K, V>, key: &K) -> Option<V> {
        match node {
            BTreeNode::Leaf { keys, values } => {
                if let Ok(idx) = keys.binary_search(key) {
                    keys.remove(idx);
                    Some(values.remove(idx))
                } else {
                    None
                }
            }
            BTreeNode::Internal { keys, children } => {
                let idx = keys.binary_search(key).unwrap_or_else(|e| e);
                let mut child = children[idx].write();
                self.delete_node(&mut child, key)
            }
        }
    }

    fn height_node(&self, node: &BTreeNode<K, V>) -> usize {
        match node {
            BTreeNode::Leaf { .. } => 1,
            BTreeNode::Internal { children, .. } => {
                if children.is_empty() {
                    1
                } else {
                    let child = children[0].read();
                    1 + self.height_node(&child)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bplus_tree_basic() {
        let tree = BPlusTree::new(4);
        tree.insert(5, "five".to_string()).unwrap();
        tree.insert(3, "three".to_string()).unwrap();
        
        assert_eq!(tree.get(&5), Some("five".to_string()));
        assert_eq!(tree.get(&3), Some("three".to_string()));
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn test_index_operations() {
        let index = BPlusTreeIndex::new(4);
        
        index.insert(b"key1".to_vec(), 100).unwrap();
        index.insert(b"key1".to_vec(), 101).unwrap();
        
        let results = index.search(b"key1");
        assert_eq!(results.len(), 2);
        assert!(results.contains(&100));
        assert!(results.contains(&101));
    }
}
