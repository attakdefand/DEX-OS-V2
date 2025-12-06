//! Security tests for the policy management module
//!
//! This module implements security tests based on the security_tests_full.csv file
//! specifically for the policy management functionality.

#[cfg(test)]
mod tests {
    use crate::governance::policy_management::PolicyCondition;
    use crate::governance::{
        GovernanceComponent, GovernanceDomain, PolicyAction, PolicyContext, PolicyManager,
        PolicyRule,
    };
    use std::collections::HashMap;

    /// Test policy enforcement on request
    #[test]
    fn test_security__governance_and_policy__policy__enforces__on_request() {
        let policy_manager = PolicyManager::new();

        // Create a policy that enforces a specific action
        let policy = PolicyRule {
            id: "test_policy_enforce".to_string(),
            name: "Test Enforce Policy".to_string(),
            description: "Policy that enforces access".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "enforces".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Deny,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "enforces".to_string(),
            condition: "on_request".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Deny action
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Deny);
        assert_eq!(
            result.matched_policy_id,
            Some("test_policy_enforce".to_string())
        );
    }

    /// Test policy validation on request
    #[test]
    fn test_security__governance_and_policy__policy__validates__on_request() {
        let policy_manager = PolicyManager::new();

        // Create a policy that validates a specific action
        let policy = PolicyRule {
            id: "test_policy_validate".to_string(),
            name: "Test Validate Policy".to_string(),
            description: "Policy that validates access".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "validates".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Allow,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "validates".to_string(),
            condition: "on_request".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Allow action
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Allow);
        assert_eq!(
            result.matched_policy_id,
            Some("test_policy_validate".to_string())
        );
    }

    /// Test policy rotation on request
    #[test]
    fn test_security__governance_and_policy__policy__rotates__on_request() {
        let policy_manager = PolicyManager::new();

        // Create initial policy
        let initial_policy = PolicyRule {
            id: "rotation_policy".to_string(),
            name: "Rotation Policy".to_string(),
            description: "Policy that will be rotated".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "rotates".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Allow,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the initial policy
        assert!(policy_manager.create_policy(initial_policy).is_ok());

        // Create updated policy (simulating rotation)
        let updated_policy = PolicyRule {
            id: "rotation_policy".to_string(),
            name: "Rotated Policy".to_string(),
            description: "Policy after rotation".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "rotates".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Deny, // Changed from Allow to Deny
            priority: 200,              // Higher priority
            enabled: true,
            metadata: HashMap::new(),
        };

        // Update the policy (simulating rotation)
        assert!(policy_manager.update_policy(updated_policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "rotates".to_string(),
            condition: "on_request".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Deny action (from updated policy)
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Deny);
        assert_eq!(
            result.matched_policy_id,
            Some("rotation_policy".to_string())
        );
    }

    /// Test policy blocking on request
    #[test]
    fn test_security__governance_and_policy__policy__blocks__on_request() {
        let policy_manager = PolicyManager::new();

        // Create a policy that blocks a specific action
        let policy = PolicyRule {
            id: "blocking_policy".to_string(),
            name: "Blocking Policy".to_string(),
            description: "Policy that blocks access".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "blocks".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Deny,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "blocks".to_string(),
            condition: "on_request".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Deny action (blocking)
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Deny);
        assert_eq!(
            result.matched_policy_id,
            Some("blocking_policy".to_string())
        );
    }

    /// Test policy detection on request
    #[test]
    fn test_security__governance_and_policy__policy__detects__on_request() {
        let policy_manager = PolicyManager::new();

        // Create a policy that detects a specific action
        let policy = PolicyRule {
            id: "detection_policy".to_string(),
            name: "Detection Policy".to_string(),
            description: "Policy that detects suspicious activity".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "detects".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Log, // Log for detection
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "detects".to_string(),
            condition: "on_request".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Log action (for detection)
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Log);
        assert_eq!(
            result.matched_policy_id,
            Some("detection_policy".to_string())
        );
    }

    /// Test policy evidence logging on request
    #[test]
    fn test_security__governance_and_policy__policy__logs_evidence__on_request() {
        let policy_manager = PolicyManager::new();

        // Create a policy that logs evidence
        let mut metadata = HashMap::new();
        metadata.insert("evidence_required".to_string(), "true".to_string());
        metadata.insert("log_level".to_string(), "info".to_string());

        let policy = PolicyRule {
            id: "evidence_logging_policy".to_string(),
            name: "Evidence Logging Policy".to_string(),
            description: "Policy that logs evidence".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "logs_evidence".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Log,
            priority: 100,
            enabled: true,
            metadata,
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "logs_evidence".to_string(),
            condition: "on_request".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Log action
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Log);
        assert_eq!(
            result.matched_policy_id,
            Some("evidence_logging_policy".to_string())
        );
    }

    /// Test policy enforcement during CI
    #[test]
    fn test_security__governance_and_policy__policy__enforces__during_ci() {
        let policy_manager = PolicyManager::new();

        // Create a policy that enforces during CI
        let policy = PolicyRule {
            id: "ci_enforcement_policy".to_string(),
            name: "CI Enforcement Policy".to_string(),
            description: "Policy that enforces during CI".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "enforces".to_string(),
                condition: "during_ci".to_string(),
            },
            action: PolicyAction::Deny,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "enforces".to_string(),
            condition: "during_ci".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Deny action
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Deny);
        assert_eq!(
            result.matched_policy_id,
            Some("ci_enforcement_policy".to_string())
        );
    }

    /// Test policy validation during CI
    #[test]
    fn test_security__governance_and_policy__policy__validates__during_ci() {
        let policy_manager = PolicyManager::new();

        // Create a policy that validates during CI
        let policy = PolicyRule {
            id: "ci_validation_policy".to_string(),
            name: "CI Validation Policy".to_string(),
            description: "Policy that validates during CI".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "validates".to_string(),
                condition: "during_ci".to_string(),
            },
            action: PolicyAction::Allow,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "validates".to_string(),
            condition: "during_ci".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Allow action
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Allow);
        assert_eq!(
            result.matched_policy_id,
            Some("ci_validation_policy".to_string())
        );
    }

