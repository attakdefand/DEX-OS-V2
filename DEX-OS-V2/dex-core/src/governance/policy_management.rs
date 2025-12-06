//! Policy Management implementation for the DEX-OS governance system
//!
//! This module provides a dedicated HashMap-based policy management system
//! with CRUD operations, evaluation, and enforcement mechanisms.

use super::{GovernanceComponent, GovernanceDomain, GovernanceScenario};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Policy management error types
#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("Policy not found: {0}")]
    PolicyNotFound(String),
    #[error("Policy already exists: {0}")]
    PolicyAlreadyExists(String),
    #[error("Invalid policy data: {0}")]
    InvalidPolicyData(String),
    #[error("Policy evaluation failed: {0}")]
    EvaluationFailed(String),
}

/// Policy identifier
pub type PolicyId = String;

/// Policy action types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyAction {
    Allow,
    Deny,
    Challenge,
    Log,
}

/// Policy condition for evaluation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyCondition {
    pub domain: GovernanceDomain,
    pub component: GovernanceComponent,
    pub behavior: String,
    pub condition: String,
}

/// Policy rule definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: PolicyId,
    pub name: String,
    pub description: String,
    pub condition: PolicyCondition,
    pub action: PolicyAction,
    pub priority: u32,
    pub enabled: bool,
    pub metadata: HashMap<String, String>,
}

/// Policy evaluation context
#[derive(Debug, Clone)]
pub struct PolicyContext {
    pub domain: GovernanceDomain,
    pub component: GovernanceComponent,
    pub behavior: String,
    pub condition: String,
    pub additional_data: HashMap<String, String>,
}

/// Policy evaluation result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyResult {
    pub action: PolicyAction,
    pub matched_policy_id: Option<PolicyId>,
    pub reason: String,
}

/// Policy manager for handling policy storage and operations
#[derive(Debug, Clone)]
pub struct PolicyManager {
    policies: Arc<RwLock<HashMap<PolicyId, PolicyRule>>>,
}

impl PolicyManager {
    /// Create a new policy manager
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new policy
    pub fn create_policy(&self, policy: PolicyRule) -> Result<PolicyId, PolicyError> {
        let mut policies = self.policies.write().map_err(|_| {
            PolicyError::InvalidPolicyData("Failed to acquire write lock".to_string())
        })?;

        if policies.contains_key(&policy.id) {
            return Err(PolicyError::PolicyAlreadyExists(policy.id));
        }

        let policy_id = policy.id.clone();
        policies.insert(policy_id.clone(), policy);
        Ok(policy_id)
    }

    /// Get a policy by ID
    pub fn get_policy(&self, policy_id: &PolicyId) -> Result<PolicyRule, PolicyError> {
        let policies = self.policies.read().map_err(|_| {
            PolicyError::InvalidPolicyData("Failed to acquire read lock".to_string())
        })?;
        policies
            .get(policy_id)
            .cloned()
            .ok_or_else(|| PolicyError::PolicyNotFound(policy_id.clone()))
    }

    /// Update an existing policy
    pub fn update_policy(&self, policy: PolicyRule) -> Result<(), PolicyError> {
        let mut policies = self.policies.write().map_err(|_| {
            PolicyError::InvalidPolicyData("Failed to acquire write lock".to_string())
        })?;

        if !policies.contains_key(&policy.id) {
            return Err(PolicyError::PolicyNotFound(policy.id));
        }

        policies.insert(policy.id.clone(), policy);
        Ok(())
    }

    /// Delete a policy by ID
    pub fn delete_policy(&self, policy_id: &PolicyId) -> Result<(), PolicyError> {
        let mut policies = self.policies.write().map_err(|_| {
            PolicyError::InvalidPolicyData("Failed to acquire write lock".to_string())
        })?;

        if policies.remove(policy_id).is_none() {
            return Err(PolicyError::PolicyNotFound(policy_id.clone()));
        }

        Ok(())
    }

    /// List all policies
    pub fn list_policies(&self) -> Result<Vec<PolicyRule>, PolicyError> {
        let policies = self.policies.read().map_err(|_| {
            PolicyError::InvalidPolicyData("Failed to acquire read lock".to_string())
        })?;
        Ok(policies.values().cloned().collect())
    }

    /// Evaluate policies against a context
    pub fn evaluate(&self, context: &PolicyContext) -> Result<PolicyResult, PolicyError> {
        let policies = self.policies.read().map_err(|_| {
            PolicyError::InvalidPolicyData("Failed to acquire read lock".to_string())
        })?;

        // Find matching policies sorted by priority (higher priority first)
        let mut matching_policies: Vec<&PolicyRule> = policies
            .values()
            .filter(|policy| {
                policy.enabled
                    && policy.condition.domain == context.domain
                    && policy.condition.component == context.component
                    && policy.condition.behavior == context.behavior
                    && policy.condition.condition == context.condition
            })
            .collect();

        // Sort by priority (descending)
        matching_policies.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Return the action of the highest priority policy
        if let Some(policy) = matching_policies.first() {
            Ok(PolicyResult {
                action: policy.action.clone(),
                matched_policy_id: Some(policy.id.clone()),
                reason: format!("Matched policy: {}", policy.name),
            })
        } else {
            // Default allow if no policies match
            Ok(PolicyResult {
                action: PolicyAction::Allow,
                matched_policy_id: None,
                reason: "No matching policies found, defaulting to allow".to_string(),
            })
        }
    }

