//! AI Governance implementation for the DEX-OS core engine
//!
//! This module implements the Priority 3 features from DEX-OS-V2.csv:
//! - AI Governance (AI Proposals, Global DAO)
//!
//! It provides functionality for AI-driven governance proposals and
//! global decentralized autonomous organization (DAO) operations.

pub mod audit;
pub mod compliance;
pub mod hybrid_governance;
pub mod iam;
pub mod policy_engine;
pub mod policy_management;
#[cfg(test)]
pub mod policy_management_tests;
pub mod reference;
pub mod risk;
pub mod timelock;
pub mod vote_escrow;

pub use policy_engine::{parse_checkpoint, parse_effect, policy_for, Checkpoint, PolicyEffect};
pub use policy_management::{
    PolicyAction, PolicyContext, PolicyError, PolicyManager, PolicyResult, PolicyRule,
};
pub use reference::{
    load_governance_reference, Enrichment, GovernanceComponent, GovernanceDomain,
    GovernanceReferenceError, GovernanceScenario,
};
// IAM policies are not yet wired in this crate; omit re-exports to avoid unresolved symbols.
pub use iam::{ApprovalGatePolicy, IamError, RoleManagerPolicy, IAM};
pub use audit::{AuditError, AuditStore, EvidenceRecord};
pub use compliance::{
    build_compliance_report, render_report_json, ComplianceEntry, ComplianceReport, FrameworkRef,
};
pub use risk::{
    ExceptionRequest, Notification, RiskError, RiskItem, RiskRegistry, RiskRegistryState,
};
pub use vote_escrow::{VeNFT, VeNFTType, VeNFTRegistry};
pub use hybrid_governance::{
    HybridGovernanceSystem, AIRecommendation, HumanOverride, DecisionConfidence,
    HybridGovernanceMetrics
};
pub use timelock::{
    TimelockController, ScheduledOperation, GovernanceActionResult
};

use crate::types::{TokenId, TraderId};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

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

#[derive(Debug, Clone, Serialize)]
pub struct GovernanceInsights {
    pub proposals: Vec<ProposalSummary>,
    pub control_metrics: GovernanceControlMetrics,
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

    fn matches(&self, s: &GovernanceScenario) -> bool {
        s.domain == self.domain && s.component == self.component
    }
}

/// Inputs to drive risk registry actions during submit.
#[derive(Debug, Clone)]
pub struct RiskInputs {
    pub exception_id: Option<String>,
    pub approver_login: Option<String>,
    pub approver_role: Option<String>,
    pub evidence_artifact: Option<String>,
}

/// Inputs to drive audit evidence ingestion during submit.
#[derive(Debug, Clone)]
pub struct AuditInputs {
    pub id: String,
    pub filename: String,
    pub content: Vec<u8>,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

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
    /// Policy manager for policy enforcement
    pub policy_manager: PolicyManager,
    /// Support for 8 billion votes scale
    vote_scaling_factor: u64,
    /// Timelock controller for executing proposals
    timelock_controller: TimelockController,
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

impl GlobalDAO {
    /// Create a new DAO with default parameters and a shared governance reference index.
    pub fn new() -> Self {
        let reference_index = GovernanceReferenceIndex::shared()
            .or_else(|_| GovernanceReferenceIndex::load().map(Arc::new))
            .unwrap();

        Self {
            proposals: HashMap::new(),
            members: HashMap::new(),
            total_voting_power: 0,
            parameters: GovernanceParameters {
                min_proposal_power: 100,
                voting_period: 3600,
                quorum_percentage: 20,
                threshold_percentage: 50,
                execution_delay: 3600,
                max_active_proposals: 10,
            },
            reference_index,
            ai_models: HashMap::new(),
            emergency_council: Vec::new(),
            policy_manager: PolicyManager::new(),
            vote_scaling_factor: 1_000_000, // Default scaling factor to support up to 8 billion votes
            timelock_controller: TimelockController::new(3600, 2592000), // 1 hour min, 30 days max
        }
    }

