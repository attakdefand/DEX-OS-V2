//! Identity and Access Management (IAM) orchestration layer
//!
//! This module provides the central orchestration for:
//! - Identity Management (via `IdentityManager`)
//! - Access Control (via `SecurityManager`)
//! - Role-Based Access Control (RBAC)
//! - Policy Enforcement

use crate::identity::{IdentityError, IdentityManager};
use crate::security::{SecurityError, SecurityManager};
use crate::types::TraderId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Errors that can occur during IAM operations
#[derive(Debug, Error)]
pub enum IamError {
    #[error("Identity error: {0}")]
    Identity(#[from] IdentityError),
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    #[error("Permission denied: User {0} does not have permission {1:?}")]
    PermissionDenied(String, Permission),
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Session expired or invalid")]
    InvalidSession,
}

/// Roles available in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// Super Administrator with full access
    SuperAdmin,
    /// Governance Administrator for managing proposals and voting
    GovernanceAdmin,
    /// Security Administrator for managing keys and security policies
    SecurityAdmin,
    /// Auditor for viewing logs and reports
    Auditor,
    /// Standard Trader
    Trader,
    /// Market Maker with special trading privileges
    MarketMaker,
    /// Custom role with specific permissions
    Custom(String),
}

/// Granular permissions for system actions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Can manage system configuration
    ManageSystem,
    /// Can manage user roles
    ManageRoles,
    /// Can create governance proposals
    CreateProposal,
    /// Can vote on proposals
    Vote,
    /// Can execute approved proposals
    ExecuteProposal,
    /// Can view audit logs
    ViewAuditLogs,
    /// Can manage security keys
    ManageKeys,
    /// Can trade on the DEX
    Trade,
    /// Can provide liquidity
    ProvideLiquidity,
    /// Can view private data
    ViewPrivateData,
}

/// Policy for role management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleManagerPolicy {
    /// Roles that can assign other roles
    pub role_assignment_rules: HashMap<Role, Vec<Role>>,
}

/// Policy for approval gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalGatePolicy {
    /// Actions that require multi-sig or specific approval
    pub restricted_actions: HashSet<String>,
}

/// Identity and Access Management Orchestrator
#[derive(Debug, Clone)]
pub struct IAM {
    /// Underlying identity manager
    pub identity_manager: IdentityManager,
    /// Underlying security manager
    pub security_manager: SecurityManager,
    /// User role assignments
    user_roles: HashMap<TraderId, HashSet<Role>>,
    /// Role to permission mappings
    role_permissions: HashMap<Role, HashSet<Permission>>,
    /// Active sessions (token -> user_id)
    sessions: HashMap<String, TraderId>,
}

impl IAM {
    /// Create a new IAM instance with default configuration
    pub fn new() -> Self {
        let mut iam = Self {
            identity_manager: IdentityManager::new(),
            security_manager: SecurityManager::new(),
            user_roles: HashMap::new(),
            role_permissions: HashMap::new(),
            sessions: HashMap::new(),
        };

        iam.initialize_default_roles();
        iam
    }

