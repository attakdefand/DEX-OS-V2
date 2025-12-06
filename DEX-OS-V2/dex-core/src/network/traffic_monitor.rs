//! Traffic Monitoring implementation for DEX-OS Network Security
//!
//! Implements Security Layer 6 - Network & Infrastructure Security (Perimeter Defense)
//! Provides real-time traffic analysis, packet inspection, flow tracking, and protocol analysis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

use super::firewall::Protocol;

/// Network flow information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkFlow {
    /// Flow ID
    pub id: String,
    /// Source IP
    pub source_ip: IpAddr,
    /// Destination IP
    pub destination_ip: IpAddr,
    /// Source port
    pub source_port: u16,
    /// Destination port
    pub destination_port: u16,
    /// Protocol
    pub protocol: Protocol,
    /// Total packets
    pub packet_count: u64,
    /// Total bytes
    pub byte_count: u64,
    /// Flow start time
    pub start_time: u64,
    /// Flow last seen time
    pub last_seen: u64,
    /// Flow duration in seconds
    pub duration: u64,
}

impl NetworkFlow {
    pub fn new(
        source_ip: IpAddr,
        destination_ip: IpAddr,
        source_port: u16,
        destination_port: u16,
        protocol: Protocol,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = format!(
            "{}:{}->{}:{}:{:?}",
            source_ip, source_port, destination_ip, destination_port, protocol
        );

        Self {
            id,
            source_ip,
            destination_ip,
            source_port,
            destination_port,
            protocol,
            packet_count: 0,
            byte_count: 0,
            start_time: now,
            last_seen: now,
            duration: 0,
        }
    }

    pub fn update(&mut self, bytes: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.packet_count += 1;
        self.byte_count += bytes;
        self.last_seen = now;
        self.duration = now - self.start_time;
    }
}

/// Packet inspection details
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PacketInspection {
    /// Inspection ID
    pub id: String,
    /// Source IP
    pub source_ip: IpAddr,
    /// Destination IP
    pub destination_ip: IpAddr,
    /// Protocol
    pub protocol: Protocol,
    /// Packet size
    pub size: u64,
    /// Inspection timestamp
    pub timestamp: u64,
    /// Flags detected
    pub flags: Vec<String>,
    /// Suspicious indicators
    pub suspicious: bool,
}

/// Bandwidth usage per IP
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BandwidthUsage {
    /// IP address
    pub ip: IpAddr,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Packets sent
    pub packets_sent: u64,
    /// Packets received
    pub packets_received: u64,
    /// Window start time
    pub window_start: u64,
}

impl BandwidthUsage {
    pub fn new(ip: IpAddr) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            ip,
            bytes_sent: 0,
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
            window_start: now,
        }
    }

    pub fn total_bytes(&self) -> u64 {
        self.bytes_sent + self.bytes_received
    }

    pub fn total_packets(&self) -> u64 {
        self.packets_sent + self.packets_received
    }
}

/// Protocol statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolStats {
    /// Protocol type
    pub protocol: Protocol,
    /// Packet count
    pub packet_count: u64,
    /// Byte count
    pub byte_count: u64,
    /// Unique source IPs
    pub unique_sources: usize,
    /// Unique destination IPs
    pub unique_destinations: usize,
}

/// Traffic Monitor for real-time network analysis
#[derive(Debug, Clone)]
pub struct TrafficMonitor {
    /// Active network flows
    flows: HashMap<String, NetworkFlow>,
    /// Packet inspections (limited history)
    inspections: Vec<PacketInspection>,
    /// Bandwidth usage per IP
    bandwidth_usage: HashMap<IpAddr, BandwidthUsage>,
    /// Protocol statistics
    protocol_stats: HashMap<String, ProtocolStats>,
    /// Maximum inspections to keep
    max_inspections: usize,
    /// Flow timeout in seconds
    flow_timeout: u64,
    /// Total packets processed
    total_packets: u64,
    /// Total bytes processed
    total_bytes: u64,
}

