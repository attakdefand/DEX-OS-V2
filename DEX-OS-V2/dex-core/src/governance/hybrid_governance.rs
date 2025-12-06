//! Hybrid Governance Module combining AI decision making with human oversight
//!
//! This module implements the Priority 2 feature from DEX-OS-V2.csv:
//! - Main Types,Governance Model,Governance,AI + Global DAO Hybrid,Hybrid Governance,High

use crate::governance::{
    GlobalDAO, Proposal, ProposalType, Proposer, ProposalStatus, Votes, Vote,
    GovernanceAction, AIAnalysis, ImpactAnalysis, HistoricalProposal,
    GovernanceError, DAOMember
};
use crate::prediction_engine::{MarketContext, PredictionEngine, PredictionResult};
use crate::types::TraderId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Hybrid governance decision confidence levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DecisionConfidence {
    High,    // > 0.8
    Medium,  // > 0.6
    Low,     // > 0.4
    Uncertain, // <= 0.4
}

/// AI recommendation for governance proposals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRecommendation {
    pub proposal_id: String,
    pub recommendation: ProposalStatus,
    pub confidence: f64,
    pub confidence_level: DecisionConfidence,
    pub rationale: String,
    pub risk_score: f64,
    pub impact_analysis: ImpactAnalysis,
    pub similar_historical_proposals: Vec<HistoricalProposal>,
}

/// Human override decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanOverride {
    pub id: String,
    pub proposal_id: String,
    pub voter_id: TraderId,
    pub decision: ProposalStatus,
    pub reason: String,
    pub timestamp: u64,
}

/// Hybrid governance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridGovernanceMetrics {
    pub total_proposals: u64,
    pub ai_recommendations: u64,
    pub human_overrides: u64,
    pub ai_human_alignment: f64, // Percentage of times AI and humans agreed
    pub average_confidence: f64,
}

/// Hybrid Governance System
pub struct HybridGovernanceSystem {
    /// Global DAO for core governance operations
    dao: GlobalDAO,
    /// AI prediction engine for decision making
    prediction_engine: PredictionEngine,
    /// AI recommendations for proposals
    ai_recommendations: HashMap<String, AIRecommendation>,
    /// Human override decisions
    human_overrides: HashMap<String, HumanOverride>,
    /// Governance metrics
    metrics: HybridGovernanceMetrics,
}

impl HybridGovernanceSystem {
    /// Create a new hybrid governance system
    pub fn new(dao: GlobalDAO, prediction_engine: PredictionEngine) -> Self {
        Self {
            dao,
            prediction_engine,
            ai_recommendations: HashMap::new(),
            human_overrides: HashMap::new(),
            metrics: HybridGovernanceMetrics {
                total_proposals: 0,
                ai_recommendations: 0,
                human_overrides: 0,
                ai_human_alignment: 0.0,
                average_confidence: 0.0,
            },
        }
    }

    /// Create a new governance proposal with AI analysis
    pub fn create_proposal(
        &mut self,
        title: String,
        description: String,
        proposal_type: ProposalType,
        proposer: Proposer,
    ) -> Result<String, GovernanceError> {
        let proposal_id = self.dao.create_proposal(
            title.clone(),
            description.clone(),
            proposal_type.clone(),
            proposer.clone(),
        )?;

        // Generate AI analysis for the proposal
        self.generate_ai_analysis(&proposal_id, &title, &description, &proposal_type)?;

        self.metrics.total_proposals += 1;
        Ok(proposal_id)
    }

