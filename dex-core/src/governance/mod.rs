//! AI Governance implementation for the DEX-OS core engine
//!
//! This module implements the Priority 3 features from DEX-OS-V2.csv:
//! - AI Governance (AI Proposals, Global DAO)
//!
//! It provides functionality for AI-driven governance proposals and
//! global decentralized autonomous organization (DAO) operations.

pub mod reference;
pub mod policy_engine;
pub mod iam;
pub mod compliance;
pub mod risk;

pub use reference::{
    load_governance_reference, Enrichment, GovernanceComponent, GovernanceDomain,
    GovernanceReferenceError, GovernanceScenario,
};
pub use policy_engine::{policy_for, parse_checkpoint, parse_effect, Checkpoint, PolicyEffect};
pub use iam::{ApprovalGatePolicy, RoleManagerPolicy, IamError};
pub use compliance::{build_compliance_report, render_report_json, ComplianceReport, ComplianceEntry, FrameworkRef};
pub use risk::{RiskRegistry, RiskRegistryState, RiskItem, ExceptionRequest, Notification, RiskError};

use crate::types::{TokenId, TraderId};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Represents a governance proposal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique identifier for the proposal
    pub id: String,
    /// Title of the proposal
    pub title: String,
    /// Detailed description of the proposal
    pub description: String,
    /// Type of proposal (e.g., parameter_change, treasury_allocation, protocol_upgrade)
    pub proposal_type: ProposalType,
    /// The proposer (could be AI or human)
    pub proposer: Proposer,
    /// Timestamp when the proposal was created
    pub created_at: u64,
    /// Timestamp when voting starts
    pub voting_start: u64,
    /// Timestamp when voting ends
    pub voting_end: u64,
    /// Current status of the proposal
    pub status: ProposalStatus,
    /// Votes for the proposal
    pub votes: Votes,
    /// Execution details if approved
    pub execution_plan: Option<ExecutionPlan>,
    /// AI analysis and recommendations
    pub ai_analysis: Option<AIAnalysis>,
    /// Optional governance control reference derived from the CSV dataset
    pub reference_control: Option<GovernanceScenario>,
    /// Whether the designated owner acknowledged the reference control
    pub reference_acknowledged: bool,
}

/// Types of governance proposals
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProposalType {
    ParameterChange,
    TreasuryAllocation,
    ProtocolUpgrade,
    NewMarketListing,
    FeeStructureChange,
    EmergencyPause,
    TreasuryAutomation,
    ObservabilityUpgrade,
    AccessControlUpdate,
    ChangeManagementOverride,
    EducationProgramRefresh,
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ProposalTypeKind {
    ParameterChange,
    TreasuryAllocation,
    ProtocolUpgrade,
    NewMarketListing,
    FeeStructureChange,
    EmergencyPause,
    TreasuryAutomation,
    ObservabilityUpgrade,
    AccessControlUpdate,
    ChangeManagementOverride,
    EducationProgramRefresh,
    Other,
}

impl ProposalTypeKind {
    #[cfg(test)]
    const CONTROLLED: [ProposalTypeKind; 11] = [
        ProposalTypeKind::ParameterChange,
        ProposalTypeKind::TreasuryAllocation,
        ProposalTypeKind::ProtocolUpgrade,
        ProposalTypeKind::NewMarketListing,
        ProposalTypeKind::FeeStructureChange,
        ProposalTypeKind::EmergencyPause,
        ProposalTypeKind::TreasuryAutomation,
        ProposalTypeKind::ObservabilityUpgrade,
        ProposalTypeKind::AccessControlUpdate,
        ProposalTypeKind::ChangeManagementOverride,
        ProposalTypeKind::EducationProgramRefresh,
    ];
}

impl ProposalType {
    fn kind(&self) -> ProposalTypeKind {
        match self {
            ProposalType::ParameterChange => ProposalTypeKind::ParameterChange,
            ProposalType::TreasuryAllocation => ProposalTypeKind::TreasuryAllocation,
            ProposalType::ProtocolUpgrade => ProposalTypeKind::ProtocolUpgrade,
            ProposalType::NewMarketListing => ProposalTypeKind::NewMarketListing,
            ProposalType::FeeStructureChange => ProposalTypeKind::FeeStructureChange,
            ProposalType::EmergencyPause => ProposalTypeKind::EmergencyPause,
            ProposalType::TreasuryAutomation => ProposalTypeKind::TreasuryAutomation,
            ProposalType::ObservabilityUpgrade => ProposalTypeKind::ObservabilityUpgrade,
            ProposalType::AccessControlUpdate => ProposalTypeKind::AccessControlUpdate,
            ProposalType::ChangeManagementOverride => ProposalTypeKind::ChangeManagementOverride,
            ProposalType::EducationProgramRefresh => ProposalTypeKind::EducationProgramRefresh,
            ProposalType::Other(_) => ProposalTypeKind::Other,
        }
    }
}

/// Information about who proposed the governance action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Proposer {
    /// AI-generated proposal with model details
    AI {
        model_id: String,
        confidence: f32,
        rationale: String,
    },
    /// Human-generated proposal
    Human { trader_id: TraderId },
    /// Hybrid proposal (AI-assisted human proposal)
    Hybrid {
        trader_id: TraderId,
        ai_model_id: String,
        ai_contribution: f32,
    },
}

/// Status of a governance proposal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Draft,
    Active,
    Passed,
    Rejected,
    Executed,
    Cancelled,
}

/// Vote tracking for a proposal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Votes {
    /// Votes in favor
    pub yes_votes: HashMap<TraderId, Vote>,
    /// Votes against
    pub no_votes: HashMap<TraderId, Vote>,
    /// Abstentions
    pub abstain_votes: HashMap<TraderId, Vote>,
    /// Total voting power that has participated
    pub total_voting_power: u64,
}

/// Individual vote information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vote {
    /// Trader who cast the vote
    pub voter: TraderId,
    /// Voting power (based on token holdings)
    pub voting_power: u64,
    /// Timestamp of the vote
    pub timestamp: u64,
    /// Optional reason for the vote
    pub reason: Option<String>,
}

/// Execution plan for an approved proposal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// Actions to execute
    pub actions: Vec<GovernanceAction>,
    /// Timestamp when execution should occur
    pub execution_time: u64,
    /// Whether execution requires manual confirmation
    pub requires_confirmation: bool,
}

/// Specific governance actions that can be executed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GovernanceAction {
    /// Change a protocol parameter
    SetParameter { key: String, value: String },
    /// Transfer tokens from treasury
    TransferTreasury {
        to: TraderId,
        token: TokenId,
        amount: u64,
    },
    /// Upgrade protocol code
    UpgradeProtocol {
        new_version: String,
        code_hash: String,
    },
    /// Add a new market
    AddMarket {
        base_token: TokenId,
        quote_token: TokenId,
    },
}