impl TrafficMonitor {
    /// Create a new traffic monitor
    pub fn new(max_inspections: usize, flow_timeout: u64) -> Self {
        Self {
            flows: HashMap::new(),
            inspections: Vec::new(),
            bandwidth_usage: HashMap::new(),
            protocol_stats: HashMap::new(),
            max_inspections,
            flow_timeout,
            total_packets: 0,
            total_bytes: 0,
        }
    }

    /// Record a packet
    pub fn record_packet(
        &mut self,
        source_ip: IpAddr,
        destination_ip: IpAddr,
        source_port: u16,
        destination_port: u16,
        protocol: Protocol,
        size: u64,
    ) {
        self.total_packets += 1;
        self.total_bytes += size;

        // Update flow
        let flow_id = format!(
            "{}:{}->{}:{}:{:?}",
            source_ip, source_port, destination_ip, destination_port, protocol
        );

        let flow = self
            .flows
            .entry(flow_id.clone())
            .or_insert_with(|| NetworkFlow::new(source_ip, destination_ip, source_port, destination_port, protocol.clone()));

        flow.update(size);

        // Update bandwidth usage
        let source_usage = self
            .bandwidth_usage
            .entry(source_ip)
            .or_insert_with(|| BandwidthUsage::new(source_ip));
        source_usage.bytes_sent += size;
        source_usage.packets_sent += 1;

        let dest_usage = self
            .bandwidth_usage
            .entry(destination_ip)
            .or_insert_with(|| BandwidthUsage::new(destination_ip));
        dest_usage.bytes_received += size;
        dest_usage.packets_received += 1;

        // Update protocol statistics
        let protocol_key = format!("{:?}", protocol);
        let stats = self
            .protocol_stats
            .entry(protocol_key)
            .or_insert_with(|| ProtocolStats {
                protocol: protocol.clone(),
                packet_count: 0,
                byte_count: 0,
                unique_sources: 0,
                unique_destinations: 0,
            });

        stats.packet_count += 1;
        stats.byte_count += size;
    }

    /// Inspect a packet for suspicious activity
    pub fn inspect_packet(
        &mut self,
        source_ip: IpAddr,
        destination_ip: IpAddr,
        protocol: Protocol,
        size: u64,
        payload: &[u8],
    ) -> PacketInspection {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut flags = Vec::new();
        let mut suspicious = false;

        // Check for suspicious patterns
        if size > 65535 {
            flags.push("OVERSIZED_PACKET".to_string());
            suspicious = true;
        }

        if size < 20 {
            flags.push("UNDERSIZED_PACKET".to_string());
            suspicious = true;
        }

        // Check payload for suspicious content (simplified)
        if payload.len() > 0 {
            let payload_str = String::from_utf8_lossy(payload);
            if payload_str.contains("../") {
                flags.push("PATH_TRAVERSAL".to_string());
                suspicious = true;
            }
            if payload_str.contains("<script>") {
                flags.push("XSS_ATTEMPT".to_string());
                suspicious = true;
            }
        }

        let inspection = PacketInspection {
            id: format!("insp_{}_{}", source_ip, now),
            source_ip,
            destination_ip,
            protocol,
            size,
            timestamp: now,
            flags,
            suspicious,
        };

        // Store inspection (with limit)
        self.inspections.push(inspection.clone());
        if self.inspections.len() > self.max_inspections {
            self.inspections.remove(0);
        }

        inspection
    }

    /// Get active flows
    pub fn get_active_flows(&self) -> Vec<&NetworkFlow> {
        self.flows.values().collect()
    }

    /// Get flows for a specific IP
    pub fn get_flows_for_ip(&self, ip: &IpAddr) -> Vec<&NetworkFlow> {
        self.flows
            .values()
            .filter(|f| &f.source_ip == ip || &f.destination_ip == ip)
            .collect()
    }

    /// Get bandwidth usage for an IP
    pub fn get_bandwidth_usage(&self, ip: &IpAddr) -> Option<&BandwidthUsage> {
        self.bandwidth_usage.get(ip)
    }

    /// Get all bandwidth usage
    pub fn get_all_bandwidth_usage(&self) -> Vec<&BandwidthUsage> {
        self.bandwidth_usage.values().collect()
    }

