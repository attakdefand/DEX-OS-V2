//! AI Governance implementation for the DEX-OS core engine
//!
//! This module implements the Priority 3 features from DEX-OS-V2.csv:
//! - AI Governance (AI Proposals, Global DAO)
//!
//! It provides functionality for AI-driven governance proposals and
//! global decentralized autonomous organization (DAO) operations.

use crate::types::{TokenId, TraderId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    Other(String),
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

impl GlobalDAO {
    /// Create a new Global DAO with default parameters
    pub fn new() -> Self {
        Self {
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
            ai_models: HashMap::new(),
            emergency_council: Vec::new(),
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_dao_creation() {
        let dao = GlobalDAO::new();
        assert!(dao.proposals.is_empty());
        assert!(dao.members.is_empty());
        assert_eq!(dao.total_voting_power, 0);
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
}