/// AI analysis and recommendations for a proposal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AIAnalysis {
    /// Predicted outcome probability
    pub predicted_outcome: f32,
    /// Risk assessment
    pub risk_score: f32,
    /// Estimated impact on key metrics
    pub impact_analysis: ImpactAnalysis,
    /// Similar historical proposals and their outcomes
    pub historical_comparison: Vec<HistoricalProposal>,
    /// Confidence level in the analysis
    pub confidence: f32,
}

/// Impact analysis of a proposal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    /// Predicted impact on liquidity
    pub liquidity_impact: f32,
    /// Predicted impact on trading volume
    pub volume_impact: f32,
    /// Predicted impact on user adoption
    pub adoption_impact: f32,
    /// Predicted impact on security
    pub security_impact: f32,
}

/// Historical proposal for comparison
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoricalProposal {
    /// ID of the historical proposal
    pub id: String,
    /// Similarity score to current proposal
    pub similarity: f32,
    /// Outcome of the historical proposal
    pub outcome: ProposalStatus,
    /// Key differences
    pub differences: Vec<String>,
}

/// Global DAO structure
#[derive(Debug, Clone)]
pub struct GlobalDAO {
    /// All governance proposals
    proposals: HashMap<String, Proposal>,
    /// DAO members and their voting power
    members: HashMap<TraderId, DAOMember>,
    /// Total voting power in the DAO
    total_voting_power: u64,
    /// Governance parameters
    parameters: GovernanceParameters,
    /// Governance control matrix derived from the reference dataset
    reference_index: Arc<GovernanceReferenceIndex>,
    /// AI models used for governance
    ai_models: HashMap<String, AIModel>,
    /// Emergency council members (with special powers)
    emergency_council: Vec<TraderId>,
}

/// DAO member information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DAOMember {
    /// Trader ID of the member
    pub trader_id: TraderId,
    /// Voting power based on token holdings
    pub voting_power: u64,
    /// Timestamp when they joined
    pub joined_at: u64,
    /// Whether they have special privileges
    pub is_council_member: bool,
}

/// Governance parameters that control DAO operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GovernanceParameters {
    /// Minimum voting power to create a proposal
    pub min_proposal_power: u64,
    /// Voting period in seconds
    pub voting_period: u64,
    /// Quorum percentage required for a proposal to pass
    pub quorum_percentage: u32,
    /// Threshold percentage of yes votes needed to pass
    pub threshold_percentage: u32,
    /// Delay before execution of passed proposals
    pub execution_delay: u64,
    /// Maximum number of active proposals
    pub max_active_proposals: u32,
}

/// AI model information for governance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AIModel {
    /// Unique identifier for the model
    pub id: String,
    /// Description of the model
    pub description: String,
    /// Performance metrics
    pub performance: ModelPerformance,
    /// Last updated timestamp
    pub last_updated: u64,
}

/// AI model performance metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelPerformance {
    /// Accuracy on historical data
    pub accuracy: f32,
    /// Number of proposals generated
    pub proposals_generated: u32,
    /// Number of proposals passed
    pub proposals_passed: u32,
    /// Average confidence score
    pub avg_confidence: f32,
}

/// Reference control matrix built from the governance CSV dataset.
#[derive(Debug, Clone)]
pub struct GovernanceReferenceIndex {
    scenarios: Vec<GovernanceScenario>,
    key_index: HashMap<ReferenceKey, usize>,
    test_index: HashMap<String, usize>,
}

static GOVERNANCE_REFERENCE_CACHE: OnceCell<Arc<GovernanceReferenceIndex>> = OnceCell::new();

impl GovernanceReferenceIndex {
    /// Loads the governance reference CSV and builds a searchable index.
    pub fn load() -> Result<Self, GovernanceReferenceError> {
        let scenarios = load_governance_reference()?;
        Ok(Self::from_scenarios(scenarios))
    }

    /// Returns a shared, cached instance of the governance reference index.
    pub fn shared() -> Result<Arc<Self>, GovernanceReferenceError> {
        GOVERNANCE_REFERENCE_CACHE
            .get_or_try_init(|| {
                let index = GovernanceReferenceIndex::load()?;
                Ok(Arc::new(index))
            })
            .map(Arc::clone)
    }

    fn from_scenarios(scenarios: Vec<GovernanceScenario>) -> Self {
        let mut key_index = HashMap::new();
        let mut test_index = HashMap::new();

        for (idx, scenario) in scenarios.iter().enumerate() {
            key_index.insert(ReferenceKey::from(scenario), idx);
            test_index.insert(scenario.test_name.clone(), idx);
        }

        Self {
            scenarios,
            key_index,
            test_index,
        }
    }

    /// Finds a control by its logical selector.
    pub fn find(
        &self,
        domain: &GovernanceDomain,
        component: &GovernanceComponent,
        behavior: &str,
        condition: &str,
    ) -> Option<&GovernanceScenario> {
        let key = ReferenceKey::new(domain, component, behavior, condition);
        self.key_index
            .get(&key)
            .and_then(|idx| self.scenarios.get(*idx))
    }

    /// Finds a control by the canonical test name.
    pub fn find_by_test_name(&self, test_name: &str) -> Option<&GovernanceScenario> {
        self.test_index
            .get(test_name)
            .and_then(|idx| self.scenarios.get(*idx))
    }

