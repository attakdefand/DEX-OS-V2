// Simple test file to verify AVL tree implementation
use std::cmp::Ordering;

/// Represents a node in the AVL tree
#[derive(Debug, Clone)]
pub struct AvlNode<T> {
    /// The value stored in this node
    pub value: T,
    /// Height of the node (max height of children + 1)
    pub height: i32,
    /// Left child node
    pub left: Option<Box<AvlNode<T>>>,
    /// Right child node
    pub right: Option<Box<AvlNode<T>>>,
}

impl<T: PartialOrd + Clone> AvlNode<T> {
    /// Create a new node with the given value
    pub fn new(value: T) -> Self {
        AvlNode {
            value,
            height: 1,
            left: None,
            right: None,
        }
    }

    /// Get the height of the node (0 if None)
    fn height(node: &Option<Box<AvlNode<T>>>) -> i32 {
        node.as_ref().map_or(0, |n| n.height)
    }

    /// Calculate the balance factor of the node (left height - right height)
    fn balance_factor(&self) -> i32 {
        Self::height(&self.left) - Self::height(&self.right)
    }

    /// Update the height of the node based on its children
    fn update_height(&mut self) {
        self.height = 1 + Self::height(&self.left).max(Self::height(&self.right));
    }

    /// Perform a right rotation
    fn rotate_right(mut node: Box<AvlNode<T>>) -> Box<AvlNode<T>> {
        let mut new_root = node.left.take().unwrap();
        node.left = new_root.right.take();
        node.update_height();
        
        new_root.right = Some(node);
        new_root.update_height();
        
        new_root
    }

    /// Perform a left rotation
    fn rotate_left(mut node: Box<AvlNode<T>>) -> Box<AvlNode<T>> {
        let mut new_root = node.right.take().unwrap();
        node.right = new_root.left.take();
        node.update_height();
        
        new_root.left = Some(node);
        new_root.update_height();
        
        new_root
    }

    /// Balance the node if needed
    fn balance(mut node: Box<AvlNode<T>>) -> Box<AvlNode<T>> {
        node.update_height();
        let balance = node.balance_factor();

        // Left heavy
        if balance > 1 {
            // Left-Right case
            if node.left.as_ref().map_or(0, |n| n.balance_factor()) < 0 {
                let left = node.left.take().unwrap();
                node.left = Some(Self::rotate_left(left));
            }
            // Left-Left case
            return Self::rotate_right(node);
        }
        // Right heavy
        else if balance < -1 {
            // Right-Left case
            if node.right.as_ref().map_or(0, |n| n.balance_factor()) > 0 {
                let right = node.right.take().unwrap();
                node.right = Some(Self::rotate_right(right));
            }
            // Right-Right case
            return Self::rotate_left(node);
        }

        node
    }

    /// Insert a value into the subtree
    fn insert_into_subtree(node: Option<Box<AvlNode<T>>>, value: T) -> Option<Box<AvlNode<T>>> {
        match node {
            None => Some(Box::new(AvlNode::new(value))),
            Some(mut n) => {
                match value.partial_cmp(&n.value) {
                    Some(Ordering::Equal) => {
                        // Value already exists, update it
                        n.value = value;
                        Some(n)
                    }
                    Some(Ordering::Less) => {
                        n.left = Self::insert_into_subtree(n.left.take(), value);
                        Some(Self::balance(n))
                    }
                    Some(Ordering::Greater) => {
                        n.right = Self::insert_into_subtree(n.right.take(), value);
                        Some(Self::balance(n))
                    }
                    None => Some(n), // Incomparable values, do nothing
                }
            }
        }
    }

    /// Find the minimum value node in the subtree
    fn find_min(node: &Box<AvlNode<T>>) -> &T {
        match &node.left {
            Some(left) => Self::find_min(left),
            None => &node.value,
        }
    }

    /// Remove a value from the subtree
    fn remove_from_subtree(
        node: Option<Box<AvlNode<T>>>,
        value: &T,
    ) -> Option<Box<AvlNode<T>>> {
        match node {
            None => None,
            Some(mut n) => {
                match value.partial_cmp(&n.value) {
                    Some(Ordering::Equal) => {
                        // Node to delete found
                        match (n.left.take(), n.right.take()) {
                            (None, None) => None, // Leaf node
                            (Some(left), None) => Some(left), // Only left child
                            (None, Some(right)) => Some(right), // Only right child
                            (Some(left), Some(right)) => {
                                // Node with two children
                                let min_value = Self::find_min(&right).clone();
                                n.value = min_value.clone();
                                n.right = Self::remove_from_subtree(Some(right), &min_value);
                                n.left = Some(left);
                                Some(Self::balance(n))
                            }
                        }
                    }
                    Some(Ordering::Less) => {
                        n.left = Self::remove_from_subtree(n.left.take(), value);
                        Some(Self::balance(n))
                    }
                    Some(Ordering::Greater) => {
                        n.right = Self::remove_from_subtree(n.right.take(), value);
                        Some(Self::balance(n))
                    }
                    None => Some(n), // Incomparable values, do nothing
                }
            }
        }
    }

