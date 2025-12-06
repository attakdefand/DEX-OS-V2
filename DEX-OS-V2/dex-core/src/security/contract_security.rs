//! Contract security records for audited smart contracts built on OpenZeppelin standards.
//!
//! Implements Priority 4 feature:
//! - `4,Governance & Security,Security Modules,Security Modules,Audited Smart Contracts (e.g. OpenZeppelin),Contract Security,High`
//!
//! The manager tracks audit metadata per contract and enforces that audited
//! contracts include well-known OpenZeppelin building blocks before they are
//! considered compliant with the security policy.

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime};

use thiserror::Error;

/// Recommended OpenZeppelin modules that should appear in audited builds.
pub const OPENZEPPELIN_RECOMMENDED_MODULES: &[&str] =
    &["Ownable", "AccessControl", "ReentrancyGuard", "Pausable"];

/// Represents a single audited smart contract entry.
#[derive(Debug, Clone)]
pub struct ContractAuditRecord {
    /// Normalized contract address (no `0x` prefix and lowercase).
    pub address: String,
    /// Friendly name (e.g., `Vaultv3`).
    pub name: String,
    /// Release version string.
    pub version: String,
    /// Framework used to build the contract.
    pub framework: SecurityFramework,
    /// Public URL for the audit report.
    pub audit_report_url: String,
    /// Modules/functions validated during the audit.
    pub audited_modules: Vec<String>,
    /// Timestamp of the audit.
    pub audited_at: SystemTime,
}

/// Supported contract security frameworks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityFramework {
    /// Canonical OpenZeppelin contracts.
    OpenZeppelin,
    /// Custom framework, tracked with its name/identifier.
    Custom(String),
}

impl fmt::Display for SecurityFramework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityFramework::OpenZeppelin => write!(f, "OpenZeppelin"),
            SecurityFramework::Custom(name) => write!(f, "Custom({name})"),
        }
    }
}

/// Outcome of an OpenZeppelin compliance evaluation.
#[derive(Debug, Clone)]
pub struct ComplianceReport {
    /// Normalized address of the audited contract.
    pub address: String,
    /// Framework used for the build.
    pub framework: SecurityFramework,
    /// Modules that were validated.
    pub audited_modules: Vec<String>,
    /// Missing recommended OpenZeppelin modules (empty when fully compliant).
    pub missing_modules: Vec<String>,
}

/// Errors produced while evaluating contract security.
#[derive(Debug, Error, PartialEq)]
pub enum ContractSecurityError {
    #[error("contract {0} has not been registered as audited")]
    NotAudited(String),
    #[error("contract {address} missing OpenZeppelin modules: {missing:?}")]
    MissingModules {
        address: String,
        missing: Vec<String>,
    },
    #[error("contract {address} audit is older than the allowed {threshold:?}")]
    AuditTooOld {
        address: String,
        threshold: Duration,
    },
}

/// Manager that keeps track of audited smart contracts.
#[derive(Debug, Default)]
pub struct ContractSecurityManager {
    audits: HashMap<String, ContractAuditRecord>,
}

impl ContractSecurityManager {
    /// Create a new manager.
    pub fn new() -> Self {
        Self {
            audits: HashMap::new(),
        }
    }

    /// Register or update an audit record.
    pub fn register_audit(&mut self, mut record: ContractAuditRecord) {
        record.address = normalize_address(&record.address);
        self.audits.insert(record.address.clone(), record);
    }

    /// Returns `true` if the contract has an audit record.
    pub fn is_audited(&self, address: &str) -> bool {
        let key = normalize_address(address);
        self.audits.contains_key(&key)
    }

    /// Get a reference to the audit record.
    pub fn get_audit(&self, address: &str) -> Result<&ContractAuditRecord, ContractSecurityError> {
        let key = normalize_address(address);
        self.audits
            .get(&key)
            .ok_or_else(|| ContractSecurityError::NotAudited(address.to_string()))
    }

    /// Evaluate if the contract includes the recommended OpenZeppelin modules.
    pub fn assess_openszeppelin_compliance(
        &self,
        address: &str,
    ) -> Result<ComplianceReport, ContractSecurityError> {
        let record = self.get_audit(address)?;
        let normalized = normalize_address(&record.address);

        let missing_modules: Vec<String> = OPENZEPPELIN_RECOMMENDED_MODULES
            .iter()
            .filter(|needed| {
                !record
                    .audited_modules
                    .iter()
                    .any(|m| m.eq_ignore_ascii_case(needed))
            })
            .map(|module| module.to_string())
            .collect();

        if !missing_modules.is_empty() {
            return Err(ContractSecurityError::MissingModules {
                address: normalized.clone(),
                missing: missing_modules,
            });
        }

        Ok(ComplianceReport {
            address: normalized,
            framework: record.framework.clone(),
            audited_modules: record.audited_modules.clone(),
            missing_modules: Vec::new(),
        })
    }

    /// Check whether a contract needs to be re-audited relative to the provided threshold.
    pub fn needs_reaudit(
        &self,
        address: &str,
        threshold: Duration,
    ) -> Result<bool, ContractSecurityError> {
        let record = self.get_audit(address)?;
        let elapsed = record
            .audited_at
            .elapsed()
            .unwrap_or_else(|_| Duration::from_secs(0));
        if elapsed > threshold {
            Err(ContractSecurityError::AuditTooOld {
                address: normalize_address(&record.address),
                threshold,
            })
        } else {
            Ok(false)
        }
    }
}

fn normalize_address(address: &str) -> String {
    address
        .trim()
        .trim_start_matches("0x")
        .to_ascii_lowercase()
}
