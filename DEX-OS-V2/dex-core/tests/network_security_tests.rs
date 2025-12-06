//! Comprehensive test suite for Security Layer 6 - Network & Infrastructure Security
//!
//! Tests all components of the perimeter defense system including:
//! - Firewall rules management and packet filtering
//! - IP routing with Trie-based longest prefix matching
//! - Service mesh with load balancing and circuit breaker
//! - DDoS protection with rate limiting and SYN flood protection
//! - IDS/IPS with signature-based and anomaly detection
//! - Traffic monitoring and analysis
//! - Perimeter defense coordinator

use dex_core::network::{
    FirewallRule, FirewallRulesManager, IpRange, NetworkPacket, PortRange, Protocol, RuleAction,
    IPRoutingTable, RouteEntry,
    ServiceMeshManager, Service, ServiceEndpoint, LoadBalancingStrategy, HealthStatus, CircuitBreaker, CircuitBreakerState,
    DDoSProtectionManager,
    IntrusionDetectionSystem, IntrusionPreventionSystem, AttackType, ThreatResponse, ThreatSeverity, ThreatSignature,
    TrafficMonitor, BandwidthUsage, PacketInspection,
    PerimeterDefenseManager, NetworkSegment, SecurityZone, VPNGateway, PerimeterDefenseStatistics
};
use std::net::{IpAddr, Ipv4Addr};// ============================================================================// FIREWALL TESTS
// ============================================================================

#[test]
fn test_firewall_rule_creation_and_matching() {
    let rule = FirewallRule::new(
        "allow_http".to_string(),
        100,
        IpRange::Any,
        IpRange::Single(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
        PortRange::any(),
        PortRange::single(80),
        Protocol::TCP,
        RuleAction::Allow,
        "Allow HTTP traffic to web server".to_string(),
    );

    let packet = NetworkPacket {
        source_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        destination_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        source_port: 54321,
        destination_port: 80,
        protocol: Protocol::TCP,
        payload_size: 1024,
    };

    assert!(rule.matches(&packet));
    assert_eq!(rule.action, RuleAction::Allow);
}

#[test]
fn test_firewall_manager_rule_priority() {
    let mut manager = FirewallRulesManager::new(RuleAction::Deny);

    // Add low priority rule (allow all)
    let rule1 = FirewallRule::new(
        "allow_all".to_string(),
        1000,
        IpRange::Any,
        IpRange::Any,
        PortRange::any(),
        PortRange::any(),
        Protocol::Any,
        RuleAction::Allow,
        "Allow all traffic".to_string(),
    );

    // Add high priority rule (deny specific IP)
    let rule2 = FirewallRule::new(
        "deny_specific".to_string(),
        10,
        IpRange::Single(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))),
        IpRange::Any,
        PortRange::any(),
        PortRange::any(),
        Protocol::Any,
        RuleAction::Deny,
        "Deny specific IP".to_string(),
    );

    manager.add_rule(rule1).unwrap();
    manager.add_rule(rule2).unwrap();

    // Packet from blocked IP should be denied (high priority rule)
    let blocked_packet = NetworkPacket {
        source_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        destination_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        source_port: 12345,
        destination_port: 80,
        protocol: Protocol::TCP,
        payload_size: 1024,
    };

    let action = manager.process_packet(&blocked_packet);
    assert_eq!(action, RuleAction::Deny);

    // Packet from other IP should be allowed (low priority rule)
    let allowed_packet = NetworkPacket {
        source_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        destination_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        source_port: 12345,
        destination_port: 80,
        protocol: Protocol::TCP,
        payload_size: 1024,
    };

    let action = manager.process_packet(&allowed_packet);
    assert_eq!(action, RuleAction::Allow);
}

#[test]
fn test_firewall_stateful_connection_tracking() {
    let mut manager = FirewallRulesManager::new(RuleAction::Allow);

    let packet = NetworkPacket {
        source_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        destination_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        source_port: 12345,
        destination_port: 80,
        protocol: Protocol::TCP,
        payload_size: 1024,
    };

    // First packet establishes connection
    manager.process_packet(&packet);

    // Second packet should use established connection
    manager.process_packet(&packet);

    let stats = manager.get_statistics();
    assert_eq!(stats.packets_processed, 2);
    assert_eq!(stats.active_connections, 1);
}