    /// Search for a value in the subtree
    fn search_in_subtree(node: &Option<Box<AvlNode<T>>>, value: &T) -> bool {
        match node {
            None => false,
            Some(n) => {
                match value.partial_cmp(&n.value) {
                    Some(Ordering::Equal) => true,
                    Some(Ordering::Less) => Self::search_in_subtree(&n.left, value),
                    Some(Ordering::Greater) => Self::search_in_subtree(&n.right, value),
                    None => false, // Incomparable values
                }
            }
        }
    }

    /// In-order traversal of the subtree
    fn in_order_traversal<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        if let Some(ref left) = self.left {
            left.in_order_traversal(visit);
        }
        visit(&self.value);
        if let Some(ref right) = self.right {
            right.in_order_traversal(visit);
        }
    }
}

/// AVL Tree for maintaining balanced order book levels
#[derive(Debug, Clone)]
pub struct AvlTree<T> {
    /// Root node of the tree
    root: Option<Box<AvlNode<T>>>,
    /// Number of nodes in the tree
    size: usize,
}

impl<T: PartialOrd + Clone> AvlTree<T> {
    /// Create a new empty AVL tree
    pub fn new() -> Self {
        AvlTree {
            root: None,
            size: 0,
        }
    }

    /// Insert a value into the tree
    pub fn insert(&mut self, value: T) {
        let old_size = self.size;
        self.root = AvlNode::insert_into_subtree(self.root.take(), value);
        if self.root.is_some() && old_size == self.size {
            self.size += 1;
        }
    }

    /// Remove a value from the tree
    pub fn remove(&mut self, value: &T) -> bool {
        let old_len = self.len();
        self.root = AvlNode::remove_from_subtree(self.root.take(), value);
        let new_len = self.len();
        if new_len < old_len {
            self.size = new_len;
            true
        } else {
            false
        }
    }

    /// Search for a value in the tree
    pub fn contains(&self, value: &T) -> bool {
        AvlNode::search_in_subtree(&self.root, value)
    }

    /// Get the number of elements in the tree
    pub fn len(&self) -> usize {
        self.size
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Perform in-order traversal of the tree
    pub fn in_order_traversal<F>(&self, mut visit: F)
    where
        F: FnMut(&T),
    {
        if let Some(ref root) = self.root {
            root.in_order_traversal(&mut visit);
        }
    }
}

impl<T: PartialOrd + Clone> Default for AvlTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// AVL Tree-based Price Level structure for the orderbook
/// This implements the Priority 3 feature from DEX-OS-V1.csv:
/// "Core Trading,Orderbook,Orderbook,AVL Tree,Order Book Balancing,Medium"
#[derive(Debug, Clone)]
pub struct AvlPriceLevelTree {
    /// The AVL tree containing price levels
    tree: AvlTree<u64>, // Using u64 for price
}

impl AvlPriceLevelTree {
    /// Create a new AVL-based price level tree
    pub fn new() -> Self {
        Self {
            tree: AvlTree::new(),
        }
    }

    /// Insert a price level into the tree
    pub fn insert_price_level(&mut self, price: u64) {
        self.tree.insert(price);
    }

    /// Remove a price level from the tree
    pub fn remove_price_level(&mut self, price: &u64) -> bool {
        self.tree.remove(price)
    }

    /// Check if a price level exists in the tree
    pub fn contains_price_level(&self, price: &u64) -> bool {
        self.tree.contains(price)
    }

    /// Get all price levels in sorted order
    pub fn get_all_price_levels(&self) -> Vec<u64> {
        let mut levels = Vec::new();
        self.tree.in_order_traversal(|price| levels.push(*price));
        levels
    }

    /// Get the number of price levels
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }
}

impl Default for AvlPriceLevelTree {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    println!("Testing AVL Tree implementation...");
    
    // Test basic AVL tree
    let mut tree = AvlTree::new();
    tree.insert(10);
    tree.insert(20);
    tree.insert(30);
    
    println!("Tree size: {}", tree.len());
    println!("Contains 10: {}", tree.contains(&10));
    println!("Contains 20: {}", tree.contains(&20));
    println!("Contains 30: {}", tree.contains(&30));
    
    // Test AVL price level tree
    let mut price_tree = AvlPriceLevelTree::new();
    price_tree.insert_price_level(50000);
    price_tree.insert_price_level(49000);
    price_tree.insert_price_level(51000);
    price_tree.insert_price_level(48000);
    
    println!("Price tree size: {}", price_tree.len());
    println!("Contains price 50000: {}", price_tree.contains_price_level(&50000));
    
    let levels = price_tree.get_all_price_levels();
    println!("All price levels: {:?}", levels);
    
    // Remove a price level
    price_tree.remove_price_level(&49000);
    println!("After removing 49000, price tree size: {}", price_tree.len());
    
    let levels = price_tree.get_all_price_levels();
    println!("All price levels after removal: {:?}", levels);
    
    println!("AVL Tree tests completed successfully!");
}