    /// Generate AI analysis for a proposal
    fn generate_ai_analysis(
        &mut self,
        proposal_id: &str,
        title: &str,
        description: &str,
        proposal_type: &ProposalType,
    ) -> Result<(), GovernanceError> {
        // Create market context from proposal data
        let proposal_text = format!("{} {}", title, description);
        let historical_votes: Vec<f64> = vec![0.5]; // Placeholder for now
        
        let volatility = 0.1;
        let momentum = 0.0;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let context = MarketContext {
            base_token: "GOVERNANCE".to_string(),
            quote_token: "PROPOSAL_SUCCESS".to_string(),
            historical_prices: historical_votes,
            volatility,
            momentum,
            timestamp,
        };
        
        // Get AI prediction
        let prediction_bundle = self.prediction_engine.predict(&context);
        let prediction = &prediction_bundle.consensus;
        
        // Determine recommendation based on prediction
        let (recommendation, confidence_level) = if prediction.price > 0.8 {
            (ProposalStatus::Passed, DecisionConfidence::High)
        } else if prediction.price > 0.6 {
            (ProposalStatus::Passed, DecisionConfidence::Medium)
        } else if prediction.price > 0.4 {
            (ProposalStatus::Rejected, DecisionConfidence::Low)
        } else {
            (ProposalStatus::Rejected, DecisionConfidence::Uncertain)
        };
        
        // Create impact analysis based on proposal type
        let impact_analysis = self.estimate_impact(proposal_type);
        
        // Find similar historical proposals (placeholder)
        let similar_historical_proposals = vec![];
        
        let ai_analysis = AIAnalysis {
            predicted_outcome: prediction.price as f32,
            risk_score: (1.0 - prediction.price) as f32,
            impact_analysis,
            historical_comparison: similar_historical_proposals,
            confidence: prediction.confidence as f32,
        };
        
        // Create AI recommendation
        let rationale = format!(
            "AI analysis predicts {:.1}% chance of success with {:.1}% confidence",
            prediction.price * 100.0,
            prediction.confidence * 100.0
        );
        
        let recommendation = AIRecommendation {
            proposal_id: proposal_id.to_string(),
            recommendation,
            confidence: prediction.price,
            confidence_level,
            rationale,
            risk_score: 1.0 - prediction.price,
            impact_analysis: ai_analysis.impact_analysis.clone(),
            similar_historical_proposals: ai_analysis.historical_comparison.clone(),
        };
        
        self.ai_recommendations.insert(proposal_id.to_string(), recommendation);
        self.metrics.ai_recommendations += 1;
        
        // Update average confidence
        self.update_average_confidence();
        
        Ok(())
    }

    /// Estimate impact of a proposal based on its type
    fn estimate_impact(&self, proposal_type: &ProposalType) -> ImpactAnalysis {
        match proposal_type {
            ProposalType::ParameterChange => ImpactAnalysis {
                liquidity_impact: 0.3,
                volume_impact: 0.2,
                adoption_impact: 0.4,
                security_impact: 0.6,
            },
            ProposalType::TreasuryAllocation => ImpactAnalysis {
                liquidity_impact: 0.7,
                volume_impact: 0.5,
                adoption_impact: 0.3,
                security_impact: 0.8,
            },
            ProposalType::ProtocolUpgrade => ImpactAnalysis {
                liquidity_impact: 0.8,
                volume_impact: 0.7,
                adoption_impact: 0.9,
                security_impact: 0.9,
            },
            ProposalType::NewMarketListing => ImpactAnalysis {
                liquidity_impact: 0.9,
                volume_impact: 0.8,
                adoption_impact: 0.7,
                security_impact: 0.5,
            },
            ProposalType::FeeStructureChange => ImpactAnalysis {
                liquidity_impact: 0.6,
                volume_impact: 0.5,
                adoption_impact: 0.4,
                security_impact: 0.3,
            },
            ProposalType::EmergencyPause => ImpactAnalysis {
                liquidity_impact: 0.2,
                volume_impact: 0.1,
                adoption_impact: 0.1,
                security_impact: 0.9,
            },
            ProposalType::TreasuryAutomation => ImpactAnalysis {
                liquidity_impact: 0.4,
                volume_impact: 0.3,
                adoption_impact: 0.5,
                security_impact: 0.7,
            },
            ProposalType::ObservabilityUpgrade => ImpactAnalysis {
                liquidity_impact: 0.2,
                volume_impact: 0.2,
                adoption_impact: 0.3,
                security_impact: 0.8,
            },
            ProposalType::AccessControlUpdate => ImpactAnalysis {
                liquidity_impact: 0.3,
                volume_impact: 0.2,
                adoption_impact: 0.4,
                security_impact: 0.9,
            },
            ProposalType::ChangeManagementOverride => ImpactAnalysis {
                liquidity_impact: 0.1,
                volume_impact: 0.1,
                adoption_impact: 0.2,
                security_impact: 0.9,
            },
            ProposalType::EducationProgramRefresh => ImpactAnalysis {
                liquidity_impact: 0.1,
                volume_impact: 0.1,
                adoption_impact: 0.6,
                security_impact: 0.4,
            },
            ProposalType::Other(_) => ImpactAnalysis {
                liquidity_impact: 0.3,
                volume_impact: 0.3,
                adoption_impact: 0.3,
                security_impact: 0.3,
            },
        }
    }

    /// Submit a proposal for voting (includes AI recommendation)
    pub fn submit_proposal(&mut self, proposal_id: &str) -> Result<(), GovernanceError> {
        self.dao.submit_proposal(proposal_id)
    }

    /// Cast a vote on a proposal
    pub fn vote(
        &mut self,
        proposal_id: &str,
        voter_id: &TraderId,
        support: bool,
        voting_power: u64,
        reason: Option<String>,
    ) -> Result<(), GovernanceError> {
        self.dao.vote(proposal_id, voter_id, support, voting_power, reason)
    }