    /// Returns all scenarios for consumers that need to iterate through them.
    pub fn scenarios(&self) -> &[GovernanceScenario] {
        &self.scenarios
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ReferenceKey {
    domain: GovernanceDomain,
    component: GovernanceComponent,
    behavior: String,
    condition: String,
}

impl ReferenceKey {
    fn new(
        domain: &GovernanceDomain,
        component: &GovernanceComponent,
        behavior: &str,
        condition: &str,
    ) -> Self {
        Self {
            domain: domain.clone(),
            component: component.clone(),
            behavior: behavior.to_string(),
            condition: condition.to_string(),
        }
    }
}

impl From<&GovernanceScenario> for ReferenceKey {
    fn from(value: &GovernanceScenario) -> Self {
        Self {
            domain: value.domain.clone(),
            component: value.component.clone(),
            behavior: value.behavior.clone(),
            condition: value.condition.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProposalContext {
    pub proposal: Proposal,
    pub reference: Option<GovernanceScenario>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GovernanceControlMetrics {
    pub total_reference_controls: usize,
    pub entries: Vec<ControlMetricEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ControlMetricEntry {
    pub domain: GovernanceDomain,
    pub component: GovernanceComponent,
    pub owner: String,
    pub proposal_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GovernanceInsights {
    pub proposals: Vec<ProposalSummary>,
    pub control_metrics: GovernanceControlMetrics,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProposalSummary {
    pub id: String,
    pub title: String,
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub reference_owner: Option<String>,
    pub reference_tool: Option<String>,
    pub reference_metric: Option<String>,
    pub reference_evidence: Option<String>,
    pub reference_acknowledged: bool,
}

#[derive(Debug, Clone)]
struct RequiredReference {
    domain: GovernanceDomain,
    component: GovernanceComponent,
}

impl RequiredReference {
    fn new(domain: GovernanceDomain, component: GovernanceComponent) -> Self {
        Self { domain, component }
    }

    fn matches(&self, scenario: &GovernanceScenario) -> bool {
        scenario.domain == self.domain && scenario.component == self.component
    }
}

impl GlobalDAO {
    /// Create a new Global DAO with default parameters
    pub fn new() -> Self {
        Self::with_reference_data().expect("failed to load governance reference data for GlobalDAO")
    }

    /// Create a new Global DAO wired to the governance reference dataset.
    pub fn with_reference_data() -> Result<Self, GovernanceReferenceError> {
        let reference_index = GovernanceReferenceIndex::shared()?;

        Ok(Self {
            proposals: HashMap::new(),
            members: HashMap::new(),
            total_voting_power: 0,
            parameters: GovernanceParameters {
                min_proposal_power: 1000,
                voting_period: 604800,    // 7 days
                quorum_percentage: 10,    // 10%
                threshold_percentage: 51, // 51%
                execution_delay: 86400,   // 1 day
                max_active_proposals: 10,
            },
            reference_index,
            ai_models: HashMap::new(),
            emergency_council: Vec::new(),
        })
    }

    /// Add a new member to the DAO
    pub fn add_member(&mut self, trader_id: TraderId, voting_power: u64, is_council_member: bool) {
        let member = DAOMember {
            trader_id: trader_id.clone(),
            voting_power,
            joined_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_council_member,
        };

        self.total_voting_power += voting_power;
        self.members.insert(trader_id, member);
    }

    /// Returns the immutable governance reference index.
    pub fn reference_index(&self) -> &GovernanceReferenceIndex {
        self.reference_index.as_ref()
    }

    /// Attaches a governance reference control to an existing proposal.
    pub fn attach_reference_control(
        &mut self,
        proposal_id: &str,
        domain: GovernanceDomain,
        component: GovernanceComponent,
        behavior: &str,
        condition: &str,
    ) -> Result<(), GovernanceError> {
        let scenario = self
            .reference_index
            .find(&domain, &component, behavior, condition)
            .cloned()
            .ok_or(GovernanceError::ReferenceScenarioNotFound)?;

        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;
        proposal.reference_control = Some(scenario);
        proposal.reference_acknowledged = false;
        Ok(())
    }

    /// Retrieves the attached reference control for a proposal, if any.
    pub fn proposal_reference_control(
        &self,
        proposal_id: &str,
    ) -> Result<Option<&GovernanceScenario>, GovernanceError> {
        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;
        Ok(proposal.reference_control.as_ref())
    }

    /// Returns a full proposal context including its attached governance control.
    pub fn proposal_context(&self, proposal_id: &str) -> Result<ProposalContext, GovernanceError> {
        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?
            .clone();
        let reference = proposal.reference_control.clone();
        Ok(ProposalContext {
            proposal,
            reference,
        })
    }

    /// Marks that the designated owner has acknowledged the control backing a proposal.
    pub fn acknowledge_reference_owner(
        &mut self,
        proposal_id: &str,
        owner_name: &str,
    ) -> Result<(), GovernanceError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;
        let control = proposal
            .reference_control
            .as_ref()
            .ok_or(GovernanceError::ReferenceControlMissing)?;

        if !control.enrichment.owner.eq_ignore_ascii_case(owner_name) {
            return Err(GovernanceError::ReferenceOwnerMismatch(
                owner_name.to_string(),
            ));
        }

        proposal.reference_acknowledged = true;
        Ok(())
    }

    /// Aggregates proposal counts by governance control for dashboards/reporting.
    pub fn governance_control_metrics(&self) -> GovernanceControlMetrics {
        let mut accum: HashMap<(GovernanceDomain, GovernanceComponent), ControlMetricEntry> =
            HashMap::new();

        for proposal in self.proposals.values() {
            if let Some(control) = &proposal.reference_control {
                let key = (control.domain.clone(), control.component.clone());
                let entry = accum.entry(key).or_insert_with(|| ControlMetricEntry {
                    domain: control.domain.clone(),
                    component: control.component.clone(),
                    owner: control.enrichment.owner.clone(),
                    proposal_ids: Vec::new(),
                });
                entry.proposal_ids.push(proposal.id.clone());
            }
        }

        GovernanceControlMetrics {
            total_reference_controls: self.reference_index.scenarios().len(),
            entries: accum.into_values().collect(),
        }
    }

    /// Builds a snapshot suitable for APIs or dashboards with enriched metadata.
    pub fn governance_insights(&self) -> GovernanceInsights {
        let proposals = self
            .proposals
            .values()
            .map(|proposal| {
                let (owner, tool, metric, evidence) = proposal
                    .reference_control
                    .as_ref()
                    .map(|control| {
                        (
                            Some(control.enrichment.owner.clone()),
                            Some(control.enrichment.tool.clone()),
                            Some(control.enrichment.metric.clone()),
                            Some(control.enrichment.evidence.clone()),
                        )
                    })
                    .unwrap_or((None, None, None, None));

                ProposalSummary {
                    id: proposal.id.clone(),
                    title: proposal.title.clone(),
                    proposal_type: proposal.proposal_type.clone(),
                    status: proposal.status.clone(),
                    reference_owner: owner,
                    reference_tool: tool,
                    reference_metric: metric,
                    reference_evidence: evidence,
                    reference_acknowledged: proposal.reference_acknowledged,
                }
            })
            .collect();

        GovernanceInsights {
            proposals,
            control_metrics: self.governance_control_metrics(),
        }
    }

    fn enforce_reference_policy(proposal: &Proposal) -> Result<(), GovernanceError> {
        let Some(required) = Self::required_reference_for(&proposal.proposal_type) else {
            return Ok(());
        };

        let reference = proposal
            .reference_control
            .as_ref()
            .ok_or(GovernanceError::ReferenceControlMissing)?;

        if required.matches(reference) {
            Ok(())
        } else {
            Err(GovernanceError::ReferenceControlMismatch)
        }
    }

    fn required_reference_for(proposal_type: &ProposalType) -> Option<RequiredReference> {
        Self::required_reference_for_kind(proposal_type.kind())
    }

    fn required_reference_for_kind(kind: ProposalTypeKind) -> Option<RequiredReference> {
        match kind {
            ProposalTypeKind::ParameterChange => Some(RequiredReference::new(
                GovernanceDomain::GovernancePolicyFramework,
                GovernanceComponent::PolicyEngine,
            )),
            ProposalTypeKind::TreasuryAllocation => Some(RequiredReference::new(
                GovernanceDomain::RiskExceptionManagement,
                GovernanceComponent::RiskRegistry,
            )),
            ProposalTypeKind::ProtocolUpgrade => Some(RequiredReference::new(
                GovernanceDomain::AuditEvidenceManagement,
                GovernanceComponent::AuditLogger,
            )),
            ProposalTypeKind::EmergencyPause => Some(RequiredReference::new(
                GovernanceDomain::DaoOnChainGovernance,
                GovernanceComponent::DaoGovernor,
            )),
            ProposalTypeKind::NewMarketListing => Some(RequiredReference::new(
                GovernanceDomain::ComplianceRegulatoryAlignment,
                GovernanceComponent::ComplianceMapper,
            )),
            ProposalTypeKind::FeeStructureChange => Some(RequiredReference::new(
                GovernanceDomain::PolicyAsCodeAutomation,
                GovernanceComponent::RegoValidator,
            )),
            ProposalTypeKind::TreasuryAutomation => Some(RequiredReference::new(
                GovernanceDomain::RiskExceptionManagement,
                GovernanceComponent::RiskRegistry,
            )),
            ProposalTypeKind::ObservabilityUpgrade => Some(RequiredReference::new(
                GovernanceDomain::TransparencyReporting,
                GovernanceComponent::ReportDashboard,
            )),
            ProposalTypeKind::AccessControlUpdate => Some(RequiredReference::new(
                GovernanceDomain::AccessAuthorizationGovernance,
                GovernanceComponent::RoleManager,
            )),
            ProposalTypeKind::ChangeManagementOverride => Some(RequiredReference::new(
                GovernanceDomain::ChangeManagementApprovalFlow,
                GovernanceComponent::ApprovalGate,
            )),
            ProposalTypeKind::EducationProgramRefresh => Some(RequiredReference::new(
                GovernanceDomain::EducationCultureAccountability,
                GovernanceComponent::PolicyEngine,
            )),
            ProposalTypeKind::Other => None,
        }
    }

    /// Create a new governance proposal
    pub fn create_proposal(
        &mut self,
        title: String,
        description: String,
        proposal_type: ProposalType,
        proposer: Proposer,
    ) -> Result<String, GovernanceError> {
        // Check if proposer has sufficient voting power
        let proposer_power = match &proposer {
            Proposer::Human { trader_id } | Proposer::Hybrid { trader_id, .. } => self
                .members
                .get(trader_id)
                .map(|m| m.voting_power)
                .unwrap_or(0),
            Proposer::AI { .. } => 0, // AI proposals don't need voting power
        };

        if proposer_power < self.parameters.min_proposal_power
            && !matches!(proposer, Proposer::AI { .. })
        {
            return Err(GovernanceError::InsufficientVotingPower);
        }

        // Check active proposal limit
        let active_proposals = self
            .proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Active || p.status == ProposalStatus::Draft)
            .count();

        if active_proposals >= self.parameters.max_active_proposals as usize {
            return Err(GovernanceError::TooManyActiveProposals);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let proposal_id = format!("proposal_{}", now);

        let proposal = Proposal {
            id: proposal_id.clone(),
            title,
            description,
            proposal_type,
            proposer,
            created_at: now,
            voting_start: now + 3600, // Voting starts in 1 hour
            voting_end: now + 3600 + self.parameters.voting_period,
            status: ProposalStatus::Draft,
            votes: Votes {
                yes_votes: HashMap::new(),
                no_votes: HashMap::new(),
                abstain_votes: HashMap::new(),
                total_voting_power: 0,
            },
            execution_plan: None,
            ai_analysis: None,
            reference_control: None,
            reference_acknowledged: false,
        };

        self.proposals.insert(proposal_id.clone(), proposal);
        Ok(proposal_id)
    }

    /// Submit a proposal for voting
    pub fn submit_proposal(&mut self, proposal_id: &str) -> Result<(), GovernanceError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Draft {
            return Err(GovernanceError::ProposalNotInDraft);
        }

        Self::enforce_reference_policy(proposal)?;

        proposal.status = ProposalStatus::Active;
        Ok(())
    }

    /// Vote on a proposal
    pub fn vote(
        &mut self,
        proposal_id: &str,
        voter_id: &TraderId,
        support: bool,
        voting_power: u64,
        reason: Option<String>,
    ) -> Result<(), GovernanceError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::ProposalNotActive);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now < proposal.voting_start {
            return Err(GovernanceError::VotingNotStarted);
        }

        if now > proposal.voting_end {
            return Err(GovernanceError::VotingEnded);
        }

        let member = self
            .members
            .get(voter_id)
            .ok_or(GovernanceError::NotDAOMember)?;

        if voting_power > member.voting_power {
            return Err(GovernanceError::InsufficientVotingPower);
        }

        let vote = Vote {
            voter: voter_id.clone(),
            voting_power,
            timestamp: now,
            reason,
        };

        if support {
            proposal.votes.yes_votes.insert(voter_id.clone(), vote);
        } else {
            proposal.votes.no_votes.insert(voter_id.clone(), vote);
        }

        proposal.votes.total_voting_power += voting_power;
        Ok(())
    }

    /// Cast an abstain vote
    pub fn abstain_vote(
        &mut self,
        proposal_id: &str,
        voter_id: &TraderId,
        voting_power: u64,
        reason: Option<String>,
    ) -> Result<(), GovernanceError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::ProposalNotActive);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now < proposal.voting_start {
            return Err(GovernanceError::VotingNotStarted);
        }

        if now > proposal.voting_end {
            return Err(GovernanceError::VotingEnded);
        }

        let member = self
            .members
            .get(voter_id)
            .ok_or(GovernanceError::NotDAOMember)?;

        if voting_power > member.voting_power {
            return Err(GovernanceError::InsufficientVotingPower);
        }

        let vote = Vote {
            voter: voter_id.clone(),
            voting_power,
            timestamp: now,
            reason,
        };

        proposal.votes.abstain_votes.insert(voter_id.clone(), vote);
        proposal.votes.total_voting_power += voting_power;
        Ok(())
    }

    /// Tally votes for a proposal and determine outcome
    pub fn tally_votes(&mut self, proposal_id: &str) -> Result<ProposalStatus, GovernanceError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now < proposal.voting_end {
            return Err(GovernanceError::VotingNotEnded);
        }

        if proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::ProposalNotActive);
        }

        // Calculate total votes
        let yes_votes: u64 = proposal
            .votes
            .yes_votes
            .values()
            .map(|vote| vote.voting_power)
            .sum();

        let no_votes: u64 = proposal
            .votes
            .no_votes
            .values()
            .map(|vote| vote.voting_power)
            .sum();

        let total_votes = yes_votes + no_votes;

        // Check quorum
        let quorum_required =
            (self.total_voting_power * self.parameters.quorum_percentage as u64) / 100;
        if proposal.votes.total_voting_power < quorum_required {
            proposal.status = ProposalStatus::Rejected;
            return Ok(ProposalStatus::Rejected);
        }

        // Check threshold
        let threshold = (total_votes * self.parameters.threshold_percentage as u64) / 100;
        if yes_votes > threshold {
            if proposal.reference_control.is_some() && !proposal.reference_acknowledged {
                return Err(GovernanceError::ReferenceOwnerAcknowledgementMissing);
            }
            proposal.status = ProposalStatus::Passed;
            Ok(ProposalStatus::Passed)
        } else {
            proposal.status = ProposalStatus::Rejected;
            Ok(ProposalStatus::Rejected)
        }
    }