    /// Get top bandwidth consumers
    pub fn get_top_bandwidth_consumers(&self, limit: usize) -> Vec<&BandwidthUsage> {
        let mut usage: Vec<&BandwidthUsage> = self.bandwidth_usage.values().collect();
        // Sort by bytes sent only, not total bytes
        usage.sort_by(|a, b| b.bytes_sent.cmp(&a.bytes_sent));
        usage.into_iter().take(limit).collect()
    }

    /// Get protocol statistics
    pub fn get_protocol_stats(&self) -> Vec<&ProtocolStats> {
        self.protocol_stats.values().collect()
    }

    /// Get suspicious inspections
    pub fn get_suspicious_inspections(&self) -> Vec<&PacketInspection> {
        self.inspections.iter().filter(|i| i.suspicious).collect()
    }

    /// Get recent inspections
    pub fn get_recent_inspections(&self, limit: usize) -> Vec<&PacketInspection> {
        let start = if self.inspections.len() > limit {
            self.inspections.len() - limit
        } else {
            0
        };
        self.inspections[start..].iter().collect()
    }

    /// Get traffic statistics
    pub fn get_statistics(&self) -> TrafficStatistics {
        TrafficStatistics {
            total_packets: self.total_packets,
            total_bytes: self.total_bytes,
            active_flows: self.flows.len(),
            unique_ips: self.bandwidth_usage.len(),
            total_inspections: self.inspections.len(),
            suspicious_packets: self.inspections.iter().filter(|i| i.suspicious).count(),
        }
    }

    /// Cleanup old flows
    pub fn cleanup_old_flows(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.flows
            .retain(|_, flow| now - flow.last_seen < self.flow_timeout);
    }

    /// Reset bandwidth usage counters
    pub fn reset_bandwidth_counters(&mut self) {
        self.bandwidth_usage.clear();
    }
}

/// Traffic statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrafficStatistics {
    pub total_packets: u64,
    pub total_bytes: u64,
    pub active_flows: usize,
    pub unique_ips: usize,
    pub total_inspections: usize,
    pub suspicious_packets: usize,
}

impl Default for TrafficMonitor {
    fn default() -> Self {
        Self::new(10000, 300) // Keep 10k inspections, 5min flow timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_traffic_monitor_creation() {
        let monitor = TrafficMonitor::new(1000, 300);
        assert_eq!(monitor.max_inspections, 1000);
        assert_eq!(monitor.flow_timeout, 300);
    }

    #[test]
    fn test_record_packet() {
        let mut monitor = TrafficMonitor::new(1000, 300);
        let source = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let dest = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

        monitor.record_packet(source, dest, 12345, 80, Protocol::TCP, 1024);

        let stats = monitor.get_statistics();
        assert_eq!(stats.total_packets, 1);
        assert_eq!(stats.total_bytes, 1024);
        assert_eq!(stats.active_flows, 1);
    }

    #[test]
    fn test_bandwidth_tracking() {
        let mut monitor = TrafficMonitor::new(1000, 300);
        let source = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let dest = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

        monitor.record_packet(source, dest, 12345, 80, Protocol::TCP, 1024);
        monitor.record_packet(source, dest, 12345, 80, Protocol::TCP, 2048);

        let usage = monitor.get_bandwidth_usage(&source).unwrap();
        assert_eq!(usage.bytes_sent, 3072);
        assert_eq!(usage.packets_sent, 2);
    }

    #[test]
    fn test_packet_inspection() {
        let mut monitor = TrafficMonitor::new(1000, 300);
        let source = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let dest = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

        let payload = b"<script>alert('xss')</script>";
        let inspection = monitor.inspect_packet(source, dest, Protocol::TCP, 1024, payload);

        assert!(inspection.suspicious);
        assert!(inspection.flags.contains(&"XSS_ATTEMPT".to_string()));
    }

    #[test]
    fn test_top_bandwidth_consumers() {
        let mut monitor = TrafficMonitor::new(1000, 300);
        let ip1 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ip2 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2));
        let dest = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

        monitor.record_packet(ip1, dest, 12345, 80, Protocol::TCP, 5000);
        monitor.record_packet(ip2, dest, 12346, 80, Protocol::TCP, 10000);

        let top = monitor.get_top_bandwidth_consumers(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].ip, ip2);
    }
}
