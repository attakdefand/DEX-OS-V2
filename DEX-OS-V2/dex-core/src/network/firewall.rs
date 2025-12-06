//! Firewall implementation for DEX-OS Network Security
//!
//! Implements Security Layer 6 - Network & Infrastructure Security (Perimeter Defense)
//! From DEX-OS-V2.csv line 226:
//! - Infrastructure,Network,Network,Hash Map,Firewall Rules,High {Security: Layer 2 - Network Security}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use thiserror::Error;

/// Firewall error types
#[derive(Debug, Error, Clone, PartialEq)]
pub enum FirewallError {
    #[error("Rule not found: {0}")]
    RuleNotFound(String),
    #[error("Invalid rule: {0}")]
    InvalidRule(String),
    #[error("Duplicate rule ID: {0}")]
    DuplicateRuleId(String),
    #[error("Invalid IP address: {0}")]
    InvalidIpAddress(String),
    #[error("Invalid port range: {0}")]
    InvalidPortRange(String),
}

/// Firewall rule action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RuleAction {
    /// Allow the packet
    Allow,
    /// Deny the packet
    Deny,
    /// Log the packet and allow
    LogAndAllow,
    /// Log the packet and deny
    LogAndDeny,
}

/// Network protocol types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Any,
}

/// IP address range for firewall rules
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IpRange {
    Single(IpAddr),
    Subnet { network: IpAddr, prefix_len: u8 },
    Any,
}

/// Port range for firewall rules
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

impl PortRange {
    pub fn single(port: u16) -> Self {
        Self { start: port, end: port }
    }

    pub fn range(start: u16, end: u16) -> Result<Self, FirewallError> {
        if start > end {
            return Err(FirewallError::InvalidPortRange(format!(
                "Start port {} is greater than end port {}",
                start, end
            )));
        }
        Ok(Self { start, end })
    }

    pub fn any() -> Self {
        Self { start: 0, end: 65535 }
    }

    pub fn contains(&self, port: u16) -> bool {
        port >= self.start && port <= self.end
    }
}

/// Firewall rule definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FirewallRule {
    /// Unique rule identifier
    pub id: String,
    /// Rule priority (lower number = higher priority)
    pub priority: u32,
    /// Source IP range
    pub source_ip: IpRange,
    /// Destination IP range
    pub destination_ip: IpRange,
    /// Source port range
    pub source_port: PortRange,
    /// Destination port range
    pub destination_port: PortRange,
    /// Protocol
    pub protocol: Protocol,
    /// Action to take
    pub action: RuleAction,
    /// Rule description
    pub description: String,
    /// Whether the rule is enabled
    pub enabled: bool,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
}

impl FirewallRule {
    /// Create a new firewall rule
    pub fn new(
        id: String,
        priority: u32,
        source_ip: IpRange,
        destination_ip: IpRange,
        source_port: PortRange,
        destination_port: PortRange,
        protocol: Protocol,
        action: RuleAction,
        description: String,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            priority,
            source_ip,
            destination_ip,
            source_port,
            destination_port,
            protocol,
            action,
            description,
            enabled: true,
            created_at: now,
            modified_at: now,
        }
    }

    /// Check if a packet matches this rule
    pub fn matches(&self, packet: &NetworkPacket) -> bool {
        if !self.enabled {
            return false;
        }

        self.matches_ip(&self.source_ip, &packet.source_ip)
            && self.matches_ip(&self.destination_ip, &packet.destination_ip)
            && self.source_port.contains(packet.source_port)
            && self.destination_port.contains(packet.destination_port)
            && self.matches_protocol(&packet.protocol)
    }

    fn matches_ip(&self, range: &IpRange, ip: &IpAddr) -> bool {
        match range {
            IpRange::Any => true,
            IpRange::Single(addr) => addr == ip,
            IpRange::Subnet { network, prefix_len } => {
                Self::ip_in_subnet(ip, network, *prefix_len)
            }
        }
    }

    fn ip_in_subnet(ip: &IpAddr, network: &IpAddr, prefix_len: u8) -> bool {
        match (ip, network) {
            (IpAddr::V4(ip4), IpAddr::V4(net4)) => {
                let ip_bits = u32::from(*ip4);
                let net_bits = u32::from(*net4);
                let mask = !0u32 << (32 - prefix_len);
                (ip_bits & mask) == (net_bits & mask)
            }
            (IpAddr::V6(ip6), IpAddr::V6(net6)) => {
                let ip_bits = u128::from(*ip6);
                let net_bits = u128::from(*net6);
                let mask = !0u128 << (128 - prefix_len);
                (ip_bits & mask) == (net_bits & mask)
            }
            _ => false,
        }
    }

    fn matches_protocol(&self, protocol: &Protocol) -> bool {
        self.protocol == Protocol::Any || self.protocol == *protocol
    }
}