    /// Initialize default roles and permissions
    fn initialize_default_roles(&mut self) {
        // SuperAdmin - All permissions
        let mut super_admin_perms = HashSet::new();
        super_admin_perms.insert(Permission::ManageSystem);
        super_admin_perms.insert(Permission::ManageRoles);
        super_admin_perms.insert(Permission::CreateProposal);
        super_admin_perms.insert(Permission::Vote);
        super_admin_perms.insert(Permission::ExecuteProposal);
        super_admin_perms.insert(Permission::ViewAuditLogs);
        super_admin_perms.insert(Permission::ManageKeys);
        super_admin_perms.insert(Permission::Trade);
        super_admin_perms.insert(Permission::ProvideLiquidity);
        super_admin_perms.insert(Permission::ViewPrivateData);
        self.role_permissions.insert(Role::SuperAdmin, super_admin_perms);

        // GovernanceAdmin
        let mut gov_admin_perms = HashSet::new();
        gov_admin_perms.insert(Permission::CreateProposal);
        gov_admin_perms.insert(Permission::Vote);
        gov_admin_perms.insert(Permission::ExecuteProposal);
        self.role_permissions.insert(Role::GovernanceAdmin, gov_admin_perms);

        // SecurityAdmin
        let mut sec_admin_perms = HashSet::new();
        sec_admin_perms.insert(Permission::ManageKeys);
        sec_admin_perms.insert(Permission::ViewAuditLogs);
        self.role_permissions.insert(Role::SecurityAdmin, sec_admin_perms);

        // Auditor
        let mut auditor_perms = HashSet::new();
        auditor_perms.insert(Permission::ViewAuditLogs);
        self.role_permissions.insert(Role::Auditor, auditor_perms);

        // Trader
        let mut trader_perms = HashSet::new();
        trader_perms.insert(Permission::Trade);
        trader_perms.insert(Permission::Vote); // Traders can vote
        self.role_permissions.insert(Role::Trader, trader_perms);

        // MarketMaker
        let mut mm_perms = HashSet::new();
        mm_perms.insert(Permission::Trade);
        mm_perms.insert(Permission::ProvideLiquidity);
        self.role_permissions.insert(Role::MarketMaker, mm_perms);
    }

    /// Register a new user with DID and initial role
    pub fn register_user(&mut self, trader_id: &TraderId) -> Result<(), IamError> {
        // Create DID in IdentityManager
        self.identity_manager.create_did(trader_id)?;
        
        // Assign default Trader role
        self.assign_role(trader_id, Role::Trader)?;
        
        Ok(())
    }

