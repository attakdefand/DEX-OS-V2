//! Perimeter Defense System - Central coordinator for network security
//!
//! Implements Security Layer 6 - Network & Infrastructure Security (Perimeter Defense)
//! From DEX-OS-V2.csv line 240:
//! - Security,Security Layer,Security Layer 6,Network & Infrastructure Security,Perimeter Defense,High

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

use super::ddos_protection::{DDoSProtectionManager, DDoSStatistics};
use super::firewall::{FirewallRulesManager, FirewallStatistics, NetworkPacket, RuleAction};
use super::ids_ips::{IntrusionPreventionSystem, IPSStatistics};
use super::ip_routing::{IPRoutingTable, RoutingStatistics};
use super::service_mesh::{ServiceMeshManager, ServiceMeshStatistics};
use super::traffic_monitor::{TrafficMonitor, TrafficStatistics};

/// Security zone types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecurityZone {
    /// External/Internet zone
    External,
    /// DMZ (Demilitarized Zone)
    DMZ,
    /// Internal/Trusted zone
    Internal,
    /// Management zone
    Management,
    /// Custom zone
    Custom(String),
}

/// Network segment definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkSegment {
    /// Segment ID
    pub id: String,
    /// Segment name
    pub name: String,
    /// Security zone
    pub zone: SecurityZone,
    /// Network CIDR
    pub network_cidr: String,
    /// Allowed zones for communication
    pub allowed_zones: Vec<SecurityZone>,
    /// Description
    pub description: String,
}

/// VPN Gateway configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VPNGateway {
    /// Gateway ID
    pub id: String,
    /// Gateway name
    pub name: String,
    /// Gateway IP
    pub gateway_ip: IpAddr,
    /// Encryption enabled
    pub encryption_enabled: bool,
    /// Active tunnels
    pub active_tunnels: u32,
    /// Maximum tunnels
    pub max_tunnels: u32,
}

/// Perimeter Defense Manager - Central coordinator for all network security features
#[derive(Debug, Clone)]
pub struct PerimeterDefenseManager {
    /// Firewall rules manager
    firewall: FirewallRulesManager,
    /// IP routing table
    routing: IPRoutingTable,
    /// Service mesh manager
    service_mesh: ServiceMeshManager,
    /// DDoS protection
    ddos_protection: DDoSProtectionManager,
    /// Intrusion Prevention System
    ips: IntrusionPreventionSystem,
    /// Traffic monitor
    traffic_monitor: TrafficMonitor,
    /// Network segments
    segments: Vec<NetworkSegment>,
    /// VPN gateways
    vpn_gateways: Vec<VPNGateway>,
    /// Defense enabled
    defense_enabled: bool,
}

impl PerimeterDefenseManager {
    /// Create a new perimeter defense manager
    pub fn new() -> Self {
        Self {
            firewall: FirewallRulesManager::new(RuleAction::Deny),
            routing: IPRoutingTable::new(),
            service_mesh: ServiceMeshManager::new(),
            ddos_protection: DDoSProtectionManager::new(),
            ips: IntrusionPreventionSystem::new(),
            traffic_monitor: TrafficMonitor::new(10000, 300),
            segments: Vec::new(),
            vpn_gateways: Vec::new(),
            defense_enabled: true,
        }
    }

    /// Process an incoming packet through all security layers
    pub fn process_packet(&mut self, packet: &NetworkPacket, payload: &[u8]) -> Result<RuleAction, String> {
        if !self.defense_enabled {
            return Ok(RuleAction::Allow);
        }

        // 1. DDoS Protection
        if let Err(e) = self.ddos_protection.check_request(&packet.source_ip, packet.payload_size as u64) {
            return Err(format!("DDoS protection: {}", e));
        }

        // 2. IPS - Intrusion Prevention
        let payload_str = String::from_utf8_lossy(payload);
        if let Err(e) = self.ips.inspect_and_prevent(packet.source_ip, &payload_str) {
            return Err(format!("IPS blocked: {}", e));
        }

        // 3. Firewall Rules
        let action = self.firewall.process_packet(packet);
        if matches!(action, RuleAction::Deny | RuleAction::LogAndDeny) {
            return Err("Firewall blocked".to_string());
        }

        // 4. Traffic Monitoring
        self.traffic_monitor.record_packet(
            packet.source_ip,
            packet.destination_ip,
            packet.source_port,
            packet.destination_port,
            packet.protocol.clone(),
            packet.payload_size as u64,
        );

        // 5. Packet Inspection
        let inspection = self.traffic_monitor.inspect_packet(
            packet.source_ip,
            packet.destination_ip,
            packet.protocol.clone(),
            packet.payload_size as u64,
            payload,
        );

        if inspection.suspicious {
            return Err(format!("Suspicious packet detected: {:?}", inspection.flags));
        }

        Ok(action)
    }

    /// Add a network segment
    pub fn add_segment(&mut self, segment: NetworkSegment) {
        self.segments.push(segment);
    }

    /// Get network segments
    pub fn get_segments(&self) -> &[NetworkSegment] {
        &self.segments
    }