    /// Add an AI model to the governance system
    pub fn add_ai_model(&mut self, model: AIModel) {
        self.ai_models.insert(model.id.clone(), model);
    }

    /// Add a member to the emergency council
    pub fn add_emergency_council_member(&mut self, trader_id: TraderId) {
        if !self.emergency_council.contains(&trader_id) {
            self.emergency_council.push(trader_id);
        }
    }

    /// Emergency pause a proposal (council-only function)
    pub fn emergency_pause(
        &mut self,
        proposal_id: &str,
        council_member: &TraderId,
    ) -> Result<(), GovernanceError> {
        if !self.emergency_council.contains(council_member) {
            return Err(GovernanceError::NotEmergencyCouncilMember);
        }

        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        proposal.status = ProposalStatus::Cancelled;
        Ok(())
    }

    /// Get a proposal by ID
    pub fn get_proposal(&self, proposal_id: &str) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }

    /// Get all active proposals
    pub fn get_active_proposals(&self) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Active)
            .collect()
    }

    /// Get proposals by proposer type
    pub fn get_proposals_by_type(&self, proposer_type: &str) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| match &p.proposer {
                Proposer::AI { .. } if proposer_type == "AI" => true,
                Proposer::Human { .. } if proposer_type == "Human" => true,
                Proposer::Hybrid { .. } if proposer_type == "Hybrid" => true,
                _ => false,
            })
            .collect()
    }
}

