//! IP Routing implementation using Trie data structure
//!
//! Implements Security Layer 6 - Network & Infrastructure Security
//! From DEX-OS-V2.csv line 228:
//! - Infrastructure,Network,Network,Trie,IP Routing,High {Security: Layer 2 - Network Security}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use thiserror::Error;

/// IP routing error types
#[derive(Debug, Error, Clone, PartialEq)]
pub enum RoutingError {
    #[error("Route not found for IP: {0}")]
    RouteNotFound(String),
    #[error("Invalid route: {0}")]
    InvalidRoute(String),
    #[error("Duplicate route: {0}")]
    DuplicateRoute(String),
    #[error("Invalid prefix length: {0}")]
    InvalidPrefixLength(u8),
}

/// Route entry in the routing table
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteEntry {
    /// Network address
    pub network: IpAddr,
    /// Prefix length (CIDR notation)
    pub prefix_len: u8,
    /// Next hop gateway
    pub gateway: IpAddr,
    /// Route metric (cost)
    pub metric: u32,
    /// Route priority
    pub priority: u32,
    /// Interface name
    pub interface: String,
    /// Route description
    pub description: String,
    /// Whether the route is active
    pub active: bool,
    /// Creation timestamp
    pub created_at: u64,
}

impl RouteEntry {
    /// Create a new route entry
    pub fn new(
        network: IpAddr,
        prefix_len: u8,
        gateway: IpAddr,
        metric: u32,
        priority: u32,
        interface: String,
        description: String,
    ) -> Result<Self, RoutingError> {
        // Validate prefix length
        match network {
            IpAddr::V4(_) if prefix_len > 32 => {
                return Err(RoutingError::InvalidPrefixLength(prefix_len));
            }
            IpAddr::V6(_) if prefix_len > 128 => {
                return Err(RoutingError::InvalidPrefixLength(prefix_len));
            }
            _ => {}
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Self {
            network,
            prefix_len,
            gateway,
            metric,
            priority,
            interface,
            description,
            active: true,
            created_at: now,
        })
    }

    /// Check if an IP address matches this route
    pub fn matches(&self, ip: &IpAddr) -> bool {
        match (ip, &self.network) {
            (IpAddr::V4(ip4), IpAddr::V4(net4)) => {
                let ip_bits = u32::from(*ip4);
                let net_bits = u32::from(*net4);
                let mask = !0u32 << (32 - self.prefix_len);
                (ip_bits & mask) == (net_bits & mask)
            }
            (IpAddr::V6(ip6), IpAddr::V6(net6)) => {
                let ip_bits = u128::from(*ip6);
                let net_bits = u128::from(*net6);
                let mask = !0u128 << (128 - self.prefix_len);
                (ip_bits & mask) == (net_bits & mask)
            }
            _ => false,
        }
    }
}

/// Trie node for IP routing
#[derive(Debug, Clone)]
struct TrieNode {
    /// Route entry at this node (if any)
    route: Option<RouteEntry>,
    /// Child nodes (0 and 1 for binary trie)
    children: HashMap<u8, Box<TrieNode>>,
}

impl TrieNode {
    fn new() -> Self {
        Self {
            route: None,
            children: HashMap::new(),
        }
    }

    fn insert_route(
        &mut self,
        bits: u128,
        prefix_len: u8,
        route: RouteEntry,
        max_bits: u8,
        current_depth: u8,
    ) -> Result<(), RoutingError> {
        if current_depth == prefix_len {
            self.route = Some(route);
            return Ok(());
        }

        // For prefix length n, we want to insert at depth n
        // So we look at bit (current_depth) from the left (0-indexed)
        let bit_pos = max_bits - 1 - current_depth;
        let bit = ((bits >> bit_pos) & 1) as u8;
        let child = self
            .children
            .entry(bit)
            .or_insert_with(|| Box::new(TrieNode::new()));

        child.insert_route(bits, prefix_len, route, max_bits, current_depth + 1)
    }
    fn remove_route_internal(
        &mut self,
        bits: u128,
        prefix_len: u8,
        max_bits: u8,
        current_depth: u8,
    ) -> bool {
        if current_depth == prefix_len {
            if self.route.is_some() {
                self.route = None;
                return true;
            }
            return false;
        }

        let bit_pos = max_bits - 1 - current_depth;
        let bit = ((bits >> bit_pos) & 1) as u8;

        if let Some(child) = self.children.get_mut(&bit) {
            let removed = child.remove_route_internal(bits, prefix_len, max_bits, current_depth + 1);
            // Clean up empty nodes
            if !removed {
                return false;
            }
            
            // If child has no route and no children, remove it
            if child.route.is_none() && child.children.is_empty() {
                self.children.remove(&bit);
            }
            
            return removed;
        }

        false
    }
}

/// IP Routing Table using Trie data structure for efficient longest prefix matching
#[derive(Debug, Clone)]
pub struct IPRoutingTable {
    /// IPv4 routing trie
    ipv4_root: TrieNode,
    /// IPv6 routing trie
    ipv6_root: TrieNode,
    /// Total number of routes
    route_count: usize,
    /// Statistics
    lookups_performed: u64,
    lookups_successful: u64,
}

impl IPRoutingTable {
    /// Create a new IP routing table
    pub fn new() -> Self {
        Self {
            ipv4_root: TrieNode::new(),
            ipv6_root: TrieNode::new(),
            route_count: 0,
            lookups_performed: 0,
            lookups_successful: 0,
        }
    }

