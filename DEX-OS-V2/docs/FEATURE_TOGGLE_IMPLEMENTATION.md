# Feature Toggle Implementation

This document describes the implementation of the Feature Toggle feature for conditional execution in the DEX-OS system, specifically addressing the Priority 3 feature from DEX-OS-V2.csv:
- Zero-Downtime Deployment,Zero-Downtime Deployment,Zero-Downtime Deployment,Feature Toggle,Conditional Execution,Medium

## Overview

The Feature Toggle implementation provides mechanisms for conditionally enabling or disabling features in the DEX-OS system. This approach enables teams to:
- Safely roll out new features to subsets of users
- Perform A/B testing
- Enable emergency kill switches
- Gradually increase feature adoption

## Key Components

### FeatureToggleConfig

The `FeatureToggleConfig` struct represents a feature toggle configuration with the following properties:
- `id`: Unique identifier for the feature
- `description`: Human-readable description of the feature
- `enabled`: Whether the feature is enabled
- `percentage`: Percentage of users for whom the feature is enabled (0.0 to 1.0)
- `user_groups`: User groups for whom the feature is enabled
- `start_time`: Start time for time-based activation (milliseconds since UNIX epoch)
- `end_time`: End time for time-based activation (milliseconds since UNIX epoch)
- `user_based`: Whether to use user-based targeting

### FeatureToggleManager

The `FeatureToggleManager` is the main entry point for feature toggle functionality:
- Manages feature toggles
- Provides methods for registering, retrieving, and removing features
- Handles feature activation checks

## Usage Examples

### Creating and Registering a Feature Toggle

```rust
use dex_core::feature_toggle::{FeatureToggleConfig, FeatureToggleManager};

// Create a feature toggle that is enabled for 30% of users
let mut config = FeatureToggleConfig::new(
    "new-dashboard".to_string(),
    "New dashboard feature".to_string(),
    true, // Initially enabled
);
config.set_percentage(0.3).unwrap(); // 30% rollout

// Register with feature toggle manager
let mut manager = FeatureToggleManager::new();
manager.register_feature(config).unwrap();
```

### Checking Feature Activation

```rust
// Check if feature is active for a specific user
if manager.is_feature_active("new-dashboard", "user123").unwrap() {
    // Enable the new feature for this user
    enable_new_dashboard();
} else {
    // Use the old feature
    use_old_dashboard();
}
```

### Time-Based Activation

```rust
use std::time::{SystemTime, UNIX_EPOCH};

// Create a feature toggle with time-based activation
let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
let start_time = now; // Start now
let end_time = now + 86400000; // End in 24 hours (milliseconds)

let config = FeatureToggleConfig::new(
    "limited-time-offer".to_string(),
    "Limited time offer feature".to_string(),
    true,
)
.with_time_window(start_time, end_time)
.unwrap();

// The feature will only be active between start_time and end_time
```

### User Group Targeting

```rust
// Create a feature toggle for specific user groups
let config = FeatureToggleConfig::new(
    "beta-feature".to_string(),
    "Beta feature for select users".to_string(),
    true,
)
.with_percentage(0.0) // 0% for general users
.with_user_groups(vec!["beta-testers".to_string(), "premium-users".to_string()]);

// Users in the specified groups will have access regardless of percentage
```

### Enabling/Disabling Features

```rust
// Enable a feature for all users
manager.enable_feature("new-dashboard").unwrap();

// Disable a feature completely
manager.disable_feature("problematic-feature").unwrap();

// Set a specific percentage rollout
manager.set_feature_percentage("gradual-rollout", 0.75).unwrap(); // 75% rollout
```

## Implementation Status

This implementation fully satisfies the Priority 3 feature requirement for Zero-Downtime Deployment with Feature Toggle and Conditional Execution.

The implementation provides a robust foundation for feature management in the DEX-OS system, enabling teams to:
- Safely roll out new features with controlled exposure
- Perform A/B testing and experimentation
- Quickly disable problematic features
- Manage feature lifecycles

## Testing

The implementation includes comprehensive tests that validate:
1. Basic functionality for all components
2. Error handling for invalid configurations
3. Percentage-based rollout distribution
4. Time-based activation and deactivation
5. User group targeting
6. Edge cases and boundary conditions

Tests can be run with:
```bash
cargo test feature_toggle
```

## Integration with DEX-OS

The Feature Toggle feature can be integrated into various components of the DEX-OS system:
1. **API Gateway**: For conditional feature routing
2. **Frontend Applications**: For UI feature control
3. **Service Mesh**: For service-level feature management
4. **Monitoring System**: For tracking feature adoption and usage