impl Default for GlobalDAO {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during governance operations
#[derive(Debug, Error)]
pub enum GovernanceError {
    #[error("Proposal not found")]
    ProposalNotFound,
    #[error("Proposal not in draft status")]
    ProposalNotInDraft,
    #[error("Proposal not active")]
    ProposalNotActive,
    #[error("Voting has not started yet")]
    VotingNotStarted,
    #[error("Voting has ended")]
    VotingEnded,
    #[error("Voting has not ended yet")]
    VotingNotEnded,
    #[error("Insufficient voting power")]
    InsufficientVotingPower,
    #[error("Not a DAO member")]
    NotDAOMember,
    #[error("Too many active proposals")]
    TooManyActiveProposals,
    #[error("Not an emergency council member")]
    NotEmergencyCouncilMember,
    #[error("reference scenario not found for the requested selector")]
    ReferenceScenarioNotFound,
    #[error("proposal requires an attached governance reference control")]
    ReferenceControlMissing,
    #[error("attached governance reference control does not satisfy policy requirements")]
    ReferenceControlMismatch,
    #[error("reference control owner acknowledgement missing")]
    ReferenceOwnerAcknowledgementMissing,
    #[error("owner {0} is not authorized to acknowledge this reference control")]
    ReferenceOwnerMismatch(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const POLICY_REFERENCE_TEST: &str = "test_governance_compliance__governance_policy_and_framework__policy_engine__defines_policy__during_commit";
    const EXPECTED_COMPONENT_COVERAGE: &[(GovernanceDomain, GovernanceComponent)] = &[
        (
            GovernanceDomain::GovernancePolicyFramework,
            GovernanceComponent::PolicyEngine,
        ),
        (
            GovernanceDomain::AccessAuthorizationGovernance,
            GovernanceComponent::RoleManager,
        ),
        (
            GovernanceDomain::ChangeManagementApprovalFlow,
            GovernanceComponent::ApprovalGate,
        ),
        (
            GovernanceDomain::ComplianceRegulatoryAlignment,
            GovernanceComponent::ComplianceMapper,
        ),
        (
            GovernanceDomain::RiskExceptionManagement,
            GovernanceComponent::RiskRegistry,
        ),
        (
            GovernanceDomain::AuditEvidenceManagement,
            GovernanceComponent::AuditLogger,
        ),
        (
            GovernanceDomain::PolicyAsCodeAutomation,
            GovernanceComponent::RegoValidator,
        ),
        (
            GovernanceDomain::TransparencyReporting,
            GovernanceComponent::ReportDashboard,
        ),
        (
            GovernanceDomain::DaoOnChainGovernance,
            GovernanceComponent::DaoGovernor,
        ),
        (
            GovernanceDomain::EducationCultureAccountability,
            GovernanceComponent::PolicyEngine,
        ),
    ];

    fn attach_reference(
        dao: &mut GlobalDAO,
        proposal_id: &str,
        domain: GovernanceDomain,
        component: GovernanceComponent,
    ) {
        dao.attach_reference_control(
            proposal_id,
            domain,
            component,
            "defines_policy",
            "during_commit",
        )
        .expect("failed to attach governance control");
    }

    fn attach_policy_reference(dao: &mut GlobalDAO, proposal_id: &str) {
        attach_reference(
            dao,
            proposal_id,
            GovernanceDomain::GovernancePolicyFramework,
            GovernanceComponent::PolicyEngine,
        );
    }

    fn fast_forward_to_voting_end(dao: &mut GlobalDAO, proposal_id: &str) {
        if let Some(proposal) = dao.proposals.get_mut(proposal_id) {
            proposal.voting_start = 0;
            proposal.voting_end = 0;
        }
    }

    #[test]
    fn test_global_dao_creation() {
        let dao = GlobalDAO::new();
        assert!(dao.proposals.is_empty());
        assert!(dao.members.is_empty());
        assert_eq!(dao.total_voting_power, 0);
        assert!(!dao.reference_index().scenarios().is_empty());
    }

    #[test]
    fn test_add_member() {
        let mut dao = GlobalDAO::new();
        let trader_id = "trader1".to_string();

        dao.add_member(trader_id.clone(), 1000, false);

        assert_eq!(dao.members.len(), 1);
        assert_eq!(dao.total_voting_power, 1000);

        let member = dao.members.get(&trader_id).unwrap();
        assert_eq!(member.trader_id, trader_id);
        assert_eq!(member.voting_power, 1000);
        assert!(!member.is_council_member);
    }

    #[test]
    fn test_create_proposal() {
        let mut dao = GlobalDAO::new();
        let trader_id = "trader1".to_string();

        // Add member with sufficient voting power
        dao.add_member(trader_id.clone(), 2000, false);

        let proposal_id = dao.create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            ProposalType::ParameterChange,
            Proposer::Human { trader_id },
        );

        assert!(proposal_id.is_ok());
        let proposal_id = proposal_id.unwrap();

        assert_eq!(dao.proposals.len(), 1);
        let proposal = dao.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.title, "Test Proposal");
        assert_eq!(proposal.status, ProposalStatus::Draft);
    }

