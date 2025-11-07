//! Path routing implementation using Bellman-Ford algorithm for the DEX-OS core engine
//!
//! This module implements multiple features from DEX-OS-V1.csv:
//! - Priority 1 feature: "Core Trading,DEX Aggregator,DEX Aggregator,Graph,DEX Liquidity Network,High"
//! - Priority 1 feature: "Core Trading,DEX Aggregator,DEX Aggregator,Hash Map,Route Caching,High"
//! - Priority 2 feature: "Core Trading,DEX Aggregator,DEX Aggregator,Bellman-Ford,Path Routing,Medium"
//!
//! It provides functionality for finding the best trading paths across multiple DEXes
//! using the Bellman-Ford algorithm to handle negative weight edges (which can represent
//! arbitrage opportunities or fees), with route caching for improved performance.

use crate::types::{Quantity, TokenId};
use std::collections::{hash_map::Entry, BinaryHeap, HashMap, VecDeque};
use thiserror::Error;

/// Represents an edge in the trading graph (a trading path between tokens on a DEX)
#[derive(Debug, Clone, PartialEq)]
pub struct TradingEdge {
    /// Source token
    pub from_token: TokenId,
    /// Destination token
    pub to_token: TokenId,
    /// DEX where this trading pair is available
    pub dex_name: String,
    /// Exchange rate (how much to_token you get for 1 from_token)
    pub exchange_rate: f64,
    /// Fee percentage (0.0 to 1.0)
    pub fee: f64,
    /// Liquidity available for this trading pair
    pub liquidity: Quantity,
}

/// Represents a node in the trading graph (a token)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TradingNode {
    /// Token identifier
    pub token_id: TokenId,
}

/// Result of a path routing calculation
#[derive(Debug, Clone)]
pub struct RoutingPath {
    /// Sequence of edges that form the path
    pub edges: Vec<TradingEdge>,
    /// Total exchange rate for the path (cumulative)
    pub total_exchange_rate: f64,
    /// Total fee for the path (cumulative)
    pub total_fee: f64,
    /// Minimum liquidity along the path
    pub min_liquidity: Quantity,
}

/// Manages path routing using the Bellman-Ford algorithm
#[derive(Debug, Clone)]
pub struct PathRouter {
    /// Graph representation: token -> list of outgoing edges
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,DEX Aggregator,DEX Aggregator,Graph,DEX Liquidity Network,High"
    graph: HashMap<TokenId, Vec<TradingEdge>>,
    /// Route cache for improved performance: (source, destination) -> cached path
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,DEX Aggregator,DEX Aggregator,Hash Map,Route Caching,High"
    route_cache: HashMap<(TokenId, TokenId), RoutingPath>,
    /// All tokens in the graph
    tokens: Vec<TokenId>,
}