    /// Test policy rotation during CI
    #[test]
    fn test_security__governance_and_policy__policy__rotates__during_ci() {
        let policy_manager = PolicyManager::new();

        // Create initial policy
        let initial_policy = PolicyRule {
            id: "ci_rotation_policy".to_string(),
            name: "CI Rotation Policy".to_string(),
            description: "Policy that rotates during CI".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "rotates".to_string(),
                condition: "during_ci".to_string(),
            },
            action: PolicyAction::Allow,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the initial policy
        assert!(policy_manager.create_policy(initial_policy).is_ok());

        // Create updated policy (simulating rotation)
        let updated_policy = PolicyRule {
            id: "ci_rotation_policy".to_string(),
            name: "Rotated CI Policy".to_string(),
            description: "Policy after CI rotation".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "rotates".to_string(),
                condition: "during_ci".to_string(),
            },
            action: PolicyAction::Challenge, // Changed to Challenge
            priority: 200,                   // Higher priority
            enabled: true,
            metadata: HashMap::new(),
        };

        // Update the policy (simulating rotation)
        assert!(policy_manager.update_policy(updated_policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "rotates".to_string(),
            condition: "during_ci".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Challenge action (from updated policy)
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Challenge);
        assert_eq!(
            result.matched_policy_id,
            Some("ci_rotation_policy".to_string())
        );
    }

