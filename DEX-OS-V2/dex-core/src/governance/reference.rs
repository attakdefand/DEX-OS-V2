use crate::reference_common::reference_root;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::path::PathBuf;
use thiserror::Error;

const GOVERNANCE_CSV: &str = "governance_compliance_full_enriched.csv";

/// Error returned when governance reference data cannot be loaded or parsed.
#[derive(Debug, Error)]
pub enum GovernanceReferenceError {
    #[error("failed to read governance reference CSV at {path}: {source}")]
    Csv {
        path: PathBuf,
        #[source]
        source: csv::Error,
    },
    #[error("unknown governance domain '{0}'")]
    UnknownDomain(String),
    #[error("unknown governance component '{0}'")]
    UnknownComponent(String),
}

/// Governance domains represented inside the reference dataset.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GovernanceDomain {
    GovernancePolicyFramework,
    AccessAuthorizationGovernance,
    ChangeManagementApprovalFlow,
    ComplianceRegulatoryAlignment,
    RiskExceptionManagement,
    AuditEvidenceManagement,
    PolicyAsCodeAutomation,
    TransparencyReporting,
    DaoOnChainGovernance,
    EducationCultureAccountability,
}

impl GovernanceDomain {
    /// Returns the string form stored in the CSV.
    pub fn as_str(&self) -> &'static str {
        match self {
            GovernanceDomain::GovernancePolicyFramework => "Governance Policy & Framework",
            GovernanceDomain::AccessAuthorizationGovernance => "Access & Authorization Governance",
            GovernanceDomain::ChangeManagementApprovalFlow => "Change Management & Approval Flow",
            GovernanceDomain::ComplianceRegulatoryAlignment => "Compliance & Regulatory Alignment",
            GovernanceDomain::RiskExceptionManagement => "Risk & Exception Management",
            GovernanceDomain::AuditEvidenceManagement => "Audit & Evidence Management",
            GovernanceDomain::PolicyAsCodeAutomation => "Policy-as-Code & Automation",
            GovernanceDomain::TransparencyReporting => "Transparency & Reporting",
            GovernanceDomain::DaoOnChainGovernance => "DAO / On-Chain Governance",
            GovernanceDomain::EducationCultureAccountability => {
                "Education & Culture of Accountability"
            }
        }
    }
}

impl TryFrom<&str> for GovernanceDomain {
    type Error = GovernanceReferenceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Governance Policy & Framework" => Ok(GovernanceDomain::GovernancePolicyFramework),
            "Access & Authorization Governance" => {
                Ok(GovernanceDomain::AccessAuthorizationGovernance)
            }
            "Change Management & Approval Flow" => {
                Ok(GovernanceDomain::ChangeManagementApprovalFlow)
            }
            "Compliance & Regulatory Alignment" => {
                Ok(GovernanceDomain::ComplianceRegulatoryAlignment)
            }
            "Risk & Exception Management" => Ok(GovernanceDomain::RiskExceptionManagement),
            "Audit & Evidence Management" => Ok(GovernanceDomain::AuditEvidenceManagement),
            "Policy-as-Code & Automation" => Ok(GovernanceDomain::PolicyAsCodeAutomation),
            "Transparency & Reporting" => Ok(GovernanceDomain::TransparencyReporting),
            "DAO / On-Chain Governance" => Ok(GovernanceDomain::DaoOnChainGovernance),
            "Education & Culture of Accountability" => {
                Ok(GovernanceDomain::EducationCultureAccountability)
            }
            other => Err(GovernanceReferenceError::UnknownDomain(other.to_owned())),
        }
    }
}

/// Components that participate in governance controls.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GovernanceComponent {
    PolicyEngine,
    RoleManager,
    ApprovalGate,
    ComplianceMapper,
    RiskRegistry,
    AuditLogger,
    RegoValidator,
    ReportDashboard,
    DaoGovernor,
}

impl GovernanceComponent {
    /// Returns the canonical identifier used inside the CSV.
    pub fn as_str(&self) -> &'static str {
        match self {
            GovernanceComponent::PolicyEngine => "policy_engine",
            GovernanceComponent::RoleManager => "role_manager",
            GovernanceComponent::ApprovalGate => "approval_gate",
            GovernanceComponent::ComplianceMapper => "compliance_mapper",
            GovernanceComponent::RiskRegistry => "risk_registry",
            GovernanceComponent::AuditLogger => "audit_logger",
            GovernanceComponent::RegoValidator => "rego_validator",
            GovernanceComponent::ReportDashboard => "report_dashboard",
            GovernanceComponent::DaoGovernor => "dao_governor",
        }
    }
}

impl TryFrom<&str> for GovernanceComponent {
    type Error = GovernanceReferenceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "policy_engine" => Ok(GovernanceComponent::PolicyEngine),
            "role_manager" => Ok(GovernanceComponent::RoleManager),
            "approval_gate" => Ok(GovernanceComponent::ApprovalGate),
            "compliance_mapper" => Ok(GovernanceComponent::ComplianceMapper),
            "risk_registry" => Ok(GovernanceComponent::RiskRegistry),
            "audit_logger" => Ok(GovernanceComponent::AuditLogger),
            "rego_validator" => Ok(GovernanceComponent::RegoValidator),
            "report_dashboard" => Ok(GovernanceComponent::ReportDashboard),
            "dao_governor" => Ok(GovernanceComponent::DaoGovernor),
            other => Err(GovernanceReferenceError::UnknownComponent(other.to_owned())),
        }
    }
}

/// Extra metadata that accompanies each governance control.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Enrichment {
    pub owner: String,
    pub tool: String,
    pub metric: String,
    pub evidence: String,
}

/// Concrete governance control pulled from the CSV.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceScenario {
    pub domain: GovernanceDomain,
    pub component: GovernanceComponent,
    pub behavior: String,
    pub condition: String,
    pub test_name: String,
    pub enrichment: Enrichment,
}

#[derive(Debug, Deserialize)]
struct RawGovernanceRow {
    main_type: String,
    component: String,
    behavior: String,
    condition: String,
    test_name: String,
    owner: String,
    tool: String,
    metric: String,
    evidence: String,
}

/// Loads the governance reference CSV and returns typed scenarios.
pub fn load_governance_reference() -> Result<Vec<GovernanceScenario>, GovernanceReferenceError> {
    let csv_path = reference_root().join(GOVERNANCE_CSV);
    let mut reader =
        csv::Reader::from_path(&csv_path).map_err(|source| GovernanceReferenceError::Csv {
            path: csv_path.clone(),
            source,
        })?;

    let mut scenarios = Vec::new();
    for record in reader.deserialize::<RawGovernanceRow>() {
        let raw = record.map_err(|source| GovernanceReferenceError::Csv {
            path: csv_path.clone(),
            source,
        })?;

        let domain = GovernanceDomain::try_from(raw.main_type.as_str())?;
        let component = GovernanceComponent::try_from(raw.component.as_str())?;
        let enrichment = Enrichment {
            owner: raw.owner,
            tool: raw.tool,
            metric: raw.metric,
            evidence: raw.evidence,
        };

        let scenario = GovernanceScenario {
            domain,
            component,
            behavior: raw.behavior,
            condition: raw.condition,
            test_name: raw.test_name,
            enrichment,
        };

        scenarios.push(scenario);
    }

    Ok(scenarios)
}
