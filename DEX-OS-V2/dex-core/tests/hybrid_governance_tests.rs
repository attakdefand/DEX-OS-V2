//! Integration tests for the Hybrid Governance System

use dex_core::governance::{
    GlobalDAO, ProposalType, Proposer, ProposalStatus, HybridGovernanceSystem
};
use dex_core::prediction_engine::{
    PredictionEngine, AggregationStrategy, TransformerPredictor, ReinforcementLearningPredictor
};
use dex_core::types::TraderId;

fn create_test_prediction_engine() -> PredictionEngine {
    let models: Vec<Box<dyn dex_core::prediction_engine::Predictor>> = vec![
        Box::new(TransformerPredictor::new("transformer", 1.0, 42)),
        Box::new(ReinforcementLearningPredictor::new("rl", 24)),
    ];
    PredictionEngine::new(models, AggregationStrategy::ConfidenceWeighted)
}

fn create_test_dao() -> GlobalDAO {
    let mut dao = GlobalDAO::new();
    // Add test members
    dao.add_member("voter1".to_string(), 1000, false);
    dao.add_member("voter2".to_string(), 500, false);
    dao.add_member("admin".to_string(), 2000, true); // Council member
    dao
}

#[test]
fn test_hybrid_governance_system_full_lifecycle() {
    let dao = create_test_dao();
    let engine = create_test_prediction_engine();
    let mut system = HybridGovernanceSystem::new(dao, engine);
    
    // Test 1: System creation
    assert_eq!(system.get_metrics().total_proposals, 0);
    assert_eq!(system.get_metrics().ai_recommendations, 0);
    assert_eq!(system.get_metrics().human_overrides, 0);
    
    // Test 2: Create proposal with AI analysis
    let proposal_id = system.create_proposal(
        "Test Governance Proposal".to_string(),
        "This is a test proposal to validate the hybrid governance system".to_string(),
        ProposalType::ParameterChange,
        Proposer::Human {
            trader_id: "voter1".to_string(),
        },
    ).expect("Failed to create proposal");
    
    assert!(!proposal_id.is_empty());
    assert_eq!(system.get_metrics().total_proposals, 1);
    assert_eq!(system.get_metrics().ai_recommendations, 1);
    
    // Test 3: Check AI recommendation
    let ai_recommendation = system.get_ai_recommendation(&proposal_id);
    assert!(ai_recommendation.is_some());
    
    let recommendation = ai_recommendation.unwrap();
    assert_eq!(recommendation.proposal_id, proposal_id);
    assert!(!recommendation.rationale.is_empty());
    assert!(recommendation.confidence >= 0.0 && recommendation.confidence <= 1.0);
    
    // Test 4: Submit proposal for voting
    let submit_result = system.submit_proposal(&proposal_id);
    assert!(submit_result.is_ok());
    
    // Test 5: Cast votes
    let vote1_result = system.vote(
        &proposal_id,
        &"voter1".to_string(),
        true, // Support
        1000,
        Some("Strong support for this proposal".to_string()),
    );
    assert!(vote1_result.is_ok());
    
    let vote2_result = system.vote(
        &proposal_id,
        &"voter2".to_string(),
        false, // Against
        500,
        Some("Concerns about implementation timeline".to_string()),
    );
    assert!(vote2_result.is_ok());
    
    // Test 6: Add human override
    let override_result = system.human_override(
        &proposal_id,
        "admin".to_string(),
        ProposalStatus::Passed,
        "Administrative override based on strategic alignment".to_string(),
    );
    assert!(override_result.is_ok());
    assert_eq!(system.get_metrics().human_overrides, 1);
    
    // Test 7: Tally votes with hybrid decision making
    // First, we need to fast-forward time to end voting
    // Note: We can't directly access private proposal fields to fast-forward time
    // In a real scenario, time would naturally pass
    
    let tally_result = system.tally_votes(&proposal_id);
    assert!(tally_result.is_ok());
    
    let final_status = tally_result.unwrap();
    // Should follow human override
    assert_eq!(final_status, ProposalStatus::Passed);
}