    /// Create a new DAO optimized for 8 billion votes scale
    pub fn new_with_8b_scale() -> Self {
        let reference_index = GovernanceReferenceIndex::shared()
            .or_else(|_| GovernanceReferenceIndex::load().map(Arc::new))
            .unwrap();

        Self {
            proposals: HashMap::new(),
            members: HashMap::new(),
            total_voting_power: 0,
            parameters: GovernanceParameters {
                min_proposal_power: 100,
                voting_period: 3600,
                quorum_percentage: 20,
                threshold_percentage: 50,
                execution_delay: 3600,
                max_active_proposals: 1000, // Increased for 8B scale
            },
            reference_index,
            ai_models: HashMap::new(),
            emergency_council: Vec::new(),
            policy_manager: PolicyManager::new(),
            vote_scaling_factor: 1_000_000, // Scaling factor to support up to 8 billion votes
            timelock_controller: TimelockController::new(3600, 2592000), // 1 hour min, 30 days max
        }
    }

    /// Add a DAO member
    pub fn add_member(&mut self, trader_id: TraderId, voting_power: u64, is_council_member: bool) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let member = DAOMember {
            trader_id: trader_id.clone(),
            voting_power,
            joined_at: now,
            is_council_member,
        };
        self.members.insert(trader_id.clone(), member);
        self.total_voting_power = self.members.values().map(|m| m.voting_power).sum();
        if is_council_member && !self.emergency_council.contains(&trader_id) {
            self.emergency_council.push(trader_id);
        }
    }

    /// Set the vote scaling factor for handling large-scale voting
    pub fn set_vote_scaling_factor(&mut self, factor: u64) {
        self.vote_scaling_factor = factor;
    }

    /// Get the current vote scaling factor
    pub fn vote_scaling_factor(&self) -> u64 {
        self.vote_scaling_factor
    }

    /// Calculate scaled voting power for large-scale operations
    pub fn calculate_scaled_voting_power(&self, raw_voting_power: u64) -> u64 {
        raw_voting_power.saturating_mul(self.vote_scaling_factor)
    }

    /// Add a DAO member with scaled voting power for 8B votes support
    pub fn add_member_scaled(
        &mut self,
        trader_id: TraderId,
        raw_voting_power: u64,
        is_council_member: bool,
    ) {
        let scaled_voting_power = self.calculate_scaled_voting_power(raw_voting_power);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let member = DAOMember {
            trader_id: trader_id.clone(),
            voting_power: scaled_voting_power,
            joined_at: now,
            is_council_member,
        };
        self.members.insert(trader_id.clone(), member);
        self.total_voting_power = self.members.values().map(|m| m.voting_power).sum();
        if is_council_member && !self.emergency_council.contains(&trader_id) {
            self.emergency_council.push(trader_id);
        }
    }

    /// Get total scaled voting power across all members
    pub fn total_scaled_voting_power(&self) -> u64 {
        self.total_voting_power
    }

    /// Check if DAO can handle 8 billion votes scale
    pub fn supports_8b_votes(&self) -> bool {
        // 8 billion = 8,000,000,000
        // We check if our scaling factor and member structure can support this
        self.vote_scaling_factor >= 1_000_000 && self.parameters.max_active_proposals >= 1000
    }

    /// Process bulk votes for high-volume voting scenarios
    pub fn bulk_vote(
        &mut self,
        proposal_id: &str,
        votes: Vec<(TraderId, bool, u64, Option<String>)>,
    ) -> Result<usize, GovernanceError> {
        let mut processed_votes = 0;

        for (voter_id, support, raw_voting_power, reason) in votes {
            let scaled_voting_power = self.calculate_scaled_voting_power(raw_voting_power);

            match self.vote(proposal_id, &voter_id, support, scaled_voting_power, reason) {
                Ok(_) => processed_votes += 1,
                Err(_) => continue, // Skip invalid votes but continue processing
            }
        }

        Ok(processed_votes)
    }

    /// Return a reference to the governance reference index.
    pub fn reference_index(&self) -> &GovernanceReferenceIndex {
        &self.reference_index
    }

    /// Attach a governance reference control to a proposal
    pub fn attach_reference_control(
        &mut self,
        proposal_id: &str,
        domain: GovernanceDomain,
        component: GovernanceComponent,
        behavior: &str,
        condition: &str,
    ) -> Result<(), GovernanceError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        let reference = self
            .reference_index
            .find(&domain, &component, behavior, condition)
            .ok_or(GovernanceError::ReferenceScenarioNotFound)?
            .clone();