    /// Add a route to the routing table
    pub fn add_route(&mut self, route: RouteEntry) -> Result<(), RoutingError> {
        match route.network {
            IpAddr::V4(ip4) => {
                let bits = u32::from(ip4);
                self.ipv4_root.insert_route(bits as u128, route.prefix_len, route, 32, 0)?;
            }
            IpAddr::V6(ip6) => {
                let bits = u128::from(ip6);
                self.ipv6_root.insert_route(bits, route.prefix_len, route, 128, 0)?;
            }
        }
        self.route_count += 1;
        Ok(())
    }

    /// Remove a route from the routing table
    pub fn remove_route(&mut self, network: IpAddr, prefix_len: u8) -> Result<(), RoutingError> {
        let removed = match network {
            IpAddr::V4(ip4) => {
                let bits = u32::from(ip4);
                self.ipv4_root.remove_route_internal(bits as u128, prefix_len, 32, 0)
            }
            IpAddr::V6(ip6) => {
                let bits = u128::from(ip6);
                self.ipv6_root.remove_route_internal(bits, prefix_len, 128, 0)
            }
        };

        if removed {
            self.route_count -= 1;
            Ok(())
        } else {
            Err(RoutingError::RouteNotFound(format!(
                "{}/{}",
                network, prefix_len
            )))
        }
    }

    /// Lookup a route using longest prefix matching
    pub fn lookup(&mut self, ip: &IpAddr) -> Option<RouteEntry> {
        self.lookups_performed += 1;

        let result = match ip {
            IpAddr::V4(ip4) => {
                let bits = u32::from(*ip4);
                self.longest_prefix_match(&self.ipv4_root, bits as u128, 32)
            }
            IpAddr::V6(ip6) => {
                let bits = u128::from(*ip6);
                self.longest_prefix_match(&self.ipv6_root, bits, 128)
            }
        };

        if result.is_some() {
            self.lookups_successful += 1;
        }

        result
    }

    /// Get all routes
    pub fn get_all_routes(&self) -> Vec<RouteEntry> {
        let mut routes = Vec::new();
        self.collect_routes(&self.ipv4_root, &mut routes);
        self.collect_routes(&self.ipv6_root, &mut routes);
        routes
    }

    /// Get active routes only
    pub fn get_active_routes(&self) -> Vec<RouteEntry> {
        self.get_all_routes()
            .into_iter()
            .filter(|r| r.active)
            .collect()
    }

    /// Get routing statistics
    pub fn get_statistics(&self) -> RoutingStatistics {
        RoutingStatistics {
            total_routes: self.route_count,
            active_routes: self.get_active_routes().len(),
            lookups_performed: self.lookups_performed,
            lookups_successful: self.lookups_successful,
            lookup_success_rate: if self.lookups_performed > 0 {
                (self.lookups_successful as f64 / self.lookups_performed as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Clear all routes
    pub fn clear(&mut self) {
        self.ipv4_root = TrieNode::new();
        self.ipv6_root = TrieNode::new();
        self.route_count = 0;
    }

    // Internal helper methods


    fn longest_prefix_match(&self, node: &TrieNode, bits: u128, remaining_bits: u8) -> Option<RouteEntry> {
        let mut best_match = node.route.clone();

        if remaining_bits == 0 {
            return best_match;
        }

        let bit = ((bits >> (remaining_bits - 1)) & 1) as u8;

        if let Some(child) = node.children.get(&bit) {
            if let Some(child_match) = self.longest_prefix_match(child, bits, remaining_bits - 1) {
                best_match = Some(child_match);
            }
        }

        best_match
    }
    fn collect_routes(&self, node: &TrieNode, routes: &mut Vec<RouteEntry>) {
        if let Some(route) = &node.route {
            routes.push(route.clone());
        }

        for child in node.children.values() {
            self.collect_routes(child, routes);
        }
    }
}

/// Routing statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutingStatistics {
    pub total_routes: usize,
    pub active_routes: usize,
    pub lookups_performed: u64,
    pub lookups_successful: u64,
    pub lookup_success_rate: f64,
}

impl Default for IPRoutingTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_entry_creation() {
        let route = RouteEntry::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)),
            24,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            10,
            100,
            "eth0".to_string(),
            "Local network".to_string(),
        );

        assert!(route.is_ok());
        let route = route.unwrap();
        assert_eq!(route.prefix_len, 24);
        assert!(route.active);
    }

    #[test]
    fn test_route_matching() {
        let route = RouteEntry::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)),
            24,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            10,
            100,
            "eth0".to_string(),
            "Local network".to_string(),
        )
        .unwrap();

        let ip1 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let ip2 = IpAddr::V4(Ipv4Addr::new(192, 168, 2, 100));

        assert!(route.matches(&ip1));
        assert!(!route.matches(&ip2));
    }

    #[test]
    fn test_routing_table_add_route() {
        let mut table = IPRoutingTable::new();
        let route = RouteEntry::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)),
            24,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            10,
            100,
            "eth0".to_string(),
            "Local network".to_string(),
        )
        .unwrap();

        assert!(table.add_route(route).is_ok());
        assert_eq!(table.get_statistics().total_routes, 1);
    }

    #[test]
    fn test_longest_prefix_matching() {
        let mut table = IPRoutingTable::new();

        // Add a /24 route
        let route1 = RouteEntry::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)),
            24,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            10,
            100,
            "eth0".to_string(),
            "Local network".to_string(),
        )
        .unwrap();

        // Add a more specific /28 route
        let route2 = RouteEntry::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 128)),
            28,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 129)),
            5,
            200,
            "eth1".to_string(),
            "Specific subnet".to_string(),
        )
        .unwrap();

        table.add_route(route1).unwrap();
        table.add_route(route2).unwrap();

        // Lookup should return the more specific route
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 130));
        let result = table.lookup(&ip);

        assert!(result.is_some());
        let route = result.unwrap();
        assert_eq!(route.prefix_len, 28);
        assert_eq!(route.interface, "eth1");
    }
}