    /// Test policy blocking during CI
    #[test]
    fn test_security__governance_and_policy__policy__blocks__during_ci() {
        let policy_manager = PolicyManager::new();

        // Create a policy that blocks during CI
        let policy = PolicyRule {
            id: "ci_blocking_policy".to_string(),
            name: "CI Blocking Policy".to_string(),
            description: "Policy that blocks during CI".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "blocks".to_string(),
                condition: "during_ci".to_string(),
            },
            action: PolicyAction::Deny,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "blocks".to_string(),
            condition: "during_ci".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Deny action (blocking)
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Deny);
        assert_eq!(
            result.matched_policy_id,
            Some("ci_blocking_policy".to_string())
        );
    }

    /// Test policy detection during CI
    #[test]
    fn test_security__governance_and_policy__policy__detects__during_ci() {
        let policy_manager = PolicyManager::new();

        // Create a policy that detects during CI
        let policy = PolicyRule {
            id: "ci_detection_policy".to_string(),
            name: "CI Detection Policy".to_string(),
            description: "Policy that detects during CI".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "detects".to_string(),
                condition: "during_ci".to_string(),
            },
            action: PolicyAction::Log,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "detects".to_string(),
            condition: "during_ci".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Log action (for detection)
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Log);
        assert_eq!(
            result.matched_policy_id,
            Some("ci_detection_policy".to_string())
        );
    }

    /// Test policy evidence logging during CI
    #[test]
    fn test_security__governance_and_policy__policy__logs_evidence__during_ci() {
        let policy_manager = PolicyManager::new();

        // Create a policy that logs evidence during CI
        let mut metadata = HashMap::new();
        metadata.insert("evidence_required".to_string(), "true".to_string());
        metadata.insert("log_level".to_string(), "debug".to_string());

        let policy = PolicyRule {
            id: "ci_evidence_logging_policy".to_string(),
            name: "CI Evidence Logging Policy".to_string(),
            description: "Policy that logs evidence during CI".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "logs_evidence".to_string(),
                condition: "during_ci".to_string(),
            },
            action: PolicyAction::Log,
            priority: 100,
            enabled: true,
            metadata,
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "logs_evidence".to_string(),
            condition: "during_ci".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Log action
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Log);
        assert_eq!(
            result.matched_policy_id,
            Some("ci_evidence_logging_policy".to_string())
        );
    }

    /// Test Risk & Threat Modeling policy enforcement on request
    #[test]
    fn test_security__risk_and_threat_modeling__policy__enforces__on_request() {
        let policy_manager = PolicyManager::new();

        // Create a policy for Risk & Threat Modeling
        let policy = PolicyRule {
            id: "risk_threat_policy".to_string(),
            name: "Risk Threat Policy".to_string(),
            description: "Policy for risk and threat modeling".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::RiskExceptionManagement,
                component: GovernanceComponent::RiskRegistry,
                behavior: "enforces".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Deny,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::RiskExceptionManagement,
            component: GovernanceComponent::RiskRegistry,
            behavior: "enforces".to_string(),
            condition: "on_request".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Deny action
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Deny);
        assert_eq!(
            result.matched_policy_id,
            Some("risk_threat_policy".to_string())
        );
    }

    /// Test Secure SDLC & Supply Chain policy validation during CI
    #[test]
    fn test_security__secure_sdlc_and_supply_chain__policy__validates__during_ci() {
        let policy_manager = PolicyManager::new();

        // Create a policy for Secure SDLC & Supply Chain
        let policy = PolicyRule {
            id: "sdlc_supply_chain_policy".to_string(),
            name: "SDLC Supply Chain Policy".to_string(),
            description: "Policy for secure SDLC and supply chain".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::PolicyAsCodeAutomation,
                component: GovernanceComponent::RegoValidator,
                behavior: "validates".to_string(),
                condition: "during_ci".to_string(),
            },
            action: PolicyAction::Allow,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::PolicyAsCodeAutomation,
            component: GovernanceComponent::RegoValidator,
            behavior: "validates".to_string(),
            condition: "during_ci".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Allow action
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Allow);
        assert_eq!(
            result.matched_policy_id,
            Some("sdlc_supply_chain_policy".to_string())
        );
    }

    /// Test Identity & Access policy rotation on request
    #[test]
    fn test_security__identity_and_access__policy__rotates__on_request() {
        let policy_manager = PolicyManager::new();

        // Create initial policy
        let initial_policy = PolicyRule {
            id: "identity_access_rotation".to_string(),
            name: "Identity Access Rotation Policy".to_string(),
            description: "Policy that rotates identity and access controls".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::AccessAuthorizationGovernance,
                component: GovernanceComponent::RoleManager,
                behavior: "rotates".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Allow,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the initial policy
        assert!(policy_manager.create_policy(initial_policy).is_ok());

        // Create updated policy (simulating rotation)
        let updated_policy = PolicyRule {
            id: "identity_access_rotation".to_string(),
            name: "Rotated Identity Access Policy".to_string(),
            description: "Policy after rotation".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::AccessAuthorizationGovernance,
                component: GovernanceComponent::RoleManager,
                behavior: "rotates".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Challenge, // Changed to Challenge
            priority: 200,                   // Higher priority
            enabled: true,
            metadata: HashMap::new(),
        };

        // Update the policy (simulating rotation)
        assert!(policy_manager.update_policy(updated_policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::AccessAuthorizationGovernance,
            component: GovernanceComponent::RoleManager,
            behavior: "rotates".to_string(),
            condition: "on_request".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Challenge action (from updated policy)
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Challenge);
        assert_eq!(
            result.matched_policy_id,
            Some("identity_access_rotation".to_string())
        );
    }

    /// Test Secrets Management policy blocking on request
    #[test]
    fn test_security__secrets_management__policy__blocks__on_request() {
        let policy_manager = PolicyManager::new();

        // Create a policy that blocks secrets management requests
        let policy = PolicyRule {
            id: "secrets_management_block".to_string(),
            name: "Secrets Management Block Policy".to_string(),
            description: "Policy that blocks secrets management requests".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::AuditEvidenceManagement,
                component: GovernanceComponent::AuditLogger,
                behavior: "blocks".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Deny,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::AuditEvidenceManagement,
            component: GovernanceComponent::AuditLogger,
            behavior: "blocks".to_string(),
            condition: "on_request".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Deny action (blocking)
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Deny);
        assert_eq!(
            result.matched_policy_id,
            Some("secrets_management_block".to_string())
        );
    }

    /// Test Key & Cryptography policy detection during CI
    #[test]
    fn test_security__key_and_cryptography__policy__detects__during_ci() {
        let policy_manager = PolicyManager::new();

        // Create a policy that detects cryptographic issues during CI
        let policy = PolicyRule {
            id: "crypto_detection_policy".to_string(),
            name: "Crypto Detection Policy".to_string(),
            description: "Policy that detects cryptographic issues during CI".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::DaoOnChainGovernance,
                component: GovernanceComponent::DaoGovernor,
                behavior: "detects".to_string(),
                condition: "during_ci".to_string(),
            },
            action: PolicyAction::Log, // Log for detection
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::DaoOnChainGovernance,
            component: GovernanceComponent::DaoGovernor,
            behavior: "detects".to_string(),
            condition: "during_ci".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Log action (for detection)
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Log);
        assert_eq!(
            result.matched_policy_id,
            Some("crypto_detection_policy".to_string())
        );
    }

    /// Test Network Segmentation policy evidence logging after deploy
    #[test]
    fn test_security__network_segmentation__policy__logs_evidence__after_deploy() {
        let policy_manager = PolicyManager::new();

        // Create a policy that logs evidence after deployment
        let mut metadata = HashMap::new();
        metadata.insert("evidence_required".to_string(), "true".to_string());
        metadata.insert("log_level".to_string(), "info".to_string());

        let policy = PolicyRule {
            id: "network_segmentation_evidence".to_string(),
            name: "Network Segmentation Evidence Policy".to_string(),
            description: "Policy that logs evidence after deployment".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::TransparencyReporting,
                component: GovernanceComponent::ReportDashboard,
                behavior: "logs_evidence".to_string(),
                condition: "after_deploy".to_string(),
            },
            action: PolicyAction::Log,
            priority: 100,
            enabled: true,
            metadata,
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::TransparencyReporting,
            component: GovernanceComponent::ReportDashboard,
            behavior: "logs_evidence".to_string(),
            condition: "after_deploy".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Log action
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Log);
        assert_eq!(
            result.matched_policy_id,
            Some("network_segmentation_evidence".to_string())
        );
    }

    /// Test Perimeter & API Gateway policy enforcement quarterly
    #[test]
    fn test_security__perimeter_and_api_gateway__policy__enforces__quarterly() {
        let policy_manager = PolicyManager::new();

        // Create a policy that enforces perimeter security quarterly
        let policy = PolicyRule {
            id: "perimeter_enforcement_policy".to_string(),
            name: "Perimeter Enforcement Policy".to_string(),
            description: "Policy that enforces perimeter security quarterly".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::ChangeManagementApprovalFlow,
                component: GovernanceComponent::ApprovalGate,
                behavior: "enforces".to_string(),
                condition: "quarterly".to_string(),
            },
            action: PolicyAction::Deny,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that matches the policy
        let context = PolicyContext {
            domain: GovernanceDomain::ChangeManagementApprovalFlow,
            component: GovernanceComponent::ApprovalGate,
            behavior: "enforces".to_string(),
            condition: "quarterly".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Deny action
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Deny);
        assert_eq!(
            result.matched_policy_id,
            Some("perimeter_enforcement_policy".to_string())
        );
    }

    /// Test multiple policy matching with priority
    #[test]
    fn test_policy_priority_enforcement() {
        let policy_manager = PolicyManager::new();

        // Create two policies with different priorities
        let low_priority_policy = PolicyRule {
            id: "low_priority".to_string(),
            name: "Low Priority Policy".to_string(),
            description: "Low priority policy".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "enforces".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Allow, // Allow
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        let high_priority_policy = PolicyRule {
            id: "high_priority".to_string(),
            name: "High Priority Policy".to_string(),
            description: "High priority policy".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "enforces".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Deny, // Deny
            priority: 200,              // Higher priority
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add both policies
        assert!(policy_manager.create_policy(low_priority_policy).is_ok());
        assert!(policy_manager.create_policy(high_priority_policy).is_ok());

        // Create a context that matches both policies
        let context = PolicyContext {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "enforces".to_string(),
            condition: "on_request".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Deny action (high priority policy)
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Deny);
        assert_eq!(result.matched_policy_id, Some("high_priority".to_string()));
    }

    /// Test disabled policy handling
    #[test]
    fn test_disabled_policy_handling() {
        let policy_manager = PolicyManager::new();

        // Create a disabled policy
        let policy = PolicyRule {
            id: "disabled_policy".to_string(),
            name: "Disabled Policy".to_string(),
            description: "Disabled policy".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "enforces".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Deny,
            priority: 100,
            enabled: false, // Disabled
            metadata: HashMap::new(),
        };

        // Add the policy to the manager
        assert!(policy_manager.create_policy(policy).is_ok());

        // Create a context that would match the policy if it were enabled
        let context = PolicyContext {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "enforces".to_string(),
            condition: "on_request".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate the policy - should return Allow action (default) since the policy is disabled
        let result = policy_manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Allow); // Default action when no policies match
        assert_eq!(result.matched_policy_id, None);
    }

    /// Test policy deletion
    #[test]
    fn test_policy_deletion() {
        let policy_manager = PolicyManager::new();

        // Create a policy
        let policy = PolicyRule {
            id: "deletable_policy".to_string(),
            name: "Deletable Policy".to_string(),
            description: "Policy to be deleted".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "enforces".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Deny,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add the policy
        assert!(policy_manager.create_policy(policy).is_ok());

        // Verify the policy exists
        assert!(policy_manager
            .get_policy(&"deletable_policy".to_string())
            .is_ok());

        // Delete the policy
        assert!(policy_manager
            .delete_policy(&"deletable_policy".to_string())
            .is_ok());

        // Verify the policy is deleted
        assert!(policy_manager
            .get_policy(&"deletable_policy".to_string())
            .is_err());

        // Try to delete a non-existent policy
        assert!(policy_manager
            .delete_policy(&"non_existent_policy".to_string())
            .is_err());
    }

    /// Test policy listing
    #[test]
    fn test_policy_listing() {
        let policy_manager = PolicyManager::new();

        // Create multiple policies
        let policy1 = PolicyRule {
            id: "policy_1".to_string(),
            name: "Policy 1".to_string(),
            description: "First policy".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "enforces".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Allow,
            priority: 100,
            enabled: true,
            metadata: HashMap::new(),
        };

        let policy2 = PolicyRule {
            id: "policy_2".to_string(),
            name: "Policy 2".to_string(),
            description: "Second policy".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::RiskExceptionManagement,
                component: GovernanceComponent::RiskRegistry,
                behavior: "validates".to_string(),
                condition: "during_ci".to_string(),
            },
            action: PolicyAction::Deny,
            priority: 200,
            enabled: true,
            metadata: HashMap::new(),
        };

        // Add both policies
        assert!(policy_manager.create_policy(policy1).is_ok());
        assert!(policy_manager.create_policy(policy2).is_ok());

        // List all policies
        let policies = policy_manager.list_policies().unwrap();
        assert_eq!(policies.len(), 2);

        // Check that both policies are in the list
        let policy_ids: Vec<String> = policies.iter().map(|p| p.id.clone()).collect();
        assert!(policy_ids.contains(&"policy_1".to_string()));
        assert!(policy_ids.contains(&"policy_2".to_string()));
    }
}