    /// Authenticate a user using biometric data
    pub fn authenticate(
        &mut self,
        trader_id: &str,
        bio_type: &str,
        bio_data: &[u8],
    ) -> Result<String, IamError> {
        // Verify biometrics via IdentityManager
        let is_verified = self.identity_manager.verify_biometric(trader_id, bio_type, bio_data)?;
        
        if is_verified {
            // Generate session token
            let token = format!("session_{}_{}", trader_id, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
            self.sessions.insert(token.clone(), trader_id.to_string());
            Ok(token)
        } else {
            Err(IamError::AuthenticationFailed)
        }
    }

    /// Check if a user has a specific permission
    pub fn has_permission(&self, trader_id: &str, permission: &Permission) -> bool {
        if let Some(roles) = self.user_roles.get(trader_id) {
            for role in roles {
                if let Some(perms) = self.role_permissions.get(role) {
                    if perms.contains(permission) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Authorize an action for a session token
    pub fn authorize(&self, session_token: &str, permission: Permission) -> Result<(), IamError> {
        let trader_id = self.sessions.get(session_token).ok_or(IamError::InvalidSession)?;
        
        if self.has_permission(trader_id, &permission) {
            Ok(())
        } else {
            Err(IamError::PermissionDenied(trader_id.clone(), permission))
        }
    }

    /// Assign a role to a user
    pub fn assign_role(&mut self, trader_id: &str, role: Role) -> Result<(), IamError> {
        // Ensure user exists (has a DID)
        if self.identity_manager.get_did(trader_id).is_none() {
            return Err(IamError::UserNotFound(trader_id.to_string()));
        }

        self.user_roles
            .entry(trader_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(role);
        Ok(())
    }

    /// Revoke a role from a user
    pub fn revoke_role(&mut self, trader_id: &str, role: &Role) -> Result<(), IamError> {
        if let Some(roles) = self.user_roles.get_mut(trader_id) {
            if roles.remove(role) {
                Ok(())
            } else {
                Err(IamError::RoleNotFound(format!("{:?}", role)))
            }
        } else {
            Err(IamError::UserNotFound(trader_id.to_string()))
        }
    }

    /// Get all roles for a user
    pub fn get_user_roles(&self, trader_id: &str) -> Option<&HashSet<Role>> {
        self.user_roles.get(trader_id)
    }

    /// Define permissions for a custom role
    pub fn define_custom_role(&mut self, role_name: String, permissions: HashSet<Permission>) {
        self.role_permissions.insert(Role::Custom(role_name), permissions);
    }
}

impl Default for IAM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iam_initialization() {
        let iam = IAM::new();
        assert!(iam.role_permissions.contains_key(&Role::SuperAdmin));
        assert!(iam.role_permissions.contains_key(&Role::Trader));
    }

    #[test]
    fn test_user_registration_and_role_assignment() {
        let mut iam = IAM::new();
        let trader_id = "trader1";

        // Register user
        assert!(iam.register_user(&trader_id.to_string()).is_ok());

        // Check default role
        let roles = iam.get_user_roles(trader_id).unwrap();
        assert!(roles.contains(&Role::Trader));

        // Assign additional role
        assert!(iam.assign_role(trader_id, Role::MarketMaker).is_ok());
        let roles = iam.get_user_roles(trader_id).unwrap();
        assert!(roles.contains(&Role::MarketMaker));
    }

    #[test]
    fn test_authentication_flow() {
        let mut iam = IAM::new();
        let trader_id = "trader1";
        let bio_type = "fingerprint";
        let bio_data = b"fingerprint_data";

        // Register user
        iam.register_user(&trader_id.to_string()).unwrap();

        // Register biometrics (directly on identity manager for setup)
        iam.identity_manager.register_biometric(trader_id, bio_type, bio_data).unwrap();

        // Authenticate successfully
        let token_result = iam.authenticate(trader_id, bio_type, bio_data);
        assert!(token_result.is_ok());
        let token = token_result.unwrap();

        // Authenticate with wrong data
        let wrong_data = b"wrong_data";
        let token_result = iam.authenticate(trader_id, bio_type, wrong_data);
        assert!(token_result.is_err()); // Should verify false, but here we expect error from verify_biometric logic or just false -> Error
        // Note: verify_biometric returns Ok(false) for mismatch, authenticate converts false to Err
    }

    #[test]
    fn test_authorization_flow() {
        let mut iam = IAM::new();
        let trader_id = "trader1";
        let bio_type = "face";
        let bio_data = b"face_data";

        // Setup user
        iam.register_user(&trader_id.to_string()).unwrap();
        iam.identity_manager.register_biometric(trader_id, bio_type, bio_data).unwrap();
        
        // Login
        let token = iam.authenticate(trader_id, bio_type, bio_data).unwrap();

        // Check Trader permissions
        assert!(iam.authorize(&token, Permission::Trade).is_ok());
        assert!(iam.authorize(&token, Permission::Vote).is_ok());
        
        // Check denied permission
        assert!(iam.authorize(&token, Permission::ManageSystem).is_err());

        // Elevate to SuperAdmin
        iam.assign_role(trader_id, Role::SuperAdmin).unwrap();
        
        // Check elevated permission
        assert!(iam.authorize(&token, Permission::ManageSystem).is_ok());
    }

    #[test]
    fn test_role_revocation() {
        let mut iam = IAM::new();
        let trader_id = "trader1";

        iam.register_user(&trader_id.to_string()).unwrap();
        iam.assign_role(trader_id, Role::Auditor).unwrap();

        assert!(iam.has_permission(trader_id, &Permission::ViewAuditLogs));

        // Revoke role
        iam.revoke_role(trader_id, &Role::Auditor).unwrap();

        assert!(!iam.has_permission(trader_id, &Permission::ViewAuditLogs));
    }

    #[test]
    fn test_custom_roles() {
        let mut iam = IAM::new();
        let trader_id = "trader1";
        let role_name = "SpecialOps".to_string();
        let custom_role = Role::Custom(role_name.clone());

        // Define custom role
        let mut perms = HashSet::new();
        perms.insert(Permission::ViewPrivateData);
        iam.define_custom_role(role_name, perms);

        // Assign custom role
        iam.register_user(&trader_id.to_string()).unwrap();
        iam.assign_role(trader_id, custom_role).unwrap();

        assert!(iam.has_permission(trader_id, &Permission::ViewPrivateData));
    }
}