    /// Add a VPN gateway
    pub fn add_vpn_gateway(&mut self, gateway: VPNGateway) {
        self.vpn_gateways.push(gateway);
    }

    /// Get VPN gateways
    pub fn get_vpn_gateways(&self) -> &[VPNGateway] {
        &self.vpn_gateways
    }

    /// Enable/disable perimeter defense
    pub fn set_defense_enabled(&mut self, enabled: bool) {
        self.defense_enabled = enabled;
    }

    /// Check if defense is enabled
    pub fn is_defense_enabled(&self) -> bool {
        self.defense_enabled
    }

    /// Get firewall manager
    pub fn get_firewall(&self) -> &FirewallRulesManager {
        &self.firewall
    }

    /// Get mutable firewall manager
    pub fn get_firewall_mut(&mut self) -> &mut FirewallRulesManager {
        &mut self.firewall
    }

    /// Get routing table
    pub fn get_routing(&self) -> &IPRoutingTable {
        &self.routing
    }

    /// Get mutable routing table
    pub fn get_routing_mut(&mut self) -> &mut IPRoutingTable {
        &mut self.routing
    }

    /// Get service mesh
    pub fn get_service_mesh(&self) -> &ServiceMeshManager {
        &self.service_mesh
    }

    /// Get mutable service mesh
    pub fn get_service_mesh_mut(&mut self) -> &mut ServiceMeshManager {
        &mut self.service_mesh
    }

    /// Get DDoS protection
    pub fn get_ddos_protection(&self) -> &DDoSProtectionManager {
        &self.ddos_protection
    }

    /// Get mutable DDoS protection
    pub fn get_ddos_protection_mut(&mut self) -> &mut DDoSProtectionManager {
        &mut self.ddos_protection
    }

    /// Get IPS
    pub fn get_ips(&self) -> &IntrusionPreventionSystem {
        &self.ips
    }

    /// Get mutable IPS
    pub fn get_ips_mut(&mut self) -> &mut IntrusionPreventionSystem {
        &mut self.ips
    }

    /// Get traffic monitor
    pub fn get_traffic_monitor(&self) -> &TrafficMonitor {
        &self.traffic_monitor
    }

    /// Get mutable traffic monitor
    pub fn get_traffic_monitor_mut(&mut self) -> &mut TrafficMonitor {
        &mut self.traffic_monitor
    }

    /// Get comprehensive security statistics
    pub fn get_statistics(&self) -> PerimeterDefenseStatistics {
        PerimeterDefenseStatistics {
            firewall: self.firewall.get_statistics(),
            routing: self.routing.get_statistics(),
            service_mesh: self.service_mesh.get_statistics(),
            ddos_protection: self.ddos_protection.get_statistics(),
            ips: self.ips.get_statistics(),
            traffic: self.traffic_monitor.get_statistics(),
            network_segments: self.segments.len(),
            vpn_gateways: self.vpn_gateways.len(),
            defense_enabled: self.defense_enabled,
        }
    }

    /// Perform maintenance tasks
    pub fn perform_maintenance(&mut self) {
        self.firewall.cleanup_connections(300); // 5 minute timeout
        self.ddos_protection.cleanup();
        self.ips.cleanup_blocked_ips();
        self.traffic_monitor.cleanup_old_flows();
    }
}

/// Comprehensive perimeter defense statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerimeterDefenseStatistics {
    pub firewall: FirewallStatistics,
    pub routing: RoutingStatistics,
    pub service_mesh: ServiceMeshStatistics,
    pub ddos_protection: DDoSStatistics,
    pub ips: IPSStatistics,
    pub traffic: TrafficStatistics,
    pub network_segments: usize,
    pub vpn_gateways: usize,
    pub defense_enabled: bool,
}

impl Default for PerimeterDefenseManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::firewall::Protocol;
    use std::net::Ipv4Addr;

    #[test]
    fn test_perimeter_defense_creation() {
        let manager = PerimeterDefenseManager::new();
        assert!(manager.is_defense_enabled());
    }

    #[test]
    fn test_packet_processing() {
        let mut manager = PerimeterDefenseManager::new();
        
        let packet = NetworkPacket {
            source_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            destination_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            source_port: 12345,
            destination_port: 80,
            protocol: Protocol::TCP,
            payload_size: 1024,
        };

        let payload = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        
        // Should be blocked by default deny firewall
        let result = manager.process_packet(&packet, payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_network_segmentation() {
        let mut manager = PerimeterDefenseManager::new();
        
        let segment = NetworkSegment {
            id: "seg1".to_string(),
            name: "DMZ".to_string(),
            zone: SecurityZone::DMZ,
            network_cidr: "10.0.1.0/24".to_string(),
            allowed_zones: vec![SecurityZone::External, SecurityZone::Internal],
            description: "DMZ segment".to_string(),
        };

        manager.add_segment(segment);
        assert_eq!(manager.get_segments().len(), 1);
    }

    #[test]
    fn test_comprehensive_statistics() {
        let manager = PerimeterDefenseManager::new();
        let stats = manager.get_statistics();
        
        assert!(stats.defense_enabled);
        assert_eq!(stats.network_segments, 0);
        assert_eq!(stats.vpn_gateways, 0);
    }
}
