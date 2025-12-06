# SRE Patterns Implementation

This document describes the implementation of Site Reliability Engineering (SRE) patterns in the DEX-OS system, specifically focusing on Error Budgets and SLO Targets as specified in the Priority 3 features.

## Overview

The SRE patterns implementation provides mechanisms for:
- Defining and tracking Service Level Objectives (SLOs)
- Managing error budgets for services
- Integrating with the existing observability system for metrics collection
- Providing tools for measuring and recording service performance

## Key Components

### SLO (Service Level Objective)

The `SLO` struct represents a service level objective with the following properties:
- `id`: Unique identifier for the SLO
- `description`: Human-readable description
- `target`: Target success rate (between 0.0 and 1.0)
- `actual`: Current actual success rate
- `total_requests`: Total number of requests recorded
- `successful_requests`: Number of successful requests
- `last_updated`: Timestamp of last update
- `window_size_seconds`: Rolling window size for SLO calculation
- `request_timestamps`: Request timestamps for rolling window calculation

### Service

The `Service` struct represents a service being monitored with SLOs:
- `id`: Unique identifier for the service
- `name`: Human-readable name
- `slos`: Collection of SLOs associated with the service
- `total_error_budget`: Total error budget for the service
- `consumed_error_budget`: Consumed error budget for the service

### SREManager

The `SREManager` is the main entry point for SRE functionality:
- Manages services and their SLOs
- Integrates with the observability system for metrics collection
- Provides methods for recording requests and managing error budgets

### SRETimer

The `SRETimer` is a helper for timing operations:
- Automatically records latency measurements
- Simplifies the process of recording successful or failed operations

## Usage Examples

### Creating and Registering a Service

```rust
use dex_core::sre_patterns::{SREManager, Service, SLO};
use dex_core::observability::ObservabilityManager;
use std::sync::Arc;

// Create observability manager
let observability = Arc::new(ObservabilityManager::new());

// Create SRE manager
let manager = SREManager::new(observability);

// Create a service with a 1% error budget
let mut service = Service::new("api-service".to_string(), "API Service".to_string(), 0.01)?;

// Create an SLO for 99.9% availability
let availability_slo = SLO::new("availability".to_string(), "API availability".to_string(), 0.999)?;

// Add the SLO to the service
service.add_slo(availability_slo);

// Register the service with the SRE manager
manager.register_service(service)?;
```

### Recording Requests

```rust
// Record a successful request
manager.record_request("api-service", "availability", true)?;

// Record a failed request
manager.record_request("api-service", "availability", false)?;

// Record a request with latency measurement
manager.record_request_with_latency("api-service", "availability", true, Some(50))?;
```

### Using the Timer Helper

```rust
use std::thread;
use std::time::Duration;

// Create a timer for an operation
let timer = manager.start_timer("api-service".to_string(), "availability".to_string());

// Simulate some work
thread::sleep(Duration::from_millis(100));

// Stop the timer and record success
timer.stop_success();

// Or for a failed operation
let timer = manager.start_timer("api-service".to_string(), "availability".to_string());
thread::sleep(Duration::from_millis(50));
timer.stop_failure();
```

### Checking Error Budget

```rust
// Check if service has sufficient error budget
let has_budget = manager.has_sufficient_error_budget("api-service", 0.005)?;

// Consume error budget
manager.consume_error_budget("api-service", 0.001)?;
```

## Integration with Observability

The SRE patterns implementation automatically integrates with the observability system, creating the following metrics:

- **Counters**:
  - `slo_requests_total_{slo_id}`: Total requests for each SLO
  - `slo_successful_requests_{slo_id}`: Successful requests for each SLO

- **Gauges**:
  - `slo_success_rate_{slo_id}`: Current success rate percentage for each SLO
  - `slo_error_budget_remaining_{slo_id}`: Remaining error budget percentage for each SLO
  - `service_error_budget_consumed_{service_id}`: Consumed error budget percentage for each service
  - `service_error_budget_remaining_{service_id}`: Remaining error budget percentage for each service

- **Histograms**:
  - `slo_request_latency_{slo_id}`: Request latency measurements for each SLO

## Error Handling

The implementation provides comprehensive error handling through the `SREError` enum:

- `InvalidSloTarget`: Invalid SLO target value (must be between 0.0 and 1.0)
- `InvalidErrorBudget`: Invalid error budget value (must be between 0.0 and 1.0)
- `SloNotFound`: SLO not found
- `ServiceNotFound`: Service not found
- `InsufficientErrorBudget`: Insufficient error budget for requested operation
- `TimeError`: Time calculation error

## Testing

The implementation includes comprehensive tests covering:
- SLO creation and validation
- Request recording and success rate calculation
- Error budget management
- Service registration and retrieval
- Timer functionality
- Edge cases and error conditions

## Implementation Status

This implementation fully satisfies the Priority 3 feature requirements:
- **SRE Patterns,SRE Patterns,SRE Patterns,Error Budget,SLO Targets,Medium** - ✅ IMPLEMENTED

The implementation provides a robust foundation for SRE practices in the DEX-OS system, enabling teams to:
- Define meaningful SLOs for their services
- Track service reliability against defined targets
- Make data-driven decisions about reliability vs. feature velocity
- Automatically collect and report on reliability metrics