/// Network packet representation for firewall inspection
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkPacket {
    pub source_ip: IpAddr,
    pub destination_ip: IpAddr,
    pub source_port: u16,
    pub destination_port: u16,
    pub protocol: Protocol,
    pub payload_size: usize,
}

/// Connection state for stateful firewall
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionState {
    New,
    Established,
    Related,
    Invalid,
}

/// Stateful connection tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    pub source_ip: IpAddr,
    pub destination_ip: IpAddr,
    pub source_port: u16,
    pub destination_port: u16,
    pub protocol: Protocol,
    pub state: ConnectionState,
    pub created_at: u64,
    pub last_seen: u64,
    pub packet_count: u64,
    pub byte_count: u64,
}

/// Firewall rules manager using Hash Map for efficient rule storage and lookup
#[derive(Debug, Clone)]
pub struct FirewallRulesManager {
    /// Rules stored in a HashMap for O(1) lookup by ID
    rules: HashMap<String, FirewallRule>,
    /// Sorted rules by priority for matching
    sorted_rules: Vec<String>,
    /// Connection tracking for stateful firewall
    connections: HashMap<String, Connection>,
    /// Default action when no rules match
    default_action: RuleAction,
    /// Statistics
    packets_processed: u64,
    packets_allowed: u64,
    packets_denied: u64,
}

impl FirewallRulesManager {
    /// Create a new firewall rules manager
    pub fn new(default_action: RuleAction) -> Self {
        Self {
            rules: HashMap::new(),
            sorted_rules: Vec::new(),
            connections: HashMap::new(),
            default_action,
            packets_processed: 0,
            packets_allowed: 0,
            packets_denied: 0,
        }
    }

    /// Add a firewall rule
    pub fn add_rule(&mut self, rule: FirewallRule) -> Result<(), FirewallError> {
        if self.rules.contains_key(&rule.id) {
            return Err(FirewallError::DuplicateRuleId(rule.id.clone()));
        }

        let rule_id = rule.id.clone();
        self.rules.insert(rule_id.clone(), rule);
        self.sorted_rules.push(rule_id);
        self.sort_rules();

        Ok(())
    }

    /// Remove a firewall rule
    pub fn remove_rule(&mut self, rule_id: &str) -> Result<(), FirewallError> {
        if !self.rules.contains_key(rule_id) {
            return Err(FirewallError::RuleNotFound(rule_id.to_string()));
        }

        self.rules.remove(rule_id);
        self.sorted_rules.retain(|id| id != rule_id);

        Ok(())
    }

    /// Update a firewall rule
    pub fn update_rule(&mut self, rule: FirewallRule) -> Result<(), FirewallError> {
        if !self.rules.contains_key(&rule.id) {
            return Err(FirewallError::RuleNotFound(rule.id.clone()));
        }

        let mut updated_rule = rule;
        updated_rule.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.rules.insert(updated_rule.id.clone(), updated_rule);
        self.sort_rules();

        Ok(())
    }