    #[test]
    fn test_submit_proposal() {
        let mut dao = GlobalDAO::new();
        let trader_id = "trader1".to_string();

        // Add member
        dao.add_member(trader_id.clone(), 2000, false);

        // Create proposal
        let proposal_id = dao
            .create_proposal(
                "Test Proposal".to_string(),
                "This is a test proposal".to_string(),
                ProposalType::ParameterChange,
                Proposer::Human { trader_id },
            )
            .unwrap();

        attach_policy_reference(&mut dao, &proposal_id);

        // Submit proposal
        assert!(dao.submit_proposal(&proposal_id).is_ok());

        let proposal = dao.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Active);
    }

    #[test]
    fn test_voting() {
        let mut dao = GlobalDAO::new();
        let trader1_id = "trader1".to_string();
        let trader2_id = "trader2".to_string();

        // Add members
        dao.add_member(trader1_id.clone(), 1000, false);
        dao.add_member(trader2_id.clone(), 500, false);

        // Create and submit proposal
        let proposal_id = dao
            .create_proposal(
                "Test Proposal".to_string(),
                "This is a test proposal".to_string(),
                ProposalType::ParameterChange,
                Proposer::Human {
                    trader_id: trader1_id.clone(),
                },
            )
            .unwrap();

        attach_policy_reference(&mut dao, &proposal_id);
        dao.submit_proposal(&proposal_id).unwrap();

        // Vote yes
        assert!(dao
            .vote(&proposal_id, &trader1_id, true, 1000, None)
            .is_ok());

        // Vote no
        assert!(dao
            .vote(&proposal_id, &trader2_id, false, 500, None)
            .is_ok());

        let proposal = dao.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.votes.yes_votes.len(), 1);
        assert_eq!(proposal.votes.no_votes.len(), 1);
        assert_eq!(proposal.votes.total_voting_power, 1500);
    }

    #[test]
    fn test_abstain_vote() {
        let mut dao = GlobalDAO::new();
        let trader_id = "trader1".to_string();

        // Add member
        dao.add_member(trader_id.clone(), 1000, false);

        // Create and submit proposal
        let proposal_id = dao
            .create_proposal(
                "Test Proposal".to_string(),
                "This is a test proposal".to_string(),
                ProposalType::ParameterChange,
                Proposer::Human {
                    trader_id: trader_id.clone(),
                },
            )
            .unwrap();

        attach_policy_reference(&mut dao, &proposal_id);
        dao.submit_proposal(&proposal_id).unwrap();

        // Abstain vote
        assert!(dao
            .abstain_vote(&proposal_id, &trader_id, 1000, None)
            .is_ok());

        let proposal = dao.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.votes.abstain_votes.len(), 1);
        assert_eq!(proposal.votes.total_voting_power, 1000);
    }

    #[test]
    fn test_ai_proposal() {
        let mut dao = GlobalDAO::new();

        // Create AI proposal (no voting power required)
        let proposal_id = dao.create_proposal(
            "AI Generated Proposal".to_string(),
            "This proposal was generated by AI".to_string(),
            ProposalType::ParameterChange,
            Proposer::AI {
                model_id: "model_1".to_string(),
                confidence: 0.95,
                rationale: "Based on market analysis".to_string(),
            },
        );

        assert!(proposal_id.is_ok());
        let proposal_id = proposal_id.unwrap();

        let proposal = dao.get_proposal(&proposal_id).unwrap();
        match &proposal.proposer {
            Proposer::AI {
                model_id,
                confidence,
                ..
            } => {
                assert_eq!(model_id, "model_1");
                assert_eq!(*confidence, 0.95);
            }
            _ => panic!("Expected AI proposer"),
        }
    }

    #[test]
    fn test_emergency_council() {
        let mut dao = GlobalDAO::new();
        let council_member = "council_member".to_string();
        let regular_member = "regular_member".to_string();

        // Add members
        dao.add_member(council_member.clone(), 1000, true);
        dao.add_member(regular_member.clone(), 1000, false);

        // Add to emergency council
        dao.add_emergency_council_member(council_member.clone());

        // Create and submit proposal
        let proposal_id = dao
            .create_proposal(
                "Test Proposal".to_string(),
                "This is a test proposal".to_string(),
                ProposalType::ParameterChange,
                Proposer::Human {
                    trader_id: regular_member.clone(),
                },
            )
            .unwrap();

        attach_policy_reference(&mut dao, &proposal_id);
        dao.submit_proposal(&proposal_id).unwrap();

        // Emergency pause by council member
        assert!(dao.emergency_pause(&proposal_id, &council_member).is_ok());

        let proposal = dao.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Cancelled);

        // Try emergency pause by regular member (should fail)
        let regular_member2 = "regular_member".to_string();
        let proposal_id2 = dao
            .create_proposal(
                "Test Proposal 2".to_string(),
                "This is another test proposal".to_string(),
                ProposalType::ParameterChange,
                Proposer::Human {
                    trader_id: regular_member2.clone(),
                },
            )
            .unwrap();

        attach_policy_reference(&mut dao, &proposal_id2);
        dao.submit_proposal(&proposal_id2).unwrap();

        let result = dao.emergency_pause(&proposal_id2, &regular_member2);
        assert!(result.is_err());
    }

    #[test]
    fn test_proposal_filtering() {
        let mut dao = GlobalDAO::new();
        let trader_id = "trader1".to_string();

        // Add member
        dao.add_member(trader_id.clone(), 1000, false);

        // Create different types of proposals
        let ai_proposal_id = dao
            .create_proposal(
                "AI Proposal".to_string(),
                "AI generated proposal".to_string(),
                ProposalType::ParameterChange,
                Proposer::AI {
                    model_id: "model_1".to_string(),
                    confidence: 0.9,
                    rationale: "AI analysis".to_string(),
                },
            )
            .unwrap();

        let human_proposal_id = dao
            .create_proposal(
                "Human Proposal".to_string(),
                "Human generated proposal".to_string(),
                ProposalType::ParameterChange,
                Proposer::Human {
                    trader_id: trader_id.clone(),
                },
            )
            .unwrap();

        // Check proposal filtering
        let ai_proposals = dao.get_proposals_by_type("AI");
        assert_eq!(ai_proposals.len(), 1);

        let human_proposals = dao.get_proposals_by_type("Human");
        assert_eq!(human_proposals.len(), 1);

        let hybrid_proposals = dao.get_proposals_by_type("Hybrid");
        assert_eq!(hybrid_proposals.len(), 0);
    }

    #[test]
    fn test_attach_reference_control_to_proposal() {
        let mut dao = GlobalDAO::new();
        let trader_id = "trader_reference".to_string();

        dao.add_member(trader_id.clone(), 2_000, false);
        let proposal_id = dao
            .create_proposal(
                "Policy enforcement".to_string(),
                "Ensure policy engine enforces definitions during commits".to_string(),
                ProposalType::ParameterChange,
                Proposer::Human {
                    trader_id: trader_id.clone(),
                },
            )
            .unwrap();

        dao.attach_reference_control(
            &proposal_id,
            GovernanceDomain::GovernancePolicyFramework,
            GovernanceComponent::PolicyEngine,
            "defines_policy",
            "during_commit",
        )
        .expect("failed to attach governance control");

        let proposal = dao.get_proposal(&proposal_id).unwrap();
        let reference = proposal
            .reference_control
            .as_ref()
            .expect("missing reference control");

        assert_eq!(reference.test_name, POLICY_REFERENCE_TEST);
        assert_eq!(reference.enrichment.owner, "Security Governance Lead");
        assert_eq!(
            reference.enrichment.metric, "policy_coverage_pct",
            "metric should mirror the CSV data"
        );

        let lookup = dao
            .proposal_reference_control(&proposal_id)
            .expect("proposal exists")
            .expect("reference control should be present");
        assert_eq!(lookup.test_name, reference.test_name);
    }

    #[test]
    fn test_submit_requires_reference_for_parameter_change() {
        let mut dao = GlobalDAO::new();
        let trader_id = "trader_policy".to_string();

        dao.add_member(trader_id.clone(), 2_000, false);
        let proposal_id = dao
            .create_proposal(
                "Policy change".to_string(),
                "Adjusts policy enforcement".to_string(),
                ProposalType::ParameterChange,
                Proposer::Human {
                    trader_id: trader_id.clone(),
                },
            )
            .unwrap();

        let err = dao.submit_proposal(&proposal_id).unwrap_err();
        assert!(matches!(err, GovernanceError::ReferenceControlMissing));

        attach_policy_reference(&mut dao, &proposal_id);
        assert!(dao.submit_proposal(&proposal_id).is_ok());
    }

    #[test]
    fn test_reference_control_mismatch_blocks_submission() {
        let mut dao = GlobalDAO::new();
        let trader_id = "trader_bad_reference".to_string();
        dao.add_member(trader_id.clone(), 2_000, false);

        let proposal_id = dao
            .create_proposal(
                "Policy change".to_string(),
                "Adjust a parameter".to_string(),
                ProposalType::ParameterChange,
                Proposer::Human {
                    trader_id: trader_id.clone(),
                },
            )
            .unwrap();

        // Manually attach a mismatched control (report_dashboard) to simulate a violation.
        let mismatched_control = dao
            .reference_index()
            .scenarios()
            .iter()
            .find(|scenario| scenario.component == GovernanceComponent::ReportDashboard)
            .cloned()
            .expect("expected to find report_dashboard scenario");

        dao.proposals
            .get_mut(&proposal_id)
            .unwrap()
            .reference_control = Some(mismatched_control);

        let err = dao.submit_proposal(&proposal_id).unwrap_err();
        assert!(matches!(err, GovernanceError::ReferenceControlMismatch));
    }

    #[test]
    fn test_fee_structure_change_requires_policy_as_code_reference() {
        let mut dao = GlobalDAO::new();
        let trader_id = "fee_reference".to_string();
        dao.add_member(trader_id.clone(), 2_000, false);

        let proposal_id = dao
            .create_proposal(
                "Update fee tiers".to_string(),
                "Align fee model with policy gates".to_string(),
                ProposalType::FeeStructureChange,
                Proposer::Human {
                    trader_id: trader_id.clone(),
                },
            )
            .unwrap();

        let err = dao.submit_proposal(&proposal_id).unwrap_err();
        assert!(matches!(err, GovernanceError::ReferenceControlMissing));

        attach_reference(
            &mut dao,
            &proposal_id,
            GovernanceDomain::PolicyAsCodeAutomation,
            GovernanceComponent::RegoValidator,
        );
        assert!(dao.submit_proposal(&proposal_id).is_ok());
    }

    #[test]
    fn test_new_market_listing_requires_compliance_reference() {
        let mut dao = GlobalDAO::new();
        let trader_id = "listing_reference".to_string();
        dao.add_member(trader_id.clone(), 2_000, false);

        let proposal_id = dao
            .create_proposal(
                "List new market".to_string(),
                "Add asset pair pending compliance review".to_string(),
                ProposalType::NewMarketListing,
                Proposer::Human {
                    trader_id: trader_id.clone(),
                },
            )
            .unwrap();

        let err = dao.submit_proposal(&proposal_id).unwrap_err();
        assert!(matches!(err, GovernanceError::ReferenceControlMissing));

        attach_reference(
            &mut dao,
            &proposal_id,
            GovernanceDomain::ComplianceRegulatoryAlignment,
            GovernanceComponent::ComplianceMapper,
        );
        assert!(dao.submit_proposal(&proposal_id).is_ok());
    }

    #[test]
    fn test_treasury_automation_requires_risk_reference() {
        let mut dao = GlobalDAO::new();
        let trader_id = "treasury_automation".to_string();
        dao.add_member(trader_id.clone(), 2_000, false);

        let proposal_id = dao
            .create_proposal(
                "Automate treasury operations".to_string(),
                "Tie payouts to automated policies".to_string(),
                ProposalType::TreasuryAutomation,
                Proposer::Human {
                    trader_id: trader_id.clone(),
                },
            )
            .unwrap();

        let err = dao.submit_proposal(&proposal_id).unwrap_err();
        assert!(matches!(err, GovernanceError::ReferenceControlMissing));

        attach_reference(
            &mut dao,
            &proposal_id,
            GovernanceDomain::RiskExceptionManagement,
            GovernanceComponent::RiskRegistry,
        );
        assert!(dao.submit_proposal(&proposal_id).is_ok());
    }

    #[test]
    fn test_observability_upgrade_requires_reporting_reference() {
        let mut dao = GlobalDAO::new();
        let trader_id = "observability_reference".to_string();
        dao.add_member(trader_id.clone(), 2_000, false);

        let proposal_id = dao
            .create_proposal(
                "Upgrade observability stack".to_string(),
                "Add mandatory dashboards and alerts".to_string(),
                ProposalType::ObservabilityUpgrade,
                Proposer::Human {
                    trader_id: trader_id.clone(),
                },
            )
            .unwrap();

        let err = dao.submit_proposal(&proposal_id).unwrap_err();
        assert!(matches!(err, GovernanceError::ReferenceControlMissing));

        attach_reference(
            &mut dao,
            &proposal_id,
            GovernanceDomain::TransparencyReporting,
            GovernanceComponent::ReportDashboard,
        );
        assert!(dao.submit_proposal(&proposal_id).is_ok());
    }

    #[test]
    fn test_access_control_update_requires_role_manager_reference() {
        let mut dao = GlobalDAO::new();
        let trader_id = "access_control_reference".to_string();
        dao.add_member(trader_id.clone(), 2_000, false);

        let proposal_id = dao
            .create_proposal(
                "Update IAM policies".to_string(),
                "Adjusts role hierarchy and RBAC mappings".to_string(),
                ProposalType::AccessControlUpdate,
                Proposer::Human {
                    trader_id: trader_id.clone(),
                },
            )
            .unwrap();

        let err = dao.submit_proposal(&proposal_id).unwrap_err();
        assert!(matches!(err, GovernanceError::ReferenceControlMissing));

        attach_reference(
            &mut dao,
            &proposal_id,
            GovernanceDomain::AccessAuthorizationGovernance,
            GovernanceComponent::RoleManager,
        );
        assert!(dao.submit_proposal(&proposal_id).is_ok());
    }

    #[test]
    fn test_change_management_override_requires_approval_gate_reference() {
        let mut dao = GlobalDAO::new();
        let trader_id = "change_override_reference".to_string();
        dao.add_member(trader_id.clone(), 2_000, false);

        let proposal_id = dao
            .create_proposal(
                "Override change freeze".to_string(),
                "Bypass the normal approval gate for emergency fix".to_string(),
                ProposalType::ChangeManagementOverride,
                Proposer::Human {
                    trader_id: trader_id.clone(),
                },
            )
            .unwrap();

        let err = dao.submit_proposal(&proposal_id).unwrap_err();
        assert!(matches!(err, GovernanceError::ReferenceControlMissing));

        attach_reference(
            &mut dao,
            &proposal_id,
            GovernanceDomain::ChangeManagementApprovalFlow,
            GovernanceComponent::ApprovalGate,
        );
        assert!(dao.submit_proposal(&proposal_id).is_ok());
    }

    #[test]
    fn test_education_program_refresh_requires_training_reference() {
        let mut dao = GlobalDAO::new();
        let trader_id = "education_refresh".to_string();
        dao.add_member(trader_id.clone(), 2_000, false);

        let proposal_id = dao
            .create_proposal(
                "Refresh security education program".to_string(),
                "Update LMS modules and accountability checkpoints".to_string(),
                ProposalType::EducationProgramRefresh,
                Proposer::Human {
                    trader_id: trader_id.clone(),
                },
            )
            .unwrap();

        let err = dao.submit_proposal(&proposal_id).unwrap_err();
        assert!(matches!(err, GovernanceError::ReferenceControlMissing));

        attach_reference(
            &mut dao,
            &proposal_id,
            GovernanceDomain::EducationCultureAccountability,
            GovernanceComponent::PolicyEngine,
        );
        assert!(dao.submit_proposal(&proposal_id).is_ok());
    }

    #[test]
    fn test_reference_owner_acknowledgement_required_for_pass() {
        let mut dao = GlobalDAO::new();
        let proposer = "owner_ack".to_string();

        dao.add_member(proposer.clone(), 2_000, false);

        let proposal_id = dao
            .create_proposal(
                "Owner ack enforcement".to_string(),
                "Ties proposal completion to reference owner".to_string(),
                ProposalType::ParameterChange,
                Proposer::Human {
                    trader_id: proposer.clone(),
                },
            )
            .unwrap();

        attach_policy_reference(&mut dao, &proposal_id);
        dao.submit_proposal(&proposal_id).unwrap();
        fast_forward_to_voting_end(&mut dao, &proposal_id);

        dao.vote(&proposal_id, &proposer, true, 1_000, None)
            .unwrap();

        let err = dao.tally_votes(&proposal_id).unwrap_err();
        assert!(matches!(
            err,
            GovernanceError::ReferenceOwnerAcknowledgementMissing
        ));

        dao.acknowledge_reference_owner(&proposal_id, "Security Governance Lead")
            .expect("owner should acknowledge control");

        let status = dao.tally_votes(&proposal_id).unwrap();
        assert_eq!(status, ProposalStatus::Passed);
    }

    #[test]
    fn test_governance_insights_surface_reference_metadata() {
        let mut dao = GlobalDAO::new();
        let proposer = "insights".to_string();
        dao.add_member(proposer.clone(), 2_000, false);

        let proposal_id = dao
            .create_proposal(
                "Insights".to_string(),
                "Expose governance metadata".to_string(),
                ProposalType::ParameterChange,
                Proposer::Human {
                    trader_id: proposer.clone(),
                },
            )
            .unwrap();

        attach_policy_reference(&mut dao, &proposal_id);
        dao.submit_proposal(&proposal_id).unwrap();
        dao.acknowledge_reference_owner(&proposal_id, "Security Governance Lead")
            .expect("owner should be able to acknowledge");

        let insights = dao.governance_insights();
        let summary = insights
            .proposals
            .into_iter()
            .find(|summary| summary.id == proposal_id)
            .expect("summary should include proposal");
        assert_eq!(
            summary.reference_owner.as_deref(),
            Some("Security Governance Lead")
        );
        assert!(summary.reference_acknowledged);
        assert!(
            insights
                .control_metrics
                .entries
                .iter()
                .any(|entry| entry.owner == "Security Governance Lead"),
            "metrics should include enrichment owner"
        );
    }

    #[test]
    fn test_required_reference_matrix_covers_expected_components() {
        let mut coverage = HashSet::new();
        for kind in ProposalTypeKind::CONTROLLED.iter().copied() {
            let reference = GlobalDAO::required_reference_for_kind(kind)
                .expect("controlled proposal kind should define a reference control");
            coverage.insert((reference.domain, reference.component));
        }

        for (domain, component) in EXPECTED_COMPONENT_COVERAGE {
            assert!(
                coverage.contains(&(domain.clone(), component.clone())),
                "missing governance coverage for {:?}/{:?}",
                domain,
                component
            );
        }
    }
}
