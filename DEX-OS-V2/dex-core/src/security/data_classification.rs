//! Data Classification for Security Layer 5 - Data Security
//!
//! Enhanced data classification with levels, policies, and labeling.
//! From DEX-OS-V2.csv line 239:
//! - Security,Security Layer,Security Layer 5,Data Security,Encryption & Classification,High

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use super::security_manager::ClassificationLevel;
/// Classification policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationPolicy {
    pub level: ClassificationLevel,
    pub encryption_required: bool,
    pub access_roles: Vec<String>,
    pub retention_days: Option<u32>,
    pub audit_required: bool,
}

/// Data label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLabel {
    pub id: String,
    pub classification: ClassificationLevel,
    pub owner: String,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub metadata: HashMap<String, String>,
}

/// Data Classification Manager
#[derive(Debug, Clone)]
pub struct DataClassificationManager {
    policies: Arc<RwLock<HashMap<ClassificationLevel, ClassificationPolicy>>>,
    labels: Arc<RwLock<HashMap<String, DataLabel>>>,
}

impl DataClassificationManager {
    pub fn new() -> Self {
        let mut manager = Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            labels: Arc::new(RwLock::new(HashMap::new())),
        };
        manager.load_default_policies();
        manager
    }

    fn load_default_policies(&mut self) {
        let mut policies = self.policies.write().unwrap();

        policies.insert(
            ClassificationLevel::Public,
            ClassificationPolicy {
                level: ClassificationLevel::Public,
                encryption_required: false,
                access_roles: vec![], // Public means accessible to everyone (or empty list implies no restriction)
                retention_days: None,
                audit_required: false,
            },
        );

        policies.insert(
            ClassificationLevel::Internal,
            ClassificationPolicy {
                level: ClassificationLevel::Internal,
                encryption_required: false,
                access_roles: vec!["employee".to_string()],
                retention_days: Some(730), // 2 years
                audit_required: false,
            },
        );

        policies.insert(
            ClassificationLevel::Confidential,
            ClassificationPolicy {
                level: ClassificationLevel::Confidential,
                encryption_required: true,
                access_roles: vec!["employee".to_string(), "manager".to_string()],
                retention_days: Some(365),
                audit_required: true,
            },
        );

        policies.insert(
            ClassificationLevel::Secret,
            ClassificationPolicy {
                level: ClassificationLevel::Secret,
                encryption_required: true,
                access_roles: vec!["admin".to_string(), "security_officer".to_string()],
                retention_days: Some(180),
                audit_required: true,
            },
        );

        policies.insert(
            ClassificationLevel::TopSecret,
            ClassificationPolicy {
                level: ClassificationLevel::TopSecret,
                encryption_required: true,
                access_roles: vec!["admin".to_string()],
                retention_days: Some(90),
                audit_required: true,
            },
        );
    }

    pub fn classify_data(&self, data_id: String, level: ClassificationLevel, owner: String) -> DataLabel {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let label = DataLabel {
            id: data_id.clone(),
            classification: level,
            owner,
            created_at: now,
            expires_at: None,
            metadata: HashMap::new(),
        };

        let mut labels = self.labels.write().unwrap();
        labels.insert(data_id, label.clone());

        label
    }

    pub fn get_policy(&self, level: &ClassificationLevel) -> Option<ClassificationPolicy> {
        let policies = self.policies.read().unwrap();
        policies.get(level).cloned()
    }

    pub fn get_label(&self, data_id: &str) -> Option<DataLabel> {
        let labels = self.labels.read().unwrap();
        labels.get(data_id).cloned()
    }

    /// Check if a user with specific roles can access data with a given classification
    pub fn check_access(&self, level: &ClassificationLevel, user_roles: &[String]) -> bool {
        let policies = self.policies.read().unwrap();
        
        if let Some(policy) = policies.get(level) {
            // Public data is accessible to everyone
            if policy.level == ClassificationLevel::Public {
                return true;
            }

            // Check if user has any of the required roles
            // If policy has no roles, it might imply restricted access or open access depending on design
            // Here we assume if roles are defined, user must have one. If empty, maybe it's open?
            // Actually for Internal/Confidential etc we defined roles.
            
            if policy.access_roles.is_empty() {
                return true; // Assume open if no roles defined (like Public)
            }

            for role in user_roles {
                if policy.access_roles.contains(role) {
                    return true;
                }
            }
            
            false
        } else {
            // Default deny if no policy found
            false
        }
    }

    /// Check access for a specific data item
    pub fn check_data_access(&self, data_id: &str, user_roles: &[String]) -> bool {
        if let Some(label) = self.get_label(data_id) {
            self.check_access(&label.classification, user_roles)
        } else {
            // Data not labeled - default to allow or deny? 
            // Let's default to deny for safety, or maybe allow if we assume unclassified is public?
            // Safer to deny if we are strict.
            false 
        }
    }
}

impl Default for DataClassificationManager {
    fn default() -> Self {
        Self::new()
    }
}
