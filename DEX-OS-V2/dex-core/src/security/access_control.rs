//! Access Control Module for Protection Layer 4 - Access Control
//!
//! Implements access control from DEX-OS-V2.csv line 248:
//! - Security,Protection Layer,Protection Layer 4,Access Control,Permission Management,High
//!
//! Features:
//! - Role-Based Access Control (RBAC)
//! - Permission management
//! - Resource access control
//! - Fine-grained permissions
//! - Permission inheritance
//! - Access decision caching

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Access control errors
#[derive(Debug, Error, Clone, PartialEq)]
pub enum AccessControlError {
    #[error("Access denied: {reason}")]
    AccessDenied { reason: String },
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    #[error("Permission not found: {0}")]
    PermissionNotFound(String),
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    #[error("Invalid permission format: {0}")]
    InvalidPermission(String),
}

/// Permission action types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Read,
    Write,
    Delete,
    Execute,
    Admin,
    Custom(String),
}

impl Action {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "read" => Action::Read,
            "write" => Action::Write,
            "delete" => Action::Delete,
            "execute" => Action::Execute,
            "admin" => Action::Admin,
            _ => Action::Custom(s.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Action::Read => "read",
            Action::Write => "write",
            Action::Delete => "delete",
            Action::Execute => "execute",
            Action::Admin => "admin",
            Action::Custom(s) => s.as_str(),
        }
    }
}

/// Permission definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Resource type (e.g., "user", "order", "wallet")
    pub resource: String,
    /// Action allowed on resource
    pub action: Action,
    /// Optional resource ID filter
    pub resource_id: Option<String>,
}

impl Permission {
    pub fn new(resource: impl Into<String>, action: Action) -> Self {
        Self {
            resource: resource.into(),
            action,
            resource_id: None,
        }
    }

    pub fn with_resource_id(mut self, id: impl Into<String>) -> Self {
        self.resource_id = Some(id.into());
        self
    }

    /// Convert to string format: "resource:action[:resource_id]"
    pub fn to_string(&self) -> String {
        if let Some(ref id) = self.resource_id {
            format!("{}:{}:{}", self.resource, self.action.as_str(), id)
        } else {
            format!("{}:{}", self.resource, self.action.as_str())
        }
    }

    /// Parse from string format
    pub fn from_string(s: &str) -> Result<Self, AccessControlError> {
        let parts: Vec<&str> = s.split(':').collect();
        
        if parts.len() < 2 {
            return Err(AccessControlError::InvalidPermission(s.to_string()));
        }

        let resource = parts[0].to_string();
        let action = Action::from_str(parts[1]);
        let resource_id = if parts.len() > 2 {
            Some(parts[2].to_string())
        } else {
            None
        };

        Ok(Self {
            resource,
            action,
            resource_id,
        })
    }

    /// Check if this permission matches/implies another permission
    pub fn implies(&self, other: &Permission) -> bool {
        // Resource must match
        if self.resource != other.resource && self.resource != "*" {
            return false;
        }

        // Admin action implies all actions
        if self.action == Action::Admin {
            return true;
        }

        // Action must match
        if self.action != other.action {
            return false;
        }

        // If we have no resource_id restriction, we grant access to all
        if self.resource_id.is_none() {
            return true;
        }

        // Otherwise, resource_id must match
        self.resource_id == other.resource_id
    }
}

/// Role definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Role {
    /// Role name
    pub name: String,
    /// Role description
    pub description: String,
    /// Permissions granted to this role
    pub permissions: HashSet<Permission>,
    /// Parent roles (for inheritance)
    pub parent_roles: HashSet<String>,
}

impl Role {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
        }
    }

    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    pub fn add_parent_role(&mut self, role_name: impl Into<String>) {
        self.parent_roles.insert(role_name.into());
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.iter().any(|p| p.implies(permission))
    }
}

/// User with roles
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    /// User identifier
    pub id: String,
    /// User roles
    pub roles: HashSet<String>,
    /// Direct permissions (in addition to role permissions)
    pub permissions: HashSet<Permission>,
}