    /// Cast a human override decision
    pub fn human_override(
        &mut self,
        proposal_id: &str,
        voter_id: TraderId,
        decision: ProposalStatus,
        reason: String,
    ) -> Result<(), GovernanceError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let override_id = format!("override_{}_{}", proposal_id, timestamp);
        
        let human_override = HumanOverride {
            id: override_id.clone(),
            proposal_id: proposal_id.to_string(),
            voter_id: voter_id.clone(),
            decision: decision.clone(),
            reason,
            timestamp,
        };
        
        self.human_overrides.insert(override_id, human_override);
        self.metrics.human_overrides += 1;
        
        // Check if this overrides AI recommendation
        if let Some(ai_rec) = self.ai_recommendations.get(proposal_id) {
            if ai_rec.recommendation == decision {
                // AI and human agree
                self.metrics.ai_human_alignment = (self.metrics.ai_human_alignment * (self.metrics.human_overrides as f64 - 1.0) 
                    + 1.0) / self.metrics.human_overrides as f64;
            } else {
                // AI and human disagree
                self.metrics.ai_human_alignment = (self.metrics.ai_human_alignment * (self.metrics.human_overrides as f64 - 1.0)) 
                    / self.metrics.human_overrides as f64;
            }
        }
        
        Ok(())
    }

    /// Tally votes for a proposal and determine outcome with hybrid decision making
    pub fn tally_votes(&mut self, proposal_id: &str) -> Result<ProposalStatus, GovernanceError> {
        // First, let the DAO do its normal tally
        let initial_result = self.dao.tally_votes(proposal_id)?;
        
        // Check if there's a human override
        let final_result = if let Some(human_override) = self.get_latest_human_override(proposal_id) {
            // Apply human override
            human_override.decision.clone()
        } else {
            // Use AI recommendation if no human override
            if let Some(ai_rec) = self.ai_recommendations.get(proposal_id) {
                match ai_rec.confidence_level {
                    DecisionConfidence::High | DecisionConfidence::Medium => {
                        // For high/medium confidence, follow AI recommendation
                        ai_rec.recommendation.clone()
                    }
                    DecisionConfidence::Low | DecisionConfidence::Uncertain => {
                        // For low confidence, stick with DAO result
                        initial_result
                    }
                }
            } else {
                // No AI recommendation, use DAO result
                initial_result
            }
        };
        
        // Update the proposal status in the DAO
        if let Some(proposal) = self.dao.proposals.get_mut(proposal_id) {
            proposal.status = final_result.clone();
        }
        
        Ok(final_result)
    }

    /// Get the latest human override for a proposal
    fn get_latest_human_override(&self, proposal_id: &str) -> Option<&HumanOverride> {
        self.human_overrides
            .values()
            .filter(|override_decision| override_decision.proposal_id == proposal_id)
            .max_by_key(|override_decision| override_decision.timestamp)
    }

    /// Get AI recommendation for a proposal
    pub fn get_ai_recommendation(&self, proposal_id: &str) -> Option<&AIRecommendation> {
        self.ai_recommendations.get(proposal_id)
    }

    /// Get human override for a proposal
    pub fn get_human_override(&self, override_id: &str) -> Option<&HumanOverride> {
        self.human_overrides.get(override_id)
    }

    /// Get governance metrics
    pub fn get_metrics(&self) -> &HybridGovernanceMetrics {
        &self.metrics
    }

    /// Get all active proposals with AI recommendations
    pub fn get_active_proposals_with_recommendations(&self) -> Vec<(&Proposal, Option<&AIRecommendation>)> {
        self.dao
            .get_active_proposals()
            .into_iter()
            .map(|proposal| {
                let recommendation = self.ai_recommendations.get(&proposal.id);
                (proposal, recommendation)
            })
            .collect()
    }

    /// Update average confidence metric
    fn update_average_confidence(&mut self) {
        if self.metrics.ai_recommendations > 0 {
            let total_confidence: f64 = self.ai_recommendations
                .values()
                .map(|rec| rec.confidence)
                .sum();
            self.metrics.average_confidence = total_confidence / self.metrics.ai_recommendations as f64;
        }
    }

    /// Add a DAO member
    pub fn add_member(&mut self, trader_id: TraderId, voting_power: u64, is_council_member: bool) {
        self.dao.add_member(trader_id, voting_power, is_council_member);
    }

    /// Get a reference to the underlying DAO
    pub fn dao(&self) -> &GlobalDAO {
        &self.dao
    }

    /// Get a mutable reference to the underlying DAO
    pub fn dao_mut(&mut self) -> &mut GlobalDAO {
        &mut self.dao
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::{ProposalType, Proposer};
    use crate::prediction_engine::{AggregationStrategy, ReinforcementLearningPredictor, TransformerPredictor};

    fn create_test_prediction_engine() -> PredictionEngine {
        let models: Vec<Box<dyn crate::prediction_engine::Predictor>> = vec![
            Box::new(TransformerPredictor::new("transformer", 1.0, 42)),
            Box::new(ReinforcementLearningPredictor::new("rl", 24)),
        ];
        PredictionEngine::new(models, AggregationStrategy::ConfidenceWeighted)
    }

    fn create_test_dao() -> GlobalDAO {
        let mut dao = GlobalDAO::new();
        // Add a test member
        dao.add_member("voter1".to_string(), 1000, false);
        dao
    }

    #[test]
    fn test_hybrid_governance_system_creation() {
        let dao = create_test_dao();
        let engine = create_test_prediction_engine();
        let system = HybridGovernanceSystem::new(dao, engine);
        assert_eq!(system.get_metrics().total_proposals, 0);
        assert_eq!(system.get_metrics().ai_recommendations, 0);
    }

    #[test]
    fn test_proposal_creation_with_ai_analysis() {
        let dao = create_test_dao();
        let engine = create_test_prediction_engine();
        let mut system = HybridGovernanceSystem::new(dao, engine);
        
        let proposal_id = system.create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal for AI analysis".to_string(),
            ProposalType::ParameterChange,
            Proposer::Human {
                trader_id: "voter1".to_string(),
            },
        ).unwrap();
        
        assert!(!proposal_id.is_empty());
        assert_eq!(system.get_metrics().total_proposals, 1);
        assert_eq!(system.get_metrics().ai_recommendations, 1);
        
        // Check that AI recommendation was generated
        let recommendation = system.get_ai_recommendation(&proposal_id);
        assert!(recommendation.is_some());
    }

    #[test]
    fn test_human_override() {
        let dao = create_test_dao();
        let engine = create_test_prediction_engine();
        let mut system = HybridGovernanceSystem::new(dao, engine);
        
        let proposal_id = system.create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            ProposalType::ParameterChange,
            Proposer::Human {
                trader_id: "voter1".to_string(),
            },
        ).unwrap();
        
        // Add human override
        let result = system.human_override(
            &proposal_id,
            "voter1".to_string(),
            ProposalStatus::Passed,
            "Human expert decision".to_string(),
        );
        
        assert!(result.is_ok());
        assert_eq!(system.get_metrics().human_overrides, 1);
    }

    #[test]
    fn test_proposal_tally_with_human_override() {
        let dao = create_test_dao();
        let engine = create_test_prediction_engine();
        let mut system = HybridGovernanceSystem::new(dao, engine);
        
        let proposal_id = system.create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            ProposalType::ParameterChange,
            Proposer::Human {
                trader_id: "voter1".to_string(),
            },
        ).unwrap();
        
        // Submit proposal
        system.submit_proposal(&proposal_id).unwrap();
        
        // Add human override
        system.human_override(
            &proposal_id,
            "voter1".to_string(),
            ProposalStatus::Passed,
            "Human expert decision".to_string(),
        ).unwrap();
        
        // Tally votes - should follow human override
        let result = system.tally_votes(&proposal_id).unwrap();
        assert_eq!(result, ProposalStatus::Passed);
    }

    #[test]
    fn test_proposal_tally_without_human_override() {
        let dao = create_test_dao();
        let engine = create_test_prediction_engine();
        let mut system = HybridGovernanceSystem::new(dao, engine);
        
        let proposal_id = system.create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            ProposalType::ParameterChange,
            Proposer::Human {
                trader_id: "voter1".to_string(),
            },
        ).unwrap();
        
        // Submit proposal
        system.submit_proposal(&proposal_id).unwrap();
        
        // Tally votes - should follow AI recommendation or DAO result
        let result = system.tally_votes(&proposal_id);
        // This might fail because voting hasn't ended, but that's expected behavior
        assert!(result.is_ok() || matches!(result, Err(GovernanceError::VotingNotEnded)));
    }

    #[test]
    fn test_metrics_tracking() {
        let dao = create_test_dao();
        let engine = create_test_prediction_engine();
        let mut system = HybridGovernanceSystem::new(dao, engine);
        
        assert_eq!(system.get_metrics().total_proposals, 0);
        assert_eq!(system.get_metrics().ai_recommendations, 0);
        assert_eq!(system.get_metrics().human_overrides, 0);
        
        // Create a proposal
        let proposal_id = system.create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            ProposalType::ParameterChange,
            Proposer::Human {
                trader_id: "voter1".to_string(),
            },
        ).unwrap();
        
        assert_eq!(system.get_metrics().total_proposals, 1);
        assert_eq!(system.get_metrics().ai_recommendations, 1);
        
        // Add human override
        system.human_override(
            &proposal_id,
            "voter1".to_string(),
            ProposalStatus::Passed,
            "Human expert decision".to_string(),
        ).unwrap();
        
        assert_eq!(system.get_metrics().human_overrides, 1);
    }
}