#[test]
fn test_firewall_subnet_matching() {
    let rule = FirewallRule::new(
        "allow_subnet".to_string(),
        100,
        IpRange::Subnet {
            network: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)),
            prefix_len: 24,
        },
        IpRange::Any,
        PortRange::any(),
        PortRange::any(),
        Protocol::Any,
        RuleAction::Allow,
        "Allow subnet".to_string(),
    );

    // IP in subnet should match
    let packet1 = NetworkPacket {
        source_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
        destination_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        source_port: 12345,
        destination_port: 80,
        protocol: Protocol::TCP,
        payload_size: 1024,
    };
    assert!(rule.matches(&packet1));

    // IP outside subnet should not match
    let packet2 = NetworkPacket {
        source_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 2, 100)),
        destination_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        source_port: 12345,
        destination_port: 80,
        protocol: Protocol::TCP,
        payload_size: 1024,
    };
    assert!(!rule.matches(&packet2));
}

// ============================================================================
// IP ROUTING TESTS
// ============================================================================

#[test]
fn test_ip_routing_add_and_lookup() {
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

    table.add_route(route).unwrap();

    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50));
    let result = table.lookup(&ip);

    assert!(result.is_some());
    let found_route = result.unwrap();
    assert_eq!(found_route.interface, "eth0");
}

#[test]
fn test_ip_routing_longest_prefix_match() {
    let mut table = IPRoutingTable::new();

    // Add /16 route
    let route1 = RouteEntry::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0)),
        16,
        IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)),
        10,
        100,
        "eth0".to_string(),
        "Wide network".to_string(),
    )
    .unwrap();

    // Add more specific /24 route
    let route2 = RouteEntry::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)),
        24,
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        5,
        200,
        "eth1".to_string(),
        "Specific network".to_string(),
    )
    .unwrap();

    table.add_route(route1).unwrap();
    table.add_route(route2).unwrap();

    // Lookup should return the more specific route
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50));
    let result = table.lookup(&ip).unwrap();
    assert_eq!(result.prefix_len, 24);
    assert_eq!(result.interface, "eth1");
}

#[test]
fn test_ip_routing_remove_route() {
    let mut table = IPRoutingTable::new();

    let route = RouteEntry::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)),
        24,
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        10,
        100,
        "eth0".to_string(),
        "Test route".to_string(),
    )
    .unwrap();

    table.add_route(route).unwrap();
    assert_eq!(table.get_statistics().total_routes, 1);

    table
        .remove_route(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)), 24)
        .unwrap();
    assert_eq!(table.get_statistics().total_routes, 0);
}

// ============================================================================
// SERVICE MESH TESTS
// ============================================================================

#[test]
fn test_service_mesh_registration() {
    let mut mesh = ServiceMeshManager::new();

    let service = Service::new(
        "api-service".to_string(),
        "API Gateway Service".to_string(),
        LoadBalancingStrategy::RoundRobin,
    );

    assert!(mesh.register_service(service).is_ok());
    assert_eq!(mesh.get_statistics().total_services, 1);
}

#[test]
fn test_service_mesh_endpoint_selection() {
    let mut mesh = ServiceMeshManager::new();

    let mut service = Service::new(
        "api-service".to_string(),
        "API Gateway".to_string(),
        LoadBalancingStrategy::RoundRobin,
    );

    // Add endpoints
    let endpoint1 = ServiceEndpoint::new(
        "ep1".to_string(),
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        8080,
        1,
    );
    let mut endpoint1_healthy = endpoint1.clone();
    endpoint1_healthy.health = HealthStatus::Healthy;

    let endpoint2 = ServiceEndpoint::new(
        "ep2".to_string(),
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        8080,
        1,
    );
    let mut endpoint2_healthy = endpoint2.clone();
    endpoint2_healthy.health = HealthStatus::Healthy;

    service.add_endpoint(endpoint1_healthy);
    service.add_endpoint(endpoint2_healthy);

    mesh.register_service(service).unwrap();

    // Select endpoint should work
    let endpoint = mesh.select_endpoint("api-service");
    assert!(endpoint.is_ok());
}

#[test]
fn test_circuit_breaker_opens_on_failures() {
    let mut cb = CircuitBreaker::new(3, 2, 60);

    assert_eq!(cb.state, CircuitBreakerState::Closed);

    // Record failures
    cb.record_failure();
    cb.record_failure();
    cb.record_failure();

    assert_eq!(cb.state, CircuitBreakerState::Open);
    assert!(!cb.can_attempt());
}