    /// Enforce policy evaluation and return appropriate action
    pub fn enforce(&self, context: &PolicyContext) -> Result<PolicyAction, PolicyError> {
        let result = self.evaluate(context)?;
        Ok(result.action)
    }

    /// Convert a GovernanceScenario to a PolicyRule
    pub fn from_governance_scenario(
        scenario: &GovernanceScenario,
        action: PolicyAction,
    ) -> PolicyRule {
        PolicyRule {
            id: scenario.test_name.clone(),
            name: format!(
                "{}-{}-{}-{}",
                scenario.domain.as_str(),
                scenario.component.as_str(),
                scenario.behavior,
                scenario.condition
            ),
            description: format!(
                "Policy derived from governance scenario: {}",
                scenario.test_name
            ),
            condition: PolicyCondition {
                domain: scenario.domain.clone(),
                component: scenario.component.clone(),
                behavior: scenario.behavior.clone(),
                condition: scenario.condition.clone(),
            },
            action,
            priority: 100, // Default priority
            enabled: true,
            metadata: HashMap::new(),
        }
    }

    /// Import policies from governance scenarios
    pub fn import_from_governance_scenarios(
        &self,
        scenarios: &[GovernanceScenario],
    ) -> Result<usize, PolicyError> {
        let mut count = 0;
        for scenario in scenarios {
            // Convert scenario to policy with default Allow action
            let policy = Self::from_governance_scenario(scenario, PolicyAction::Allow);

            // Try to create the policy, ignore if it already exists
            if self.create_policy(policy).is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }
}

impl Default for PolicyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::reference::{Enrichment, GovernanceScenario};

    #[test]
    fn test_policy_crud_operations() {
        let manager = PolicyManager::new();

        // Create a policy
        let policy = PolicyRule {
            id: "test_policy_1".to_string(),
            name: "Test Policy".to_string(),
            description: "A test policy".to_string(),
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

        // Create
        let policy_id = manager.create_policy(policy.clone()).unwrap();
        assert_eq!(policy_id, "test_policy_1");

        // Read
        let retrieved_policy = manager.get_policy(&policy_id).unwrap();
        assert_eq!(retrieved_policy, policy);

        // Update
        let mut updated_policy = retrieved_policy.clone();
        updated_policy.action = PolicyAction::Deny;
        manager.update_policy(updated_policy.clone()).unwrap();

        let updated_retrieved = manager.get_policy(&policy_id).unwrap();
        assert_eq!(updated_retrieved.action, PolicyAction::Deny);

        // List
        let policies = manager.list_policies().unwrap();
        assert_eq!(policies.len(), 1);

        // Delete
        manager.delete_policy(&policy_id).unwrap();

        // Verify deletion
        assert!(manager.get_policy(&policy_id).is_err());
    }

    #[test]
    fn test_policy_evaluation() {
        let manager = PolicyManager::new();

        // Create policies with different priorities
        let deny_policy = PolicyRule {
            id: "deny_policy".to_string(),
            name: "Deny Policy".to_string(),
            description: "A deny policy with high priority".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "enforces".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Deny,
            priority: 200, // Higher priority
            enabled: true,
            metadata: HashMap::new(),
        };

        let allow_policy = PolicyRule {
            id: "allow_policy".to_string(),
            name: "Allow Policy".to_string(),
            description: "An allow policy with lower priority".to_string(),
            condition: PolicyCondition {
                domain: GovernanceDomain::GovernancePolicyFramework,
                component: GovernanceComponent::PolicyEngine,
                behavior: "enforces".to_string(),
                condition: "on_request".to_string(),
            },
            action: PolicyAction::Allow,
            priority: 100, // Lower priority
            enabled: true,
            metadata: HashMap::new(),
        };

        manager.create_policy(deny_policy).unwrap();
        manager.create_policy(allow_policy).unwrap();

        // Create evaluation context
        let context = PolicyContext {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "enforces".to_string(),
            condition: "on_request".to_string(),
            additional_data: HashMap::new(),
        };

        // Evaluate - should match the deny policy due to higher priority
        let result = manager.evaluate(&context).unwrap();
        assert_eq!(result.action, PolicyAction::Deny);
        assert_eq!(result.matched_policy_id, Some("deny_policy".to_string()));
    }

    #[test]
    fn test_import_from_governance_scenarios() {
        let manager = PolicyManager::new();

        // Create test scenarios
        let scenarios = vec![GovernanceScenario {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "enforces".to_string(),
            condition: "on_request".to_string(),
            test_name: "test_policy_enforce_on_request".to_string(),
            enrichment: Enrichment {
                owner: "test".to_string(),
                tool: "test".to_string(),
                metric: "test".to_string(),
                evidence: "test".to_string(),
            },
        }];

        // Import scenarios
        let count = manager
            .import_from_governance_scenarios(&scenarios)
            .unwrap();
        assert_eq!(count, 1);

        // Verify policy was created
        let policies = manager.list_policies().unwrap();
        assert_eq!(policies.len(), 1);
        assert_eq!(policies[0].id, "test_policy_enforce_on_request");
    }
}