impl User {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            roles: HashSet::new(),
            permissions: HashSet::new(),
        }
    }

    pub fn add_role(&mut self, role: impl Into<String>) {
        self.roles.insert(role.into());
    }

    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }
}

/// Access decision
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessDecision {
    /// Whether access is allowed
    pub allowed: bool,
    /// Reason for the decision
    pub reason: String,
    /// Permissions that granted access (if allowed)
    pub granted_by: Vec<String>,
}

/// Access Control Manager
#[derive(Debug, Clone)]
pub struct AccessControlManager {
    /// Registered roles
    roles: Arc<RwLock<HashMap<String, Role>>>,
    /// Registered users
    users: Arc<RwLock<HashMap<String, User>>>,
    /// Decision cache (user_id:resource:action -> decision)
    decision_cache: Arc<RwLock<HashMap<String, AccessDecision>>>,
    /// Cache TTL in seconds
    cache_ttl: u64,
}

impl AccessControlManager {
    /// Create a new access control manager
    pub fn new() -> Self {
        let mut manager = Self {
            roles: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
            decision_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: 300, // 5 minutes
        };

        // Register default roles
        manager.register_default_roles();
        
        manager
    }

    /// Register default system roles
    fn register_default_roles(&mut self) {
        // Super Admin role
        let mut admin = Role::new("admin", "System Administrator");
        admin.add_permission(Permission::new("*", Action::Admin));
        self.register_role(admin).ok();

        // User role (basic permissions)
        let mut user = Role::new("user", "Regular User");
        user.add_permission(Permission::new("profile", Action::Read));
        user.add_permission(Permission::new("profile", Action::Write));
        user.add_permission(Permission::new("wallet", Action::Read));
        self.register_role(user).ok();

        // Trader role
        let mut trader = Role::new("trader", "Trading User");
        trader.add_parent_role("user");
        trader.add_permission(Permission::new("order", Action::Read));
        trader.add_permission(Permission::new("order", Action::Write));
        trader.add_permission(Permission::new("trade", Action::Execute));
        self.register_role(trader).ok();

        // Manager role
        let mut manager_role = Role::new("manager", "System Manager");
        manager_role.add_parent_role("trader");
        manager_role.add_permission(Permission::new("user", Action::Read));
        manager_role.add_permission(Permission::new("report", Action::Read));
        self.register_role(manager_role).ok();
    }

    /// Register a new role
    pub fn register_role(&self, role: Role) -> Result<(), AccessControlError> {
        let mut roles = self.roles.write().unwrap();
        roles.insert(role.name.clone(), role);
        Ok(())
    }

    /// Get a role by name
    pub fn get_role(&self, name: &str) -> Result<Role, AccessControlError> {
        let roles = self.roles.read().unwrap();
        roles.get(name)
            .cloned()
            .ok_or_else(|| AccessControlError::RoleNotFound(name.to_string()))
    }

    /// Register a new user
    pub fn register_user(&self, user: User) -> Result<(), AccessControlError> {
        let mut users = self.users.write().unwrap();
        let user_id = user.id.clone();
        users.insert(user_id.clone(), user);
        
        // Clear cache for this user
        self.clear_user_cache(&user_id);
        
        Ok(())
    }

    /// Get a user by ID
    pub fn get_user(&self, user_id: &str) -> Result<User, AccessControlError> {
        let users = self.users.read().unwrap();
        users.get(user_id)
            .cloned()
            .ok_or_else(|| AccessControlError::UserNotFound(user_id.to_string()))
    }

    /// Assign role to user
    pub fn assign_role(&self, user_id: &str, role_name: &str) -> Result<(), AccessControlError> {
        // Verify role exists
        self.get_role(role_name)?;

        let mut users = self.users.write().unwrap();
        let user = users.get_mut(user_id)
            .ok_or_else(|| AccessControlError::UserNotFound(user_id.to_string()))?;

        user.add_role(role_name);
        
        // Clear cache for this user
        drop(users);
        self.clear_user_cache(user_id);
        
        Ok(())
    }

