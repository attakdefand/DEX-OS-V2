//! Tests for the Load Balancer functionality
//!
//! This file tests the service mesh implementation which provides load balancing
//! capabilities for the DEX-OS network infrastructure.

use dex_core::network::service_mesh::{
    Service, ServiceEndpoint, ServiceMeshManager, LoadBalancingStrategy, HealthStatus
};
use std::net::{IpAddr, Ipv4Addr};

#[test]
fn test_round_robin_load_balancing() {
    let mut service = Service::new(
        "test-service".to_string(),
        "Test Service".to_string(),
        LoadBalancingStrategy::RoundRobin,
    );

    // Add healthy endpoints
    let endpoint1 = ServiceEndpoint::new(
        "endpoint1".to_string(),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        8080,
        1,
    );
    let endpoint2 = ServiceEndpoint::new(
        "endpoint2".to_string(),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)),
        8080,
        1,
    );
    let endpoint3 = ServiceEndpoint::new(
        "endpoint3".to_string(),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3)),
        8080,
        1,
    );

    service.add_endpoint(endpoint1);
    service.add_endpoint(endpoint2);
    service.add_endpoint(endpoint3);

    // Mark all endpoints as healthy
    for endpoint in &mut service.endpoints {
        endpoint.health = HealthStatus::Healthy;
    }

    // Test round-robin selection
    let selected1 = service.select_endpoint().unwrap().id.clone();
    let selected2 = service.select_endpoint().unwrap().id.clone();
    let selected3 = service.select_endpoint().unwrap().id.clone();
    let selected4 = service.select_endpoint().unwrap().id.clone(); // Should cycle back to first

    assert_eq!(selected1, "endpoint1");
    assert_eq!(selected2, "endpoint2");
    assert_eq!(selected3, "endpoint3");
    assert_eq!(selected4, "endpoint1");
}

#[test]
fn test_least_connections_load_balancing() {
    let mut service = Service::new(
        "test-service".to_string(),
        "Test Service".to_string(),
        LoadBalancingStrategy::LeastConnections,
    );

    // Add healthy endpoints with different connection counts
    let mut endpoint1 = ServiceEndpoint::new(
        "endpoint1".to_string(),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        8080,
        1,
    );
    endpoint1.active_connections = 10;

    let mut endpoint2 = ServiceEndpoint::new(
        "endpoint2".to_string(),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)),
        8080,
        1,
    );
    endpoint2.active_connections = 5;

    let mut endpoint3 = ServiceEndpoint::new(
        "endpoint3".to_string(),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3)),
        8080,
        1,
    );
    endpoint3.active_connections = 15;

    service.add_endpoint(endpoint1);
    service.add_endpoint(endpoint2);
    service.add_endpoint(endpoint3);

    // Mark all endpoints as healthy
    for endpoint in &mut service.endpoints {
        endpoint.health = HealthStatus::Healthy;
    }

    // Should select endpoint with least connections
    let selected = service.select_endpoint().unwrap();
    assert_eq!(selected.id, "endpoint2");
}

#[test]
fn test_weighted_round_robin_load_balancing() {
    let mut service = Service::new(
        "test-service".to_string(),
        "Test Service".to_string(),
        LoadBalancingStrategy::WeightedRoundRobin,
    );

    // Add healthy endpoints with different weights
    let mut endpoint1 = ServiceEndpoint::new(
        "endpoint1".to_string(),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        8080,
        1, // Low weight
    );
    endpoint1.health = HealthStatus::Healthy;

    let mut endpoint2 = ServiceEndpoint::new(
        "endpoint2".to_string(),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)),
        8080,
        3, // High weight
    );
    endpoint2.health = HealthStatus::Healthy;

    service.add_endpoint(endpoint1);
    service.add_endpoint(endpoint2);

    // With weighted round-robin, endpoint2 should be selected more often
    let mut endpoint1_count = 0;
    let mut endpoint2_count = 0;

    // Run multiple selections
    for _ in 0..20 {
        let selected = service.select_endpoint().unwrap();
        if selected.id == "endpoint1" {
            endpoint1_count += 1;
        } else {
            endpoint2_count += 1;
        }
    }

    // endpoint2 should be selected more often due to higher weight
    assert!(endpoint2_count > endpoint1_count);
}

#[test]
fn test_service_mesh_registration() {
    let mut mesh = ServiceMeshManager::new();
    
    let service = Service::new(
        "api-service".to_string(),
        "API Gateway".to_string(),
        LoadBalancingStrategy::RoundRobin,
    );

    // Register service
    assert!(mesh.register_service(service).is_ok());

    // Try to register the same service again - should fail
    let duplicate_service = Service::new(
        "api-service".to_string(),
        "Duplicate API Gateway".to_string(),
        LoadBalancingStrategy::RoundRobin,
    );
    assert!(mesh.register_service(duplicate_service).is_err());

    // Get service
    let retrieved_service = mesh.get_service("api-service");
    assert!(retrieved_service.is_some());
    assert_eq!(retrieved_service.unwrap().name, "api-service");
}

#[test]
fn test_service_mesh_endpoint_selection() {
    let mut mesh = ServiceMeshManager::new();
    
    let mut service = Service::new(
        "test-service".to_string(),
        "Test Service".to_string(),
        LoadBalancingStrategy::RoundRobin,
    );

    let endpoint = ServiceEndpoint::new(
        "test-endpoint".to_string(),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        8080,
        1,
    );

    service.add_endpoint(endpoint);
    
    // Mark endpoint as healthy
    if let Some(ep) = service.endpoints.first_mut() {
        ep.health = HealthStatus::Healthy;
    }

    mesh.register_service(service).unwrap();

    // Select endpoint through mesh
    let selected_endpoint = mesh.select_endpoint("test-service");
    assert!(selected_endpoint.is_ok());
    assert_eq!(selected_endpoint.unwrap().id, "test-endpoint");

    // Try to select from non-existent service
    let non_existent = mesh.select_endpoint("non-existent-service");
    assert!(non_existent.is_err());
}

#[test]
fn test_healthy_endpoints_filtering() {
    let mut service = Service::new(
        "test-service".to_string(),
        "Test Service".to_string(),
        LoadBalancingStrategy::RoundRobin,
    );

    // Add endpoints with different health statuses
    let mut healthy_endpoint = ServiceEndpoint::new(
        "healthy".to_string(),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        8080,
        1,
    );
    healthy_endpoint.health = HealthStatus::Healthy;

    let mut degraded_endpoint = ServiceEndpoint::new(
        "degraded".to_string(),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)),
        8080,
        1,
    );
    degraded_endpoint.health = HealthStatus::Degraded;

    let mut unhealthy_endpoint = ServiceEndpoint::new(
        "unhealthy".to_string(),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3)),
        8080,
        1,
    );
    unhealthy_endpoint.health = HealthStatus::Unhealthy;

    service.add_endpoint(healthy_endpoint);
    service.add_endpoint(degraded_endpoint);
    service.add_endpoint(unhealthy_endpoint);

    // Get healthy endpoints (should include healthy and degraded)
    let healthy_endpoints = service.get_healthy_endpoints();
    assert_eq!(healthy_endpoints.len(), 2);

    let ids: Vec<&str> = healthy_endpoints.iter().map(|e| e.id.as_str()).collect();
    assert!(ids.contains(&"healthy"));
    assert!(ids.contains(&"degraded"));
    assert!(!ids.contains(&"unhealthy"));
}