#[test]
fn test_circuit_breaker_half_open_recovery() {
    let mut cb = CircuitBreaker::new(2, 2, 1); // 1 second timeout

    // Open the circuit
    cb.record_failure();
    cb.record_failure();
    assert_eq!(cb.state, CircuitBreakerState::Open);

    // Wait for timeout (simulate)
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Should transition to half-open
    assert!(cb.can_attempt());
    assert_eq!(cb.state, CircuitBreakerState::HalfOpen);

    // Record successes to close
    cb.record_success();
    cb.record_success();
    assert_eq!(cb.state, CircuitBreakerState::Closed);
}

// ============================================================================
// DDOS PROTECTION TESTS
// ============================================================================

#[test]
fn test_ddos_rate_limiting() {
    let mut manager = DDoSProtectionManager::with_config(5, 10, 10, 1_000_000);
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

    // Should allow first 5 requests
    for _ in 0..5 {
        assert!(manager.check_request(&ip, 1024).is_ok());
    }

    // Should block 6th request
    assert!(manager.check_request(&ip, 1024).is_err());

    let stats = manager.get_statistics();
    assert_eq!(stats.total_requests, 6);
    assert_eq!(stats.blocked_requests, 1);
}

#[test]
fn test_ddos_syn_flood_protection() {
    let mut manager = DDoSProtectionManager::with_config(100, 3, 100, 10_000_000);
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

    // Should allow first 3 SYN packets
    for _ in 0..3 {
        assert!(manager.check_syn_packet(&ip).is_ok());
    }

    // Should detect SYN flood on 4th packet
    assert!(manager.check_syn_packet(&ip).is_err());
    assert!(manager.is_blocked(&ip));
}

#[test]
fn test_ddos_connection_throttling() {
    let mut manager = DDoSProtectionManager::with_config(100, 10, 2, 10_000_000);
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

    // Should allow 2 connections
    assert!(manager.check_new_connection(&ip).is_ok());
    assert!(manager.check_new_connection(&ip).is_ok());

    // Should block 3rd connection
    assert!(manager.check_new_connection(&ip).is_err());

    // Release one connection
    manager.release_connection(&ip);

    // Should allow another connection
    assert!(manager.check_new_connection(&ip).is_ok());
}

#[test]
fn test_ddos_geo_blocking() {
    let manager = DDoSProtectionManager::new();

    // Default config allows all countries
    assert!(manager.check_geo_location("US").is_ok());
    assert!(manager.check_geo_location("CN").is_ok());
}

// ============================================================================
// IDS/IPS TESTS
// ============================================================================

#[test]
fn test_ids_sql_injection_detection() {
    let mut ids = IntrusionDetectionSystem::new();
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

    let threat = ids.inspect_data(ip, "SELECT * UNION SELECT * FROM passwords");
    assert!(threat.is_some());

    let detected = threat.unwrap();
    assert_eq!(detected.attack_type, AttackType::SqlInjection);
    assert_eq!(detected.severity, ThreatSeverity::High);
}

#[test]
fn test_ids_xss_detection() {
    let mut ids = IntrusionDetectionSystem::new();
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

    let threat = ids.inspect_data(ip, "<script>alert('xss')</script>");
    assert!(threat.is_some());

    let detected = threat.unwrap();
    assert_eq!(detected.attack_type, AttackType::XssAttack);
}

#[test]
fn test_ips_automatic_blocking() {
    let mut ips = IntrusionPreventionSystem::new();
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

    // Should detect and block SQL injection
    let result = ips.inspect_and_prevent(ip, "SELECT * UNION SELECT * FROM passwords");
    assert!(result.is_err());
    assert!(ips.is_blocked(&ip));

    // Subsequent requests should be blocked
    let result = ips.inspect_and_prevent(ip, "SELECT * FROM users");
    assert!(result.is_err());

    let stats = ips.get_statistics();
    assert_eq!(stats.threats_blocked, 1);
}