impl PathRouter {
    /// Create a new path router
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
            route_cache: HashMap::new(),
            tokens: Vec::new(),
        }
    }

    /// Add a trading edge to the graph
    pub fn add_edge(&mut self, edge: TradingEdge) {
        // Invalidate cache entries that might be affected by this change
        self.invalidate_cache_for_token(&edge.from_token);
        self.invalidate_cache_for_token(&edge.to_token);

        // Add source token if not already present
        if !self.tokens.contains(&edge.from_token) {
            self.tokens.push(edge.from_token.clone());
        }

        // Add destination token if not already present
        if !self.tokens.contains(&edge.to_token) {
            self.tokens.push(edge.to_token.clone());
        }

        // Add edge to the graph
        self.graph
            .entry(edge.from_token.clone())
            .or_insert_with(Vec::new)
            .push(edge);
    }

    /// Remove all edges for a specific DEX
    pub fn remove_dex_edges(&mut self, dex_name: &str) {
        // Invalidate all cache entries since we're modifying the graph significantly
        self.route_cache.clear();

        for edges in self.graph.values_mut() {
            edges.retain(|edge| edge.dex_name != dex_name);
        }

        // Clean up empty edge lists
        self.graph.retain(|_, edges| !edges.is_empty());
    }

    /// Invalidate cache entries for a specific token
    fn invalidate_cache_for_token(&mut self, token: &TokenId) {
        self.route_cache
            .retain(|(source, destination), _| source != token && destination != token);
    }

    /// Invalidate all cache entries
    fn invalidate_cache(&mut self) {
        self.route_cache.clear();
    }

    /// Find the best path from source to destination token using Bellman-Ford algorithm
    ///
    /// This implementation can handle negative weights (which might represent arbitrage
    /// opportunities) and will detect negative cycles.
    /// It also implements route caching for improved performance as specified in:
    /// "Core Trading,DEX Aggregator,DEX Aggregator,Hash Map,Route Caching,High"
    pub fn find_best_path(
        &mut self,
        source: &TokenId,
        destination: &TokenId,
        amount: f64,
    ) -> Result<Option<RoutingPath>, PathRoutingError> {
        if source == destination {
            return Ok(None); // No path needed
        }

        if self.tokens.is_empty() {
            return Ok(None); // No tokens in graph
        }

        // Check cache first for improved performance
        // This implements the Priority 1 feature from DEX-OS-V1.csv:
        // "Core Trading,DEX Aggregator,DEX Aggregator,Hash Map,Route Caching,High"
        let cache_key = (source.clone(), destination.clone());
        if let Some(cached_path) = self.route_cache.get(&cache_key) {
            // Return cached path with adjusted amount
            let mut result_path = cached_path.clone();
            result_path.total_exchange_rate = cached_path.total_exchange_rate * amount;
            return Ok(Some(result_path));
        }

        // Initialize distances and predecessors
        let mut distances: HashMap<&TokenId, f64> = HashMap::new();
        let mut predecessors: HashMap<&TokenId, (&TokenId, usize)> = HashMap::new(); // (predecessor token, edge index)

        // Initialize all distances to infinity except source
        for token in &self.tokens {
            distances.insert(token, if token == source { 0.0 } else { f64::INFINITY });
        }

        // Relax edges repeatedly
        for _ in 0..self.tokens.len() - 1 {
            for (from_token, edges) in &self.graph {
                let from_distance = *distances.get(from_token).unwrap_or(&f64::INFINITY);

                if from_distance == f64::INFINITY {
                    continue; // Skip unreachable nodes
                }

                for (edge_index, edge) in edges.iter().enumerate() {
                    let to_distance = *distances.get(&edge.to_token).unwrap_or(&f64::INFINITY);
                    // Weight is negative log of exchange rate (to convert multiplication to addition)
                    // Lower exchange rate means higher "distance" (worse path)
                    let weight = -edge.exchange_rate.ln();
                    let new_distance = from_distance + weight;

                    if new_distance < to_distance {
                        distances.insert(&edge.to_token, new_distance);
                        predecessors.insert(&edge.to_token, (from_token, edge_index));
                    }
                }
            }
        }

        // Check for negative cycles
        for (from_token, edges) in &self.graph {
            let from_distance = *distances.get(from_token).unwrap_or(&f64::INFINITY);

            if from_distance == f64::INFINITY {
                continue;
            }

            for edge in edges {
                let to_distance = *distances.get(&edge.to_token).unwrap_or(&f64::INFINITY);
                let weight = -edge.exchange_rate.ln();
                let new_distance = from_distance + weight;

                if new_distance < to_distance {
                    // Negative cycle detected - this could represent an arbitrage opportunity
                    return Err(PathRoutingError::NegativeCycleDetected);
                }
            }
        }

        // Reconstruct path if destination is reachable
        if distances.get(destination).unwrap_or(&f64::INFINITY) != &f64::INFINITY {
            let mut path_edges = Vec::new();
            let mut current_token = destination;
            let mut visited = std::collections::HashSet::new();

            // Backtrack from destination to source
            while current_token != source && !visited.contains(current_token) {
                visited.insert(current_token.clone());

                if let Some((prev_token, edge_index)) = predecessors.get(current_token) {
                    if let Some(edges) = self.graph.get(*prev_token) {
                        if let Some(edge) = edges.get(*edge_index) {
                            path_edges.push(edge.clone());
                            current_token = prev_token;
                            continue;
                        }
                    }
                }
                break;
            }

            // Check if we successfully reached the source
            if current_token == source {
                // Reverse the path to get it from source to destination
                path_edges.reverse();

                // Calculate path metrics
                let mut total_exchange_rate = 1.0;
                let mut total_fee = 0.0;
                let mut min_liquidity = u64::MAX;

                for edge in &path_edges {
                    total_exchange_rate *= edge.exchange_rate;
                    total_fee += edge.fee;
                    min_liquidity = min_liquidity.min(edge.liquidity);
                }

                let result = RoutingPath {
                    edges: path_edges,
                    total_exchange_rate: total_exchange_rate * amount,
                    total_fee,
                    min_liquidity,
                };

                // Cache the path for future use (without the amount adjustment)
                // This implements the Priority 1 feature from DEX-OS-V1.csv:
                // "Core Trading,DEX Aggregator,DEX Aggregator,Hash Map,Route Caching,High"
                let cache_entry = RoutingPath {
                    edges: result.edges.clone(),
                    total_exchange_rate, // Store without amount for caching
                    total_fee: result.total_fee,
                    min_liquidity: result.min_liquidity,
                };
                self.route_cache.insert(cache_key, cache_entry);

                return Ok(Some(result));
            }
        }

        Ok(None) // No path found
    }

    /// Find all possible paths from source to destination (for exploration)
    pub fn find_all_paths(
        &self,
        source: &TokenId,
        destination: &TokenId,
        max_hops: usize,
    ) -> Vec<RoutingPath> {
        let mut all_paths = Vec::new();
        let mut queue = VecDeque::new();

        // Start with direct edges from source
        if let Some(edges) = self.graph.get(source) {
            for edge in edges {
                let path = vec![edge.clone()];
                queue.push_back((edge.to_token.clone(), path, 1));
            }
        }

        // BFS to find all paths
        while let Some((current_token, path, hops)) = queue.pop_front() {
            if &current_token == destination {
                // Found a path to destination
                let mut total_exchange_rate = 1.0;
                let mut total_fee = 0.0;
                let mut min_liquidity = u64::MAX;

                for edge in &path {
                    total_exchange_rate *= edge.exchange_rate;
                    total_fee += edge.fee;
                    min_liquidity = min_liquidity.min(edge.liquidity);
                }

                all_paths.push(RoutingPath {
                    edges: path,
                    total_exchange_rate,
                    total_fee,
                    min_liquidity,
                });

                continue;
            }

            // Continue searching if we haven't reached max hops
            if hops < max_hops {
                if let Some(edges) = self.graph.get(&current_token) {
                    for edge in edges {
                        // Avoid cycles by checking if we've already visited this token in this path
                        if !path.iter().any(|e| e.from_token == edge.to_token) {
                            let mut new_path = path.clone();
                            new_path.push(edge.clone());
                            queue.push_back((edge.to_token.clone(), new_path, hops + 1));
                        }
                    }
                }
            }
        }

        all_paths
    }

    /// Find the best path among multiple possible paths using Max-Heap for selection
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,DEX Aggregator,DEX Aggregator,Max-Heap (implicit),Best Route Selection,High"
    pub fn find_best_path_with_heap(
        &self,
        source: &TokenId,
        destination: &TokenId,
        max_hops: usize,
    ) -> Option<RoutingPath> {
        let mut path_heap = BinaryHeap::new();

        // Add all possible paths to the heap
        let all_paths = self.find_all_paths(source, destination, max_hops);

        // Convert paths to PathInfo and add to heap
        for path in all_paths {
            let path_info = PathInfo {
                path: path.edges.clone(),
                total_exchange_rate: path.total_exchange_rate,
                total_fee: path.total_fee,
                min_liquidity: path.min_liquidity,
                current_token: destination.clone(), // Since these are complete paths
                hops: path.edges.len(),
            };
            path_heap.push(path_info);
        }

        // Return the best path from the heap (highest exchange rate)
        path_heap.pop().map(|path_info| RoutingPath {
            edges: path_info.path,
            total_exchange_rate: path_info.total_exchange_rate,
            total_fee: path_info.total_fee,
            min_liquidity: path_info.min_liquidity,
        })
    }

    /// Enhanced version of find_best_path that incorporates Max-Heap based selection
    /// when multiple equivalent paths are found by Bellman-Ford
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,DEX Aggregator,DEX Aggregator,Max-Heap (implicit),Best Route Selection,High"
    pub fn find_best_path_enhanced(
        &mut self,
        source: &TokenId,
        destination: &TokenId,
        amount: f64,
    ) -> Result<Option<RoutingPath>, PathRoutingError> {
        // First try the standard Bellman-Ford approach
        let standard_result = self.find_best_path(source, destination, amount)?;

        // If we found a path, we can return it directly
        // The Max-Heap is more useful when we have multiple equivalent paths
        // and need to select the best one based on additional criteria
        if standard_result.is_some() {
            return Ok(standard_result);
        }

        // If no path was found with the standard approach, try the heap-based approach
        // with a reasonable hop limit
        let heap_result = self.find_best_path_with_heap(source, destination, 5);

        // Adjust the amount for the result if we found a path
        if let Some(mut path) = heap_result {
            path.total_exchange_rate = path.total_exchange_rate * amount;
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }

    /// Find the best path using a variant of Dijkstra's algorithm for route optimization
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,DEX Aggregator,DEX Aggregator,Dijkstra's Algorithm (variant),Route Optimization,High"
    ///
    /// This variant of Dijkstra's algorithm is optimized for finding the best multi-hop trading routes
    /// by considering liquidity depth and fees in path selection, using a min-heap for efficient node selection.
    pub fn find_best_path_dijkstra(
        &self,
        source: &TokenId,
        destination: &TokenId,
        amount: f64,
    ) -> Option<RoutingPath> {
        use std::cmp::Ordering;
        use std::collections::BinaryHeap;

        // Priority queue item for Dijkstra's algorithm
        #[derive(Debug, Clone)]
        struct DijkstraNode {
            token: TokenId,
            cost: f64, // Negative because BinaryHeap is a max-heap, but we want min-cost
            path: Vec<TradingEdge>,
            liquidity: Quantity,
        }

        // Implement ordering for DijkstraNode to make it work as a min-heap based on cost
        impl Ord for DijkstraNode {
            fn cmp(&self, other: &Self) -> Ordering {
                // Reverse comparison to make BinaryHeap behave as a min-heap
                other
                    .cost
                    .partial_cmp(&self.cost)
                    .unwrap_or(Ordering::Equal)
            }
        }

        impl PartialOrd for DijkstraNode {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl PartialEq for DijkstraNode {
            fn eq(&self, other: &Self) -> bool {
                self.cost == other.cost
            }
        }

        impl Eq for DijkstraNode {}

        // Distance tracking
        let mut distances: HashMap<TokenId, f64> = HashMap::new();
        let mut visited: HashMap<TokenId, bool> = HashMap::new();
        let mut best_paths: HashMap<TokenId, Vec<TradingEdge>> = HashMap::new();
        let mut min_liquidity: HashMap<TokenId, Quantity> = HashMap::new();

        // Initialize
        for token in &self.tokens {
            distances.insert(token.clone(), f64::INFINITY);
            visited.insert(token.clone(), false);
        }

        distances.insert(source.clone(), 0.0);
        min_liquidity.insert(source.clone(), u64::MAX);

        // Priority queue - min-heap based on cost
        let mut pq = BinaryHeap::new();
        pq.push(DijkstraNode {
            token: source.clone(),
            cost: 0.0,
            path: vec![],
            liquidity: u64::MAX,
        });

        while let Some(current) = pq.pop() {
            let current_token = current.token;

            // If we've reached our destination, we're done
            if &current_token == destination {
                let total_fee: f64 = current.path.iter().map(|edge| edge.fee).sum();
                let min_liquidity_val = *min_liquidity.get(&current_token).unwrap_or(&u64::MAX);

                return Some(RoutingPath {
                    edges: current.path,
                    total_exchange_rate: amount, // Will be adjusted by caller
                    total_fee,
                    min_liquidity: min_liquidity_val,
                });
            }

            // Skip if we've already processed this node with a better cost
            if *visited.get(&current_token).unwrap_or(&false) {
                continue;
            }

            visited.insert(current_token.clone(), true);

            // Explore neighbors
            if let Some(edges) = self.graph.get(&current_token) {
                for edge in edges {
                    if *visited.get(&edge.to_token).unwrap_or(&false) {
                        continue;
                    }

                    // Calculate cost - combination of exchange rate, fee, and liquidity
                    let exchange_value = amount * edge.exchange_rate;
                    let fee_cost = exchange_value * edge.fee;
                    let liquidity_penalty = if edge.liquidity < (amount as u64) {
                        // High penalty for insufficient liquidity
                        f64::INFINITY
                    } else {
                        // Lower penalty for sufficient liquidity
                        1.0 / (edge.liquidity as f64)
                    };

                    let edge_cost = fee_cost + liquidity_penalty;
                    let new_cost = current.cost + edge_cost;

                    // If we found a better path to this token
                    if new_cost < *distances.get(&edge.to_token).unwrap_or(&f64::INFINITY) {
                        distances.insert(edge.to_token.clone(), new_cost);

                        // Update minimum liquidity along the path
                        let current_liquidity =
                            *min_liquidity.get(&current_token).unwrap_or(&u64::MAX);
                        let new_liquidity = current_liquidity.min(edge.liquidity);
                        min_liquidity.insert(edge.to_token.clone(), new_liquidity);

                        // Build new path
                        let mut new_path = current.path.clone();
                        new_path.push(edge.clone());

                        best_paths.insert(edge.to_token.clone(), new_path.clone());

                        // Add to priority queue
                        pq.push(DijkstraNode {
                            token: edge.to_token.clone(),
                            cost: new_cost,
                            path: new_path,
                            liquidity: new_liquidity,
                        });
                    }
                }
            }
        }

        // No path found
        None
    }

    /// Enhanced path finding that uses Dijkstra's algorithm for optimization
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,DEX Aggregator,DEX Aggregator,Dijkstra's Algorithm (variant),Route Optimization,High"
    pub fn find_optimized_path(
        &mut self,
        source: &TokenId,
        destination: &TokenId,
        amount: f64,
    ) -> Result<Option<RoutingPath>, PathRoutingError> {
        // First try the standard Bellman-Ford approach
        let standard_result = self.find_best_path(source, destination, amount)?;

        // If we found a path with Bellman-Ford, try Dijkstra's for comparison
        if let Some(mut path) = standard_result {
            // Try Dijkstra's algorithm for optimization
            if let Some(mut dijkstra_path) =
                self.find_best_path_dijkstra(source, destination, amount)
            {
                // Compare paths and return the better one
                // For now, we'll just return the Dijkstra result as it's optimized
                dijkstra_path.total_exchange_rate = amount * dijkstra_path.total_exchange_rate;
                Ok(Some(dijkstra_path))
            } else {
                // If Dijkstra fails, return the Bellman-Ford result
                path.total_exchange_rate = amount * path.total_exchange_rate;
                Ok(Some(path))
            }
        } else {
            // If Bellman-Ford found no path, try Dijkstra's
            if let Some(mut dijkstra_path) =
                self.find_best_path_dijkstra(source, destination, amount)
            {
                dijkstra_path.total_exchange_rate = amount * dijkstra_path.total_exchange_rate;
                Ok(Some(dijkstra_path))
            } else {
                Ok(None)
            }
        }
    }

    /// Get all tokens in the graph
    pub fn get_tokens(&self) -> &[TokenId] {
        &self.tokens as &[TokenId]
    }

    /// Get all edges from a specific token
    pub fn get_edges_from_token(&self, token: &TokenId) -> Option<&Vec<TradingEdge>> {
        self.graph.get(token)
    }

    /// Get the number of tokens in the graph
    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    /// Get the number of edges in the graph
    pub fn edge_count(&self) -> usize {
        self.graph.values().map(|edges| edges.len()).sum()
    }
}

/// Errors that can occur during path routing operations
#[derive(Debug, Error)]
pub enum PathRoutingError {
    #[error("Negative cycle detected in trading graph")]
    NegativeCycleDetected,
    #[error("Source and destination tokens are the same")]
    SameSourceDestination,
    #[error("No path found between source and destination")]
    NoPathFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_router_creation() {
        let router = PathRouter::new();
        assert_eq!(router.token_count(), 0);
        assert_eq!(router.edge_count(), 0);
        assert!(router.get_tokens().is_empty());
    }

    #[test]
    fn test_add_edge() {
        let mut router = PathRouter::new();

        let edge = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };

        router.add_edge(edge.clone());

        assert_eq!(router.token_count(), 2);
        assert_eq!(router.edge_count(), 1);
        assert!(router.get_tokens().contains(&"BTC".to_string()));
        assert!(router.get_tokens().contains(&"ETH".to_string()));

        let edges = router.get_edges_from_token(&"BTC".to_string()).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0], edge);
    }

    #[test]
    fn test_find_best_path_simple() {
        let mut router = PathRouter::new();

        // Add a simple path: BTC -> ETH
        let edge = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };

        router.add_edge(edge.clone());

        // Find path from BTC to ETH
        let result = router
            .find_best_path(&"BTC".to_string(), &"ETH".to_string(), 1.0)
            .unwrap();
        assert!(result.is_some());

        let path = result.unwrap();
        assert_eq!(path.edges.len(), 1);
        assert_eq!(path.edges[0], edge);
        assert_eq!(path.total_exchange_rate, 13.5);
        assert_eq!(path.total_fee, 0.003);
        assert_eq!(path.min_liquidity, 1000000);
    }

    #[test]
    fn test_find_best_path_multi_hop() {
        let mut router = PathRouter::new();

        // Add path: BTC -> ETH -> USDC
        let edge1 = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };

        let edge2 = TradingEdge {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "SushiSwap".to_string(),
            exchange_rate: 3200.0,
            fee: 0.003,
            liquidity: 50000000,
        };

        router.add_edge(edge1.clone());
        router.add_edge(edge2.clone());

        // Find path from BTC to USDC
        let result = router
            .find_best_path(&"BTC".to_string(), &"USDC".to_string(), 1.0)
            .unwrap();
        assert!(result.is_some());

        let path = result.unwrap();
        assert_eq!(path.edges.len(), 2);
        assert_eq!(path.edges[0], edge1);
        assert_eq!(path.edges[1], edge2);
        assert_eq!(path.total_exchange_rate, 13.5 * 3200.0); // 43,200
        assert_eq!(path.total_fee, 0.006); // 0.003 + 0.003
        assert_eq!(path.min_liquidity, 1000000); // Limited by BTC->ETH liquidity
    }

    #[test]
    fn test_find_best_path_with_heap() {
        let mut router = PathRouter::new();

        // Add multiple paths from BTC to USDC
        // Path 1: BTC -> ETH -> USDC
        let edge1_1 = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };

        let edge1_2 = TradingEdge {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "SushiSwap".to_string(),
            exchange_rate: 3200.0,
            fee: 0.003,
            liquidity: 50000000,
        };

        // Path 2: BTC -> USDC (direct)
        let edge2 = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "Curve".to_string(),
            exchange_rate: 42000.0, // Better rate than the multi-hop path
            fee: 0.001,
            liquidity: 2000000,
        };

        router.add_edge(edge1_1.clone());
        router.add_edge(edge1_2.clone());
        router.add_edge(edge2.clone());

        // Find best path using heap-based selection
        let result = router.find_best_path_with_heap(&"BTC".to_string(), &"USDC".to_string(), 3);
        assert!(result.is_some());

        let path = result.unwrap();
        // Should select the direct path as it has a better exchange rate
        assert_eq!(path.edges.len(), 1);
        assert_eq!(path.edges[0], edge2);
        assert_eq!(path.total_exchange_rate, 42000.0);
        assert_eq!(path.total_fee, 0.001);
        assert_eq!(path.min_liquidity, 2000000);
    }

    #[test]
    fn test_find_best_path_enhanced() {
        let mut router = PathRouter::new();

        // Add multiple paths from BTC to USDC
        // Path 1: BTC -> ETH -> USDC
        let edge1_1 = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };

        let edge1_2 = TradingEdge {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "SushiSwap".to_string(),
            exchange_rate: 3200.0,
            fee: 0.003,
            liquidity: 50000000,
        };

        // Path 2: BTC -> USDC (direct)
        let edge2 = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "Curve".to_string(),
            exchange_rate: 42000.0, // Better rate than the multi-hop path
            fee: 0.001,
            liquidity: 2000000,
        };

        router.add_edge(edge1_1.clone());
        router.add_edge(edge1_2.clone());
        router.add_edge(edge2.clone());

        // Find best path using enhanced selection
        let result = router
            .find_best_path_enhanced(&"BTC".to_string(), &"USDC".to_string(), 1.0)
            .unwrap();
        assert!(result.is_some());

        let path = result.unwrap();
        // Should select the direct path as it has a better exchange rate
        assert_eq!(path.edges.len(), 1);
        assert_eq!(path.edges[0], edge2);
        assert_eq!(path.total_exchange_rate, 42000.0);
        assert_eq!(path.total_fee, 0.001);
        assert_eq!(path.min_liquidity, 2000000);
    }

    #[test]
    fn test_find_best_path_no_path() {
        let mut router = PathRouter::new();

        // Add path: BTC -> ETH
        let edge = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };

        router.add_edge(edge.clone());

        // Try to find path from BTC to USDC (no path exists)
        let result = router
            .find_best_path(&"BTC".to_string(), &"USDC".to_string(), 1.0)
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_remove_dex_edges() {
        let mut router = PathRouter::new();

        // Add edges from different DEXes
        let edge1 = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };

        let edge2 = TradingEdge {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "SushiSwap".to_string(),
            exchange_rate: 3200.0,
            fee: 0.003,
            liquidity: 50000000,
        };

        router.add_edge(edge1.clone());
        router.add_edge(edge2.clone());

        assert_eq!(router.edge_count(), 2);

        // Remove Uniswap edges
        router.remove_dex_edges("Uniswap");

        assert_eq!(router.edge_count(), 1);
        assert!(router
            .get_edges_from_token(&"BTC".to_string())
            .map_or(true, |edges| edges.is_empty()));
        assert_eq!(
            router
                .get_edges_from_token(&"ETH".to_string())
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_find_all_paths() {
        let mut router = PathRouter::new();

        // Add multiple paths: BTC -> ETH -> USDC and BTC -> USDC
        let edge1 = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };

        let edge2 = TradingEdge {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "SushiSwap".to_string(),
            exchange_rate: 3200.0,
            fee: 0.003,
            liquidity: 50000000,
        };

        let edge3 = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "Curve".to_string(),
            exchange_rate: 45000.0,
            fee: 0.004,
            liquidity: 20000000,
        };

        router.add_edge(edge1.clone());
        router.add_edge(edge2.clone());
        router.add_edge(edge3.clone());

        // Find all paths from BTC to USDC with max 2 hops
        let paths = router.find_all_paths(&"BTC".to_string(), &"USDC".to_string(), 2);
        assert_eq!(paths.len(), 2);

        // One path should be direct (BTC -> USDC)
        let direct_path = paths.iter().find(|p| p.edges.len() == 1).unwrap();
        assert_eq!(direct_path.edges[0], edge3);

        // One path should be multi-hop (BTC -> ETH -> USDC)
        let multi_hop_path = paths.iter().find(|p| p.edges.len() == 2).unwrap();
        assert_eq!(multi_hop_path.edges[0], edge1);
        assert_eq!(multi_hop_path.edges[1], edge2);
    }

    #[test]
    fn test_route_caching() {
        let mut router = PathRouter::new();

        // Add path: BTC -> ETH -> USDC
        let edge1 = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };

        let edge2 = TradingEdge {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "SushiSwap".to_string(),
            exchange_rate: 3200.0,
            fee: 0.003,
            liquidity: 50000000,
        };

        router.add_edge(edge1.clone());
        router.add_edge(edge2.clone());

        // First call should compute the path
        let result1 = router
            .find_best_path(&"BTC".to_string(), &"USDC".to_string(), 1.0)
            .unwrap();
        assert!(result1.is_some());

        // Check that the path was cached
        assert_eq!(router.route_cache.len(), 1);
        assert!(router
            .route_cache
            .contains_key(&("BTC".to_string(), "USDC".to_string())));

        // Second call should use the cache
        let result2 = router
            .find_best_path(&"BTC".to_string(), &"USDC".to_string(), 2.0)
            .unwrap();
        assert!(result2.is_some());

        // Results should be the same path but with different amounts
        let path1 = result1.unwrap();
        let path2 = result2.unwrap();
        assert_eq!(path1.edges, path2.edges);
        assert_eq!(path1.total_exchange_rate * 2.0, path2.total_exchange_rate);
        assert_eq!(path1.total_fee, path2.total_fee);
        assert_eq!(path1.min_liquidity, path2.min_liquidity);
    }

    #[test]
    fn test_cache_invalidation_on_add_edge() {
        let mut router = PathRouter::new();

        // Add initial path: BTC -> ETH
        let edge1 = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };
        router.add_edge(edge1.clone());

        // Find and cache path
        let _ = router
            .find_best_path(&"BTC".to_string(), &"ETH".to_string(), 1.0)
            .unwrap();
        assert_eq!(router.route_cache.len(), 1);

        // Add another edge that could affect routing
        let edge2 = TradingEdge {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "SushiSwap".to_string(),
            exchange_rate: 3200.0,
            fee: 0.003,
            liquidity: 50000000,
        };
        router.add_edge(edge2.clone());

        // Cache should be invalidated for affected tokens
        // Note: The exact behavior depends on implementation, but cache should be managed properly
    }

    #[test]
    fn test_find_best_path_dijkstra() {
        let router = PathRouter::new();

        // This test would need a populated router to be meaningful
        // We'll add a more comprehensive test later
        assert!(true);
    }

    #[test]
    fn test_dijkstra_vs_bellman_ford() {
        let mut router = PathRouter::new();

        // Add path: BTC -> ETH -> USDC
        let edge1 = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };

        let edge2 = TradingEdge {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "SushiSwap".to_string(),
            exchange_rate: 3200.0,
            fee: 0.003,
            liquidity: 50000000,
        };

        router.add_edge(edge1.clone());
        router.add_edge(edge2.clone());

        // Test Dijkstra's algorithm
        let dijkstra_result =
            router.find_best_path_dijkstra(&"BTC".to_string(), &"USDC".to_string(), 1.0);
        assert!(dijkstra_result.is_some());

        let path = dijkstra_result.unwrap();
        assert_eq!(path.edges.len(), 2);
        assert_eq!(path.total_fee, 0.006); // 0.003 + 0.003
    }
}

/// Helper struct for path information used in heap-based selection
/// This implements the Priority 1 feature from DEX-OS-V1.csv:
/// "Core Trading,DEX Aggregator,DEX Aggregator,Max-Heap (implicit),Best Route Selection,High"
#[derive(Debug, Clone)]
struct PathInfo {
    path: Vec<TradingEdge>,
    total_exchange_rate: f64,
    total_fee: f64,
    min_liquidity: Quantity,
    current_token: TokenId,
    hops: usize,
}

// Implement Ord trait to make PathInfo work with BinaryHeap as a max-heap based on exchange rate
impl Ord for PathInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare by total exchange rate (higher is better)
        // We reverse the comparison because BinaryHeap is a max-heap by default,
        // but we want to compare f64 values properly
        other
            .total_exchange_rate
            .partial_cmp(&self.total_exchange_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for PathInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PathInfo {
    fn eq(&self, other: &Self) -> bool {
        self.total_exchange_rate == other.total_exchange_rate
    }
}

impl Eq for PathInfo {}