#[test]
fn test_different_proposal_types() {
    let dao = create_test_dao();
    let engine = create_test_prediction_engine();
    let mut system = HybridGovernanceSystem::new(dao, engine);
    
    let proposal_types = vec![
        ProposalType::ParameterChange,
        ProposalType::TreasuryAllocation,
        ProposalType::ProtocolUpgrade,
        ProposalType::NewMarketListing,
        ProposalType::FeeStructureChange,
        ProposalType::EmergencyPause,
        ProposalType::TreasuryAutomation,
        ProposalType::ObservabilityUpgrade,
        ProposalType::AccessControlUpdate,
        ProposalType::ChangeManagementOverride,
        ProposalType::EducationProgramRefresh,
        ProposalType::Other("CustomProposalType".to_string()),
    ];
    
    for (i, proposal_type) in proposal_types.iter().enumerate() {
        let proposal_id = system.create_proposal(
            format!("Proposal Type Test {}", i),
            format!("Testing proposal type: {:?}", proposal_type),
            proposal_type.clone(),
            Proposer::Human {
                trader_id: "voter1".to_string(),
            },
        ).expect("Failed to create proposal");
        
        assert!(!proposal_id.is_empty());
        
        // Check that AI analysis was generated for each type
        let ai_recommendation = system.get_ai_recommendation(&proposal_id);
        assert!(ai_recommendation.is_some());
        
        // Check that impact analysis is appropriate for the proposal type
        let impact_analysis = &ai_recommendation.unwrap().impact_analysis;
        assert!(impact_analysis.liquidity_impact >= 0.0 && impact_analysis.liquidity_impact <= 1.0);
        assert!(impact_analysis.volume_impact >= 0.0 && impact_analysis.volume_impact <= 1.0);
        assert!(impact_analysis.adoption_impact >= 0.0 && impact_analysis.adoption_impact <= 1.0);
        assert!(impact_analysis.security_impact >= 0.0 && impact_analysis.security_impact <= 1.0);
    }
    
    assert_eq!(system.get_metrics().total_proposals, proposal_types.len() as u64);
    assert_eq!(system.get_metrics().ai_recommendations, proposal_types.len() as u64);
}

#[test]
fn test_ai_proposer_with_hybrid_governance() {
    let dao = create_test_dao();
    let engine = create_test_prediction_engine();
    let mut system = HybridGovernanceSystem::new(dao, engine);
    
    // Test AI-generated proposal
    let proposal_id = system.create_proposal(
        "AI-Generated Governance Proposal".to_string(),
        "This proposal was automatically generated by our AI systems based on market analysis".to_string(),
        ProposalType::ParameterChange,
        Proposer::AI {
            model_id: "governance_ai_v1".to_string(),
            confidence: 0.92,
            rationale: "Market analysis indicates this change will improve system efficiency by 15%".to_string(),
        },
    ).expect("Failed to create AI proposal");
    
    assert!(!proposal_id.is_empty());
    assert_eq!(system.get_metrics().total_proposals, 1);
    assert_eq!(system.get_metrics().ai_recommendations, 1);
    
    // Check AI recommendation for AI-generated proposal
    let ai_recommendation = system.get_ai_recommendation(&proposal_id);
    assert!(ai_recommendation.is_some());
    
    // Submit and test voting
    let submit_result = system.submit_proposal(&proposal_id);
    assert!(submit_result.is_ok());
    
    // Note: We can't directly access private proposal fields to fast-forward time
    // In a real scenario, time would naturally pass    
    // Tally votes - should follow AI recommendation for high-confidence AI proposals
    let tally_result = system.tally_votes(&proposal_id);
    assert!(tally_result.is_ok());
}

#[test]
fn test_hybrid_decision_making_scenarios() {
    let dao = create_test_dao();
    let engine = create_test_prediction_engine();
    let mut system = HybridGovernanceSystem::new(dao, engine);
    
    // Scenario 1: High confidence AI recommendation with human agreement
    let proposal_id1 = system.create_proposal(
        "High Confidence Proposal".to_string(),
        "This proposal has high likelihood of success".to_string(),
        ProposalType::ProtocolUpgrade,
        Proposer::Human {
            trader_id: "voter1".to_string(),
        },
    ).expect("Failed to create proposal");
    
    // Simulate high confidence AI recommendation
    // (In real implementation, this would depend on the AI model's prediction)
    
    // Add human override that agrees with AI
    let override1_result = system.human_override(
        &proposal_id1,
        "admin".to_string(),
        ProposalStatus::Passed,
        "Agree with AI analysis".to_string(),
    );
    assert!(override1_result.is_ok());
    
    // Scenario 2: Low confidence AI recommendation with human override
    let proposal_id2 = system.create_proposal(
        "Low Confidence Proposal".to_string(),
        "This proposal is experimental".to_string(),
        ProposalType::NewMarketListing,
        Proposer::Human {
            trader_id: "voter2".to_string(),
        },
    ).expect("Failed to create proposal");
    
    // Add human override that contradicts potential AI recommendation
    let override2_result = system.human_override(
        &proposal_id2,
        "admin".to_string(),
        ProposalStatus::Rejected,
        "Not aligned with current strategic priorities".to_string(),
    );
    assert!(override2_result.is_ok());
    
    // Check metrics
    assert_eq!(system.get_metrics().total_proposals, 2);
    assert_eq!(system.get_metrics().ai_recommendations, 2);
    assert_eq!(system.get_metrics().human_overrides, 2);
}