#[test]
fn test_ids_custom_signature() {
    let mut ids = IntrusionDetectionSystem::new();

    let custom_sig = ThreatSignature::new(
        "custom_001".to_string(),
        "Custom Attack Pattern".to_string(),
        AttackType::SuspiciousActivity,
        "MALICIOUS_PATTERN".to_string(),
        ThreatSeverity::Critical,
        ThreatResponse::BlockAndAlert,
        "Custom attack pattern".to_string(),
    );

    ids.add_signature(custom_sig).unwrap();

    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let threat = ids.inspect_data(ip, "This contains MALICIOUS_PATTERN in it");

    assert!(threat.is_some());
    let detected = threat.unwrap();
    assert_eq!(detected.severity, ThreatSeverity::Critical);
}

// ============================================================================
// TRAFFIC MONITOR TESTS
// ============================================================================

#[test]
fn test_traffic_monitor_packet_recording() {
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
fn test_traffic_monitor_bandwidth_tracking() {
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
fn test_traffic_monitor_suspicious_packet_detection() {
    let mut monitor = TrafficMonitor::new(1000, 300);
    let source = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let dest = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

    let payload = b"<script>alert('xss')</script>";
    let inspection = monitor.inspect_packet(source, dest, Protocol::TCP, 1024, payload);

    assert!(inspection.suspicious);
    assert!(inspection.flags.contains(&"XSS_ATTEMPT".to_string()));
}

#[test]
fn test_traffic_monitor_top_consumers() {
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

// ============================================================================
// PERIMETER DEFENSE INTEGRATION TESTS
// ============================================================================

#[test]
fn test_perimeter_defense_multi_layer_protection() {
    let mut manager = PerimeterDefenseManager::new();

    // Add allow rule to firewall
    let rule = FirewallRule::new(
        "allow_http".to_string(),
        100,
        IpRange::Any,
        IpRange::Any,
        PortRange::any(),
        PortRange::single(80),
        Protocol::TCP,
        RuleAction::Allow,
        "Allow HTTP".to_string(),
    );
    manager.get_firewall_mut().add_rule(rule).unwrap();

    let packet = NetworkPacket {
        source_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        destination_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        source_port: 12345,
        destination_port: 80,
        protocol: Protocol::TCP,
        payload_size: 1024,
    };

    let payload = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";

    // Should pass all security layers
    let result = manager.process_packet(&packet, payload);
    assert!(result.is_ok());
}

#[test]
fn test_perimeter_defense_blocks_malicious_traffic() {
    let mut manager = PerimeterDefenseManager::new();

    // Add allow rule to firewall
    let rule = FirewallRule::new(
        "allow_all".to_string(),
        100,
        IpRange::Any,
        IpRange::Any,
        PortRange::any(),
        PortRange::any(),
        Protocol::Any,
        RuleAction::Allow,
        "Allow all".to_string(),
    );
    manager.get_firewall_mut().add_rule(rule).unwrap();

    let packet = NetworkPacket {
        source_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        destination_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        source_port: 12345,
        destination_port: 80,
        protocol: Protocol::TCP,
        payload_size: 1024,
    };

    // SQL injection payload
    let payload = b"SELECT * UNION SELECT * FROM passwords";

    // Should be blocked by IPS
    let result = manager.process_packet(&packet, payload);
    assert!(result.is_err());
}

#[test]
fn test_perimeter_defense_network_segmentation() {
    let mut manager = PerimeterDefenseManager::new();

    let segment = NetworkSegment {
        id: "dmz".to_string(),
        name: "DMZ Zone".to_string(),
        zone: SecurityZone::DMZ,
        network_cidr: "10.0.1.0/24".to_string(),
        allowed_zones: vec![SecurityZone::External, SecurityZone::Internal],
        description: "Demilitarized zone".to_string(),
    };

    manager.add_segment(segment);
    assert_eq!(manager.get_segments().len(), 1);
}

#[test]
fn test_perimeter_defense_comprehensive_statistics() {
    let manager = PerimeterDefenseManager::new();
    let stats = manager.get_statistics();

    assert!(stats.defense_enabled);
    assert_eq!(stats.firewall.total_rules, 0);
    assert_eq!(stats.routing.total_routes, 0);
    assert_eq!(stats.service_mesh.total_services, 0);

}

#[test]
fn test_perimeter_defense_maintenance() {
    let mut manager = PerimeterDefenseManager::new();

    // Perform maintenance (should not panic)
    manager.perform_maintenance();

    // Should still be operational
    assert!(manager.is_defense_enabled());
}
