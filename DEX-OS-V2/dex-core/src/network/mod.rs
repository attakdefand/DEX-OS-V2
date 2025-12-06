//! Network module for DEX-OS
//!
//! This module implements network protocols for node discovery and communication.
//! Implements the Priority 3 feature from DEX-OS-V2.csv:
//! - Infrastructure,Network,Network,Gossip Protocol,Node Discovery,Medium
//!
//! Also implements Security Layer 6 - Network & Infrastructure Security (Perimeter Defense)
//! From DEX-OS-V2.csv line 240:
//! - Security,Security Layer,Security Layer 6,Network & Infrastructure Security,Perimeter Defense,High

pub mod gossip;
pub mod gossip_sync;
pub mod pubsub;

// Security Layer 6 - Network & Infrastructure Security modules
pub mod firewall;
pub mod ip_routing;
pub mod service_mesh;
pub mod ddos_protection;
pub mod ids_ips;
pub mod traffic_monitor;
pub mod perimeter_defense;

pub use gossip::{GossipConfig, GossipError, GossipNode};
pub use gossip_sync::{GossipSyncConfig, GossipSyncError, GossipSyncNode, SyncData};
pub use pubsub::{
    MessageBroker, PubSubConfig, PubSubError, PubSubMessage, Subscription, TopicStats,
};

// Export network security components
pub use firewall::{
    FirewallError, FirewallRule, FirewallRulesManager, FirewallStatistics, IpRange, NetworkPacket,
    PortRange, Protocol, RuleAction,
};
pub use ip_routing::{IPRoutingTable, RouteEntry, RoutingError, RoutingStatistics};
pub use service_mesh::{
    CircuitBreaker, CircuitBreakerState, HealthStatus, LoadBalancingStrategy, Service,
    ServiceEndpoint, ServiceMeshError, ServiceMeshManager, ServiceMeshStatistics,
};
pub use ddos_protection::{
    DDoSError, DDoSProtectionManager, DDoSStatistics, GeoBlockingConfig,
};
pub use ids_ips::{
    AttackType, DetectedThreat, IDSError, IDSStatistics, IPSStatistics, IntrusionDetectionSystem,
    IntrusionPreventionSystem, ThreatResponse, ThreatSeverity, ThreatSignature,
};
pub use traffic_monitor::{
    BandwidthUsage, NetworkFlow, PacketInspection, ProtocolStats, TrafficMonitor,
    TrafficStatistics,
};
pub use perimeter_defense::{
    NetworkSegment, PerimeterDefenseManager, PerimeterDefenseStatistics, SecurityZone, VPNGateway,
};