#[test]
fn test_governance_metrics_and_performance() {
    let dao = create_test_dao();
    let engine = create_test_prediction_engine();
    let mut system = HybridGovernanceSystem::new(dao, engine);
    
    // Initial metrics check
    let initial_metrics = system.get_metrics();
    assert_eq!(initial_metrics.total_proposals, 0);
    assert_eq!(initial_metrics.ai_recommendations, 0);
    assert_eq!(initial_metrics.human_overrides, 0);
    assert_eq!(initial_metrics.ai_human_alignment, 0.0);
    assert_eq!(initial_metrics.average_confidence, 0.0);
    
    // Create multiple proposals rapidly to test performance
    let start_time = std::time::Instant::now();
    
    for i in 0..50 {
        let proposal_id = system.create_proposal(
            format!("Performance Test Proposal {}", i),
            format!("This is performance test proposal #{}", i),
            ProposalType::ParameterChange,
            Proposer::Human {
                trader_id: if i % 2 == 0 { "voter1".to_string() } else { "voter2".to_string() },
            },
        ).expect("Failed to create proposal");
        
        // Add some human overrides
        if i % 10 == 0 {
            let _ = system.human_override(
                &proposal_id,
                "admin".to_string(),
                ProposalStatus::Passed,
                format!("Administrative approval for test proposal {}", i),
            );
        }
    }
    
    let duration = start_time.elapsed();
    
    // Check final metrics
    let final_metrics = system.get_metrics();
    assert_eq!(final_metrics.total_proposals, 50);
    assert_eq!(final_metrics.ai_recommendations, 50);
    assert!(final_metrics.human_overrides >= 5); // At least 5 overrides
    
    // Performance check - should complete in reasonable time
    assert!(duration.as_millis() < 10000); // Less than 10 seconds
    
    // Check active proposals with recommendations
    let active_with_rec = system.get_active_proposals_with_recommendations();
    assert!(!active_with_rec.is_empty());
}

#[test]
fn test_edge_cases_and_error_handling() {
    let dao = create_test_dao();
    let engine = create_test_prediction_engine();
    let mut system = HybridGovernanceSystem::new(dao, engine);
    
    // Test querying non-existent proposals
    assert!(system.get_ai_recommendation("non_existent_proposal").is_none());
    assert!(system.get_human_override("non_existent_override").is_none());
    
    // Test human override with non-existent proposal
    let override_result = system.human_override(
        "non_existent_proposal",
        "admin".to_string(),
        ProposalStatus::Passed,
        "Test override".to_string(),
    );
    assert!(override_result.is_ok()); // Should still succeed in recording the override
    
    // Test with empty proposal details
    let empty_proposal_id = system.create_proposal(
        "".to_string(),
        "".to_string(),
        ProposalType::Other("EmptyTest".to_string()),
        Proposer::Human {
            trader_id: "voter1".to_string(),
        },
    ).expect("Failed to create empty proposal");
    
    assert!(!empty_proposal_id.is_empty());
    
    // Test with very long proposal details
    let long_text = "A".repeat(10000); // 10KB of text
    let long_proposal_id = system.create_proposal(
        long_text.clone(),
        long_text,
        ProposalType::ParameterChange,
        Proposer::Human {
            trader_id: "voter1".to_string(),
        },
    ).expect("Failed to create long proposal");
    
    assert!(!long_proposal_id.is_empty());
    
    // Check that AI analysis still works with long text
    let long_ai_recommendation = system.get_ai_recommendation(&long_proposal_id);
    assert!(long_ai_recommendation.is_some());
}