    /// Grant permission directly to user
    pub fn grant_permission(&self, user_id: &str, permission: Permission) -> Result<(), AccessControlError> {
        let mut users = self.users.write().unwrap();
        let user = users.get_mut(user_id)
            .ok_or_else(|| AccessControlError::UserNotFound(user_id.to_string()))?;

        user.add_permission(permission);
        
        // Clear cache for this user
        drop(users);
        self.clear_user_cache(user_id);
        
        Ok(())
    }

    /// Get all permissions for a user (including role permissions)
    pub fn get_user_permissions(&self, user_id: &str) -> Result<HashSet<Permission>, AccessControlError> {
        let user = self.get_user(user_id)?;
        let mut permissions = user.permissions.clone();

        // Add permissions from roles
        for role_name in &user.roles {
            if let Ok(role) = self.get_role(role_name) {
                permissions.extend(self.get_role_permissions_recursive(&role));
            }
        }

        Ok(permissions)
    }

    /// Get permissions for a role recursively (including parent roles)
    fn get_role_permissions_recursive(&self, role: &Role) -> HashSet<Permission> {
        let mut permissions = role.permissions.clone();

        // Add permissions from parent roles
        for parent_name in &role.parent_roles {
            if let Ok(parent_role) = self.get_role(parent_name) {
                permissions.extend(self.get_role_permissions_recursive(&parent_role));
            }
        }

        permissions
    }

    /// Check if user has permission
    pub fn check_permission(&self, user_id: &str, permission: &Permission) -> Result<AccessDecision, AccessControlError> {
        // Check cache first
        let cache_key = format!("{}:{}", user_id, permission.to_string());
        {
            let cache = self.decision_cache.read().unwrap();
            if let Some(decision) = cache.get(&cache_key) {
                return Ok(decision.clone());
            }
        }

        // Get user permissions
        let permissions = self.get_user_permissions(user_id)?;

        // Check if any permission implies the requested one
        let mut granted_by = Vec::new();
        for user_perm in permissions {
            if user_perm.implies(permission) {
                granted_by.push(user_perm.to_string());
            }
        }

        let decision = if granted_by.is_empty() {
            AccessDecision {
                allowed: false,
                reason: format!("User {} does not have permission {}", user_id, permission.to_string()),
                granted_by: vec![],
            }
        } else {
            AccessDecision {
                allowed: true,
                reason: format!("Permission granted"),
                granted_by,
            }
        };

        // Cache the decision
        {
            let mut cache = self.decision_cache.write().unwrap();
            cache.insert(cache_key, decision.clone());
        }

        Ok(decision)
    }

    /// Clear all decision cache
    pub fn clear_cache(&self) {
        let mut cache = self.decision_cache.write().unwrap();
        cache.clear();
    }

    /// Get statistics
    pub fn get_statistics(&self) -> AccessControlStatistics {
        let roles = self.roles.read().unwrap();
        let users = self.users.read().unwrap();
        let cache = self.decision_cache.read().unwrap();

        AccessControlStatistics {
            total_roles: roles.len(),
            total_users: users.len(),
            cached_decisions: cache.len(),
        }
    }
    /// Clear cache for a specific user
    fn clear_user_cache(&self, user_id: &str) {
        let mut cache = self.decision_cache.write().unwrap();
        let prefix = format!("{}:", user_id);
        cache.retain(|k, _| !k.starts_with(&prefix));
    }

    /// Check if user has permission (boolean helper)
    pub fn has_permission(&self, user_id: &str, permission: &Permission) -> bool {
        match self.check_permission(user_id, permission) {
            Ok(decision) => decision.allowed,
            Err(_) => false,
        }
    }
}

/// Access control statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessControlStatistics {
    pub total_roles: usize,
    pub total_users: usize,
    pub cached_decisions: usize,
}

impl Default for AccessControlManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_creation() {
        let perm = Permission::new("user", Action::Read);
        assert_eq!(perm.resource, "user");
        assert_eq!(perm.action, Action::Read);
        assert_eq!(perm.to_string(), "user:read");
    }

    #[test]
    fn test_permission_parsing() {
        let perm = Permission::from_string("order:write:123").unwrap();
        assert_eq!(perm.resource, "order");
        assert_eq!(perm.action, Action::Write);
        assert_eq!(perm.resource_id, Some("123".to_string()));
    }

    #[test]
    fn test_permission_implies() {
        let admin_perm = Permission::new("user", Action::Admin);
        let read_perm = Permission::new("user", Action::Read);
        
        assert!(admin_perm.implies(&read_perm));
        assert!(!read_perm.implies(&admin_perm));
    }

    #[test]
    fn test_wildcard_permission() {
        let wildcard = Permission::new("*", Action::Admin);
        let specific = Permission::new("user", Action::Read);
        
        assert!(wildcard.implies(&specific));
    }

    #[test]
    fn test_role_creation() {
        let mut role = Role::new("admin", "Administrator");
        role.add_permission(Permission::new("user", Action::Admin));
        
        assert_eq!(role.name, "admin");
        assert_eq!(role.permissions.len(), 1);
    }

    #[test]
    fn test_user_creation() {
        let mut user = User::new("user123");
        user.add_role("trader");
        
        assert_eq!(user.id, "user123");
        assert!(user.roles.contains("trader"));
    }

    #[test]
    fn test_access_control_manager() {
        let acm = AccessControlManager::new();
        
        // Create and register a custom role
        let mut custom_role = Role::new("custom", "Custom Role");
        custom_role.add_permission(Permission::new("resource", Action::Read));
        acm.register_role(custom_role).unwrap();
        
        // Create and register a user
        let mut user = User::new("user1");
        user.add_role("custom");
        acm.register_user(user).unwrap();
        
        // Check permission
        let perm = Permission::new("resource", Action::Read);
        assert!(acm.has_permission("user1", &perm));
    }

    #[test]
    fn test_role_inheritance() {
        let acm = AccessControlManager::new();
        
        // User role exists by default, trader inherits from user
        let mut user = User::new("user2");
        user.add_role("trader");
        acm.register_user(user).unwrap();
        
        // Trader should have user permissions
        let profile_perm = Permission::new("profile", Action::Read);
        assert!(acm.has_permission("user2", &profile_perm));
        
        // Trader should also have trading permissions
        let order_perm = Permission::new("order", Action::Write);
        assert!(acm.has_permission("user2", &order_perm));
    }

    #[test]
    fn test_direct_permission_grant() {
        let acm = AccessControlManager::new();
        
        let user = User::new("user3");
        acm.register_user(user).unwrap();
        
        // Grant specific permission
        let perm = Permission::new("special", Action::Execute);
        acm.grant_permission("user3", perm.clone()).unwrap();
        
        assert!(acm.has_permission("user3", &perm));
    }

    #[test]
    fn test_access_denied() {
        let acm = AccessControlManager::new();
        
        let user = User::new("user4");
        acm.register_user(user).unwrap();
        
        // User without roles should be denied
        let perm = Permission::new("admin", Action::Admin);
        assert!(!acm.has_permission("user4", &perm));
    }

    #[test]
    fn test_admin_role() {
        let acm = AccessControlManager::new();
        
        let mut user = User::new("admin_user");
        user.add_role("admin");
        acm.register_user(user).unwrap();
        
        // Admin should have access to everything
        let perm = Permission::new("anything", Action::Write);
        assert!(acm.has_permission("admin_user", &perm));
    }

    #[test]
    fn test_decision_caching() {
        let acm = AccessControlManager::new();
        
        let mut user = User::new("user5");
        user.add_role("user");
        acm.register_user(user).unwrap();
        
        let perm = Permission::new("profile", Action::Read);
        
        // First check (not cached)
        let stats1 = acm.get_statistics();
        acm.check_permission("user5", &perm).unwrap();
        
        // Second check (should be cached)
        let stats2 = acm.get_statistics();
        acm.check_permission("user5", &perm).unwrap();
        
        assert!(stats2.cached_decisions > stats1.cached_decisions);
    }
}