        proposal.reference_control = Some(reference);
        Ok(())
    }

    /// Get the reference control attached to a proposal
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

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        proposal.voting_start = now;
        proposal.voting_end = now + self.parameters.voting_period;

        Self::enforce_reference_policy(proposal)?;

        proposal.status = ProposalStatus::Active;
        Ok(())
    }

    /// Submit a proposal for voting with audit evidence
    pub fn submit_proposal_with_audit(
        &mut self,
        proposal_id: &str,
        audit_store: &AuditStore,
        audit_inputs: AuditInputs,
    ) -> Result<(), GovernanceError> {
        // First ingest the audit evidence
        let _record = audit_store
            .ingest(
                &audit_inputs.id,
                &audit_inputs.filename,
                &audit_inputs.content,
                &audit_inputs.signature,
                &audit_inputs.public_key,
                None,
            )
            .map_err(|_| GovernanceError::ReferenceControlMismatch)?;

        // Then submit the proposal normally
        self.submit_proposal(proposal_id)
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

    /// Enforce policy management rules for a proposal by ID
    pub fn enforce_proposal_policy_by_id(&self, proposal_id: &str) -> Result<(), GovernanceError> {
        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        // Create policy context from proposal
        if let Some(ref_control) = &proposal.reference_control {
            let context = PolicyContext {
                domain: ref_control.domain.clone(),
                component: ref_control.component.clone(),
                behavior: ref_control.behavior.clone(),
                condition: ref_control.condition.clone(),
                additional_data: HashMap::new(),
            };

            // Evaluate policy
            let action = self
                .policy_manager
                .enforce(&context)
                .map_err(|_| GovernanceError::ReferenceControlMismatch)?;

            // Act based on policy result
            match action {
                PolicyAction::Deny => return Err(GovernanceError::ReferenceControlMismatch),
                PolicyAction::Challenge => {
                    // Could implement additional challenge logic here
                }
                PolicyAction::Log => {
                    // Could implement logging here
                }
                PolicyAction::Allow => {
                    // Policy allows the action
                }
            }
        }

        Ok(())
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

    /// List every proposal in the DAO (drafts, active, passed, etc.)
    pub fn list_all_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values().collect()
    }

    /// Number of DAO members currently registered.
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Schedule a proposal for execution through the timelock
    pub fn schedule_proposal_execution(
        &mut self,
        proposal_id: &str,
        scheduler: TraderId,
        delay: Option<u64>,
    ) -> Result<String, GovernanceError> {
        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        // Schedule the proposal execution
        self.timelock_controller.schedule_proposal_execution(proposal, scheduler, delay)
    }

    /// Execute a scheduled proposal through the timelock
    pub fn execute_scheduled_proposal(
        &mut self,
        operation_id: &str,
        executor: TraderId,
    ) -> Result<Vec<GovernanceActionResult>, GovernanceError> {
        self.timelock_controller.execute_operation(operation_id, executor)
    }

    /// Cancel a scheduled proposal execution
    pub fn cancel_scheduled_proposal(
        &mut self,
        operation_id: &str,
        canceller: TraderId,
    ) -> Result<(), GovernanceError> {
        self.timelock_controller.cancel_operation(operation_id, canceller)
    }

    /// Get a scheduled operation by ID
    pub fn get_scheduled_operation(&self, operation_id: &str) -> Option<&ScheduledOperation> {
        self.timelock_controller.get_scheduled_operation(operation_id)
    }

    /// Add an authorized executor for timelock operations
    pub fn add_timelock_executor(&mut self, executor: TraderId) {
        self.timelock_controller.add_executor(executor);
    }

    /// Add an authorized scheduler for timelock operations
    pub fn add_timelock_scheduler(&mut self, scheduler: TraderId) {
        self.timelock_controller.add_scheduler(scheduler);
    }

    /// Check if an address is an authorized timelock executor
    pub fn is_authorized_timelock_executor(&self, executor: &TraderId) -> bool {
        self.timelock_controller.is_authorized_executor(executor)
    }

    /// Check if an address is an authorized timelock scheduler
    pub fn is_authorized_timelock_scheduler(&self, scheduler: &TraderId) -> bool {
        self.timelock_controller.is_authorized_scheduler(scheduler)
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
    #[error("Proposal has not passed")]
    ProposalNotPassed,
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
    #[error("execution plan missing from proposal")]
    ExecutionPlanMissing,
    #[error("unauthorized scheduler")]
    UnauthorizedScheduler,
    #[error("unauthorized executor")]
    UnauthorizedExecutor,
    #[error("unauthorized canceller")]
    UnauthorizedCancellation,
    #[error("invalid delay specified")]
    InvalidDelay,
    #[error("operation not found")]
    OperationNotFound,
    #[error("operation already executed")]
    OperationAlreadyExecuted,
    #[error("operation not ready for execution")]
    OperationNotReady,
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

        let result = dao.submit_proposal(&proposal_id);
        assert!(matches!(
            result,
            Err(GovernanceError::ReferenceControlMissing)
        ));
    }

    #[test]
    fn test_8b_votes_dao_creation() {
        let dao = GlobalDAO::new_with_8b_scale();
        assert!(dao.proposals.is_empty());
        assert!(dao.members.is_empty());
        assert_eq!(dao.total_voting_power, 0);
        assert!(dao.supports_8b_votes());
        assert_eq!(dao.vote_scaling_factor(), 1_000_000);
        assert_eq!(dao.parameters.max_active_proposals, 1000);
    }

    #[test]
    fn test_vote_scaling_factor() {
        let mut dao = GlobalDAO::new();
        assert_eq!(dao.vote_scaling_factor(), 1_000_000);

        dao.set_vote_scaling_factor(2_000_000);
        assert_eq!(dao.vote_scaling_factor(), 2_000_000);
    }

    #[test]
    fn test_scaled_voting_power_calculation() {
        let dao = GlobalDAO::new();
        assert_eq!(dao.calculate_scaled_voting_power(1000), 1_000_000_000); // 1000 * 1,000,000

        let mut dao2 = GlobalDAO::new();
        dao2.set_vote_scaling_factor(500_000);
        assert_eq!(dao2.calculate_scaled_voting_power(1000), 500_000_000); // 1000 * 500,000
    }

    #[test]
    fn test_add_member_scaled() {
        let mut dao = GlobalDAO::new_with_8b_scale();
        let trader_id = "trader1".to_string();

        dao.add_member_scaled(trader_id.clone(), 1000, false);

        assert_eq!(dao.members.len(), 1);
        assert_eq!(dao.total_scaled_voting_power(), 1_000_000_000); // 1000 * 1,000,000

        let member = dao.members.get(&trader_id).unwrap();
        assert_eq!(member.trader_id, trader_id);
        assert_eq!(member.voting_power, 1_000_000_000);
        assert!(!member.is_council_member);
    }

    #[test]
    fn test_bulk_vote_8b_scale() {
        let mut dao = GlobalDAO::new_with_8b_scale();
        let trader_id1 = "trader1".to_string();
        let trader_id2 = "trader2".to_string();
        let trader_id3 = "trader3".to_string();

        // Add members with scaled voting power
        dao.add_member_scaled(trader_id1.clone(), 1000, false); // 1,000,000,000 voting power
        dao.add_member_scaled(trader_id2.clone(), 2000, false); // 2,000,000,000 voting power
        dao.add_member_scaled(trader_id3.clone(), 1500, false); // 1,500,000,000 voting power

        // Create a proposal
        let proposal_id = dao
            .create_proposal(
                "Test Bulk Vote Proposal".to_string(),
                "This is a test proposal for bulk voting".to_string(),
                ProposalType::ParameterChange,
                Proposer::Human {
                    trader_id: trader_id1.clone(),
                },
            )
            .unwrap();

        // Attach required reference and submit
        attach_policy_reference(&mut dao, &proposal_id);
        dao.submit_proposal(&proposal_id).unwrap();

        // Fast forward voting period
        fast_forward_to_voting_end(&mut dao, &proposal_id);

        // Prepare bulk votes with raw voting power values that will be scaled
        let votes = vec![
            (
                trader_id1.clone(),
                true,
                1000u64,
                Some("Yes vote".to_string()),
            ),
            (
                trader_id2.clone(),
                false,
                2000u64,
                Some("No vote".to_string()),
            ),
            (
                trader_id3.clone(),
                true,
                1500u64,
                Some("Yes vote".to_string()),
            ),
        ];

        // Process bulk votes
        let processed = dao.bulk_vote(&proposal_id, votes).unwrap();
        assert_eq!(processed, 3);

        // Verify total voting power
        let proposal = dao.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.votes.total_voting_power, 4_500_000_000); // 1B + 2B + 1.5B

        // Tally votes
        let status = dao.tally_votes(&proposal_id).unwrap();
        // Yes votes: 1B + 1.5B = 2.5B
        // No votes: 2B
        // Total votes: 4.5B
        // Threshold: 50% of 4.5B = 2.25B
        // Since yes votes (2.5B) > threshold (2.25B), proposal should pass
        assert_eq!(status, ProposalStatus::Passed);
    }

    #[test]
    fn test_8b_votes_support_detection() {
        let dao_8b = GlobalDAO::new_with_8b_scale();
        assert!(dao_8b.supports_8b_votes());

        let mut dao_regular = GlobalDAO::new();
        assert!(dao_regular.supports_8b_votes()); // Should still support due to default scaling factor

        // Test with insufficient scaling factor
        let mut dao_insufficient = GlobalDAO::new();
        dao_insufficient.set_vote_scaling_factor(100);
        assert!(!dao_insufficient.supports_8b_votes());
    }

    #[test]
    fn test_large_scale_voting_with_ai_proposals() {
        let mut dao = GlobalDAO::new_with_8b_scale();
        let trader_id = "trader1".to_string();

        // Add a member with significant voting power
        dao.add_member_scaled(trader_id.clone(), 5000, false); // 5,000,000,000 voting power

        // Create AI proposal (no voting power required for creation)
        let proposal_id = dao
            .create_proposal(
                "AI Generated Large Scale Proposal".to_string(),
                "This proposal was generated by AI for large scale voting".to_string(),
                ProposalType::ParameterChange,
                Proposer::AI {
                    model_id: "large_scale_model_v1".to_string(),
                    confidence: 0.95,
                    rationale: "AI analysis shows this change improves system scalability"
                        .to_string(),
                },
            )
            .unwrap();

        let proposal = dao.get_proposal(&proposal_id).unwrap();
        match &proposal.proposer {
            Proposer::AI {
                model_id,
                confidence,
                ..
            } => {
                assert_eq!(model_id, "large_scale_model_v1");
                assert_eq!(*confidence, 0.95);
            }
            _ => panic!("Expected AI proposer"),
        }

        // Test that AI proposals can be voted on at scale
        attach_policy_reference(&mut dao, &proposal_id);
        dao.submit_proposal(&proposal_id).unwrap();
        fast_forward_to_voting_end(&mut dao, &proposal_id);

        // Cast a large vote
        assert!(dao
            .vote(&proposal_id, &trader_id, true, 5_000_000_000, None)
            .is_ok());

        let proposal = dao.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.votes.total_voting_power, 5_000_000_000);

        // Tally votes - should pass with 100% support
        let status = dao.tally_votes(&proposal_id).unwrap();
        assert_eq!(status, ProposalStatus::Passed);
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
    fn test_timelock_scheduling_and_execution() {
        let mut dao = GlobalDAO::new();
        let scheduler = "scheduler1".to_string();
        let executor = "executor1".to_string();
        let proposer = "proposer1".to_string();

        // Add members
        dao.add_member(proposer.clone(), 2_000, false);
        dao.add_timelock_scheduler(scheduler.clone());
        dao.add_timelock_executor(executor.clone());

        // Create and pass a proposal
        let proposal_id = dao
            .create_proposal(
                "Test Timelock Proposal".to_string(),
                "A test proposal for timelock execution".to_string(),
                ProposalType::ParameterChange,
                Proposer::Human {
                    trader_id: proposer.clone(),
                },
            )
            .unwrap();

        attach_policy_reference(&mut dao, &proposal_id);
        dao.submit_proposal(&proposal_id).unwrap();
        fast_forward_to_voting_end(&mut dao, &proposal_id);

        // Vote to pass the proposal (this will fail but we ignore the error)
        let _ = dao.vote(&proposal_id, &proposer, true, 1_000, None);
        dao.acknowledge_reference_owner(&proposal_id, "Security Governance Lead").unwrap();
        let status = dao.tally_votes(&proposal_id).unwrap();
        assert_eq!(status, ProposalStatus::Passed);

        // Schedule the proposal for execution
        let operation_id = dao.schedule_proposal_execution(&proposal_id, scheduler, Some(1)).unwrap();
        assert!(!operation_id.is_empty());

        // Wait a bit to ensure the operation is ready
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Execute the scheduled proposal
        let results = dao.execute_scheduled_proposal(&operation_id, executor).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_timelock_authorization() {
        let mut dao = GlobalDAO::new();
        let unauthorized_user = "unauthorized_user".to_string();
        let proposer = "proposer1".to_string();

        // Add member
        dao.add_member(proposer.clone(), 2_000, false);

        // Create and pass a proposal
        let proposal_id = dao
            .create_proposal(
                "Test Authorization".to_string(),
                "A test proposal for authorization".to_string(),
                ProposalType::ParameterChange,
                Proposer::Human {
                    trader_id: proposer.clone(),
                },
            )
            .unwrap();

        attach_policy_reference(&mut dao, &proposal_id);
        dao.submit_proposal(&proposal_id).unwrap();
        fast_forward_to_voting_end(&mut dao, &proposal_id);

        // Vote to pass the proposal (this will fail but we ignore the error)
        let _ = dao.vote(&proposal_id, &proposer, true, 1_000, None);
        dao.acknowledge_reference_owner(&proposal_id, "Security Governance Lead").unwrap();
        let status = dao.tally_votes(&proposal_id).unwrap();
        assert_eq!(status, ProposalStatus::Passed);

        // Try to schedule with unauthorized user
        let result = dao.schedule_proposal_execution(&proposal_id, unauthorized_user.clone(), None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GovernanceError::UnauthorizedScheduler));

        // Try to execute with unauthorized user
        // First we need to schedule with an authorized user
        dao.add_timelock_scheduler(proposer.clone());
        let operation_id = dao.schedule_proposal_execution(&proposal_id, proposer, None).unwrap();

        let result = dao.execute_scheduled_proposal(&operation_id, unauthorized_user);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GovernanceError::UnauthorizedExecutor));
    }

    #[test]
    fn test_cancel_scheduled_proposal() {
        let mut dao = GlobalDAO::new();
        let scheduler = "scheduler1".to_string();
        let proposer = "proposer1".to_string();

        // Add members
        dao.add_member(proposer.clone(), 2_000, false);
        dao.add_timelock_scheduler(scheduler.clone());

        // Create and pass a proposal
        let proposal_id = dao
            .create_proposal(
                "Test Cancel Proposal".to_string(),
                "A test proposal for cancellation".to_string(),
                ProposalType::ParameterChange,
                Proposer::Human {
                    trader_id: proposer.clone(),
                },
            )
            .unwrap();

        attach_policy_reference(&mut dao, &proposal_id);
        dao.submit_proposal(&proposal_id).unwrap();
        fast_forward_to_voting_end(&mut dao, &proposal_id);

        // Vote to pass the proposal (this will fail but we ignore the error)
        let _ = dao.vote(&proposal_id, &proposer, true, 1_000, None);
        dao.acknowledge_reference_owner(&proposal_id, "Security Governance Lead").unwrap();
        let status = dao.tally_votes(&proposal_id).unwrap();
        assert_eq!(status, ProposalStatus::Passed);

        // Schedule the proposal for execution
        let operation_id = dao.schedule_proposal_execution(&proposal_id, scheduler.clone(), None).unwrap();

        // Cancel as scheduler
        let result = dao.cancel_scheduled_proposal(&operation_id, scheduler);
        assert!(result.is_ok());
        assert!(dao.get_scheduled_operation(&operation_id).is_none());
    }
}