    /// Get a firewall rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Option<&FirewallRule> {
        self.rules.get(rule_id)
    }

    /// Enable a firewall rule
    pub fn enable_rule(&mut self, rule_id: &str) -> Result<(), FirewallError> {
        if let Some(rule) = self.rules.get_mut(rule_id) {
            rule.enabled = true;
            rule.modified_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Ok(())
        } else {
            Err(FirewallError::RuleNotFound(rule_id.to_string()))
        }
    }

    /// Disable a firewall rule
    pub fn disable_rule(&mut self, rule_id: &str) -> Result<(), FirewallError> {
        if let Some(rule) = self.rules.get_mut(rule_id) {
            rule.enabled = false;
            rule.modified_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Ok(())
        } else {
            Err(FirewallError::RuleNotFound(rule_id.to_string()))
        }
    }

    /// Process a packet through the firewall
    pub fn process_packet(&mut self, packet: &NetworkPacket) -> RuleAction {
        self.packets_processed += 1;

        // Check stateful connections first
        let conn_key = self.connection_key(packet);
        if let Some(conn) = self.connections.get_mut(&conn_key) {
            conn.last_seen = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            conn.packet_count += 1;
            conn.byte_count += packet.payload_size as u64;

            if conn.state == ConnectionState::Established {
                self.packets_allowed += 1;
                return RuleAction::Allow;
            }
        }

        // Check rules in priority order
        for rule_id in &self.sorted_rules {
            if let Some(rule) = self.rules.get(rule_id) {
                if rule.matches(packet) {
                    let action = rule.action.clone();
                    
                    // Track new connections
                    if matches!(action, RuleAction::Allow | RuleAction::LogAndAllow) {
                        self.track_connection(packet);
                        self.packets_allowed += 1;
                    } else {
                        self.packets_denied += 1;
                    }

                    return action;
                }
            }
        }

        // No rule matched, use default action
        let action = self.default_action.clone();
        match &action {
            RuleAction::Allow | RuleAction::LogAndAllow => {
                self.track_connection(packet);
                self.packets_allowed += 1;
            },
            RuleAction::Deny | RuleAction::LogAndDeny => self.packets_denied += 1,
        }

        action    }

    /// Get all rules
    pub fn get_all_rules(&self) -> Vec<&FirewallRule> {
        self.sorted_rules
            .iter()
            .filter_map(|id| self.rules.get(id))
            .collect()
    }

    /// Get statistics
    pub fn get_statistics(&self) -> FirewallStatistics {
        FirewallStatistics {
            total_rules: self.rules.len(),
            enabled_rules: self.rules.values().filter(|r| r.enabled).count(),
            packets_processed: self.packets_processed,
            packets_allowed: self.packets_allowed,
            packets_denied: self.packets_denied,
            active_connections: self.connections.len(),
        }
    }

    /// Clear old connections (connection timeout)
    pub fn cleanup_connections(&mut self, timeout_seconds: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.connections
            .retain(|_, conn| now - conn.last_seen < timeout_seconds);
    }

    fn sort_rules(&mut self) {
        self.sorted_rules.sort_by(|a, b| {
            let rule_a = self.rules.get(a).unwrap();
            let rule_b = self.rules.get(b).unwrap();
            rule_a.priority.cmp(&rule_b.priority)
        });
    }

    fn connection_key(&self, packet: &NetworkPacket) -> String {
        format!(
            "{}:{}->{}:{}:{:?}",
            packet.source_ip,
            packet.source_port,
            packet.destination_ip,
            packet.destination_port,
            packet.protocol
        )
    }

    fn track_connection(&mut self, packet: &NetworkPacket) {
        let key = self.connection_key(packet);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let connection = Connection {
            source_ip: packet.source_ip,
            destination_ip: packet.destination_ip,
            source_port: packet.source_port,
            destination_port: packet.destination_port,
            protocol: packet.protocol.clone(),
            state: ConnectionState::Established,
            created_at: now,
            last_seen: now,
            packet_count: 1,
            byte_count: packet.payload_size as u64,
        };

        self.connections.insert(key, connection);
    }
}

/// Firewall statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FirewallStatistics {
    pub total_rules: usize,
    pub enabled_rules: usize,
    pub packets_processed: u64,
    pub packets_allowed: u64,
    pub packets_denied: u64,
    pub active_connections: usize,
}

impl Default for FirewallRulesManager {
    fn default() -> Self {
        Self::new(RuleAction::Deny)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firewall_rule_creation() {
        let rule = FirewallRule::new(
            "rule1".to_string(),
            100,
            IpRange::Any,
            IpRange::Single(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
            PortRange::any(),
            PortRange::single(80),
            Protocol::TCP,
            RuleAction::Allow,
            "Allow HTTP traffic".to_string(),
        );

        assert_eq!(rule.id, "rule1");
        assert_eq!(rule.priority, 100);
        assert!(rule.enabled);
    }

    #[test]
    fn test_firewall_manager_add_rule() {
        let mut manager = FirewallRulesManager::new(RuleAction::Deny);
        let rule = FirewallRule::new(
            "rule1".to_string(),
            100,
            IpRange::Any,
            IpRange::Any,
            PortRange::any(),
            PortRange::single(80),
            Protocol::TCP,
            RuleAction::Allow,
            "Allow HTTP".to_string(),
        );

        assert!(manager.add_rule(rule).is_ok());
        assert_eq!(manager.get_statistics().total_rules, 1);
    }

    #[test]
    fn test_firewall_packet_matching() {
        let rule = FirewallRule::new(
            "rule1".to_string(),
            100,
            IpRange::Any,
            IpRange::Single(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
            PortRange::any(),
            PortRange::single(80),
            Protocol::TCP,
            RuleAction::Allow,
            "Allow HTTP".to_string(),
        );

        let packet = NetworkPacket {
            source_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            destination_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            source_port: 12345,
            destination_port: 80,
            protocol: Protocol::TCP,
            payload_size: 1024,
        };

        assert!(rule.matches(&packet));
    }
}
