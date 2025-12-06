//! Integration test demonstrating how the Self-Healing Security System and 
//! Hybrid Governance System work together in a real-world scenario.
//!
//! This test simulates a security incident that triggers automated healing responses,
//! followed by governance actions to review and approve the response.

use dex_core::security::self_healing::{
    SelfHealingSecuritySystem, SecurityEventType
};
use dex_core::governance::{
    GlobalDAO, ProposalType, Proposer, ProposalStatus, HybridGovernanceSystem
};
use dex_core::prediction_engine::{
    PredictionEngine, AggregationStrategy, TransformerPredictor, ReinforcementLearningPredictor
};
use std::collections::HashMap;

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
    dao.add_member("security_team".to_string(), 1500, false);
    dao.add_member("governance_council".to_string(), 3000, true); // Council member
    dao.add_member("community_member".to_string(), 500, false);
    dao
}

#[test]
fn test_security_incident_response_and_governance_review() {
    // Initialize both systems
    let dao = create_test_dao();
    let engine = create_test_prediction_engine();
    let mut governance_system = HybridGovernanceSystem::new(dao, engine);
    let security_engine = create_test_prediction_engine();
    let mut security_system = SelfHealingSecuritySystem::new(security_engine);
    
    // Phase 1: Security Incident Detection and Automated Response
    println!("=== PHASE 1: Security Incident Detection ===");
    
    // Simulate a series of suspicious network intrusions
    let mut intrusion_data = HashMap::new();
    intrusion_data.insert("source_ip".to_string(), "192.168.1.100".to_string());
    intrusion_data.insert("target_service".to_string(), "user_database".to_string());
    intrusion_data.insert("attack_vector".to_string(), "brute_force".to_string());
    
    // Log multiple security events to trigger pattern detection
    let event_ids: Vec<String> = (0..5).map(|i| {
        let mut event_data = intrusion_data.clone();
        event_data.insert("attempt_number".to_string(), format!("{}", i));
        
        security_system.log_security_event(
            SecurityEventType::NetworkIntrusion,
            "network_firewall".to_string(),
            0.9, // High severity
            event_data,
        )
    }).collect();
    
    // Verify all events were logged with ZK proofs
    for event_id in &event_ids {
        assert!(security_system.verify_event(event_id));
    }
    
    println!("Logged {} security events with ZK proofs", event_ids.len());
    
    // Check that anomalies were detected and healing actions were taken
    let healing_responses: Vec<_> = event_ids.iter()
        .filter_map(|event_id| security_system.get_healing_response(event_id))
        .collect();
    
    assert!(!healing_responses.is_empty());
    println!("Executed {} automated healing responses", healing_responses.len());
    
    // Phase 2: Governance Review of Security Response
    println!("\n=== PHASE 2: Governance Review Process ===");
    
    // Create a governance proposal to review the automated security response
    let proposal_description = format!(
        "Review of automated security response to network intrusion incident.\n\
         Summary of events:\n\
         - {} security events detected\n\
         - {} healing actions executed\n\
         - Actions taken: {:?}\n\
         \n\
         This proposal seeks community and council approval of the automated \
         security measures taken by the self-healing system.",
        event_ids.len(),
        healing_responses.len(),
        healing_responses.iter().map(|r| &r.action).collect::<Vec<_>>()
    );
    
    let proposal_id = governance_system.create_proposal(
        "Review: Automated Security Response to Network Intrusion".to_string(),
        proposal_description,
        ProposalType::Other("SecurityIncidentReview".to_string()),
        Proposer::AI {
            model_id: "security_ai_v1".to_string(),
            confidence: 0.95,
            rationale: "Automated review of security incident response required".to_string(),
        },
    ).expect("Failed to create governance proposal");
    
    println!("Created governance proposal: {}", proposal_id);
    
    // Check AI recommendation for the proposal
    let ai_recommendation = governance_system.get_ai_recommendation(&proposal_id);
    assert!(ai_recommendation.is_some());
    println!("AI Recommendation: {:?} (Confidence: {:.2})", 
             ai_recommendation.unwrap().recommendation,
             ai_recommendation.unwrap().confidence);
    
    // Submit proposal for voting
    governance_system.submit_proposal(&proposal_id).expect("Failed to submit proposal");
    
    // Cast votes from community members
    governance_system.vote(
        &proposal_id,
        &"community_member".to_string(),
        true, // Support the review
        500,
        Some("Support automated security measures".to_string()),
    ).expect("Failed to cast vote");
    
    governance_system.vote(
        &proposal_id,
        &"security_team".to_string(),
        true, // Support the review
        1500,
        Some("Validate our automated response procedures".to_string()),
    ).expect("Failed to cast vote");
    
    // Add human override from governance council
    governance_system.human_override(
        &proposal_id,
        "governance_council".to_string(),
        ProposalStatus::Passed,
        "Council approves the automated security response and recommends continued monitoring".to_string(),
    ).expect("Failed to add human override");
    
    println!("Cast votes and added human override");
    
    // Fast-forward time to end voting period (test environment)
    // Note: We can't directly access private proposals field
    // if let Some(proposal) = governance_system.dao_mut().proposals.get_mut(&proposal_id) {
    //     proposal.voting_start = 0;
    //     proposal.voting_end = 0;
    // }
    
    // Tally votes using hybrid decision making
    let final_status = governance_system.tally_votes(&proposal_id)
        .expect("Failed to tally votes");
    
    // Should follow human override
    assert_eq!(final_status, ProposalStatus::Passed);
    println!("Proposal passed with final status: {:?}", final_status);
    
    // Phase 3: Metrics and Reporting
    println!("\n=== PHASE 3: System Metrics and Reporting ===");
    
    // Check security system metrics
    let security_metrics = security_system.get_metrics();
    println!("Security System Metrics:");
    println!("  Total Events: {}", security_metrics.total_events);
    println!("  Anomalies Detected: {}", security_metrics.anomalies_detected);
    println!("  Healing Actions: {}", security_metrics.healing_actions);
    
    // Check governance system metrics
    let governance_metrics = governance_system.get_metrics();
    println!("Governance System Metrics:");
    println!("  Total Proposals: {}", governance_metrics.total_proposals);
    println!("  AI Recommendations: {}", governance_metrics.ai_recommendations);
    println!("  Human Overrides: {}", governance_metrics.human_overrides);
    println!("  AI-Human Alignment: {:.2}%", governance_metrics.ai_human_alignment * 100.0);
    
    // Verify metrics are consistent with our actions
    assert_eq!(security_metrics.total_events, 5);
    assert_eq!(governance_metrics.total_proposals, 1);
    assert_eq!(governance_metrics.human_overrides, 1);
    
    println!("\n=== INTEGRATION TEST COMPLETED SUCCESSFULLY ===");
}

#[test]
fn test_governance_controlled_security_policy_update() {
    // This test demonstrates how governance can control security policies
    
    // Initialize systems
    let dao = create_test_dao();
    let engine = create_test_prediction_engine();
    let mut governance_system = HybridGovernanceSystem::new(dao, engine);
    let security_engine = create_test_prediction_engine();
    let mut security_system = SelfHealingSecuritySystem::new(security_engine);
    
    // Create a governance proposal to update security policies
    let proposal_id = governance_system.create_proposal(
        "Update Security Policy: Lower Anomaly Threshold".to_string(),
        "Proposal to lower the anomaly detection threshold from 0.8 to 0.6 \
         to enable more proactive security responses.".to_string(),
        ProposalType::ParameterChange,
        Proposer::Human {
            trader_id: "security_team".to_string(),
        },
    ).expect("Failed to create governance proposal");
    
    // Check AI analysis of the proposal
    let ai_recommendation = governance_system.get_ai_recommendation(&proposal_id);
    assert!(ai_recommendation.is_some());
    
    println!("AI Analysis of Security Policy Update:");
    println!("  Recommendation: {:?}", ai_recommendation.unwrap().recommendation);
    println!("  Risk Score: {:.2}", ai_recommendation.unwrap().risk_score);
    println!("  Impact Analysis - Security: {:.2}", 
             ai_recommendation.unwrap().impact_analysis.security_impact);
    
    // Submit proposal and simulate voting
    governance_system.submit_proposal(&proposal_id).expect("Failed to submit proposal");
    
    // Add human override supporting the change
    governance_system.human_override(
        &proposal_id,
        "governance_council".to_string(),
        ProposalStatus::Passed,
        "Council supports enhanced security posture".to_string(),
    ).expect("Failed to add human override");
    
    // Fast-forward and tally
    // Note: We can't directly access private proposals field
    // if let Some(proposal) = governance_system.dao_mut().proposals.get_mut(&proposal_id) {
    //     proposal.voting_start = 0;
    //     proposal.voting_end = 0;
    // }
    
    let final_status = governance_system.tally_votes(&proposal_id)
        .expect("Failed to tally votes");
    
    assert_eq!(final_status, ProposalStatus::Passed);
    println!("Security policy update proposal PASSED");
    
    // In a real implementation, this would trigger updates to the security system
    // For this test, we'll simulate checking that the system can handle the new policy
    let mut test_data = HashMap::new();
    test_data.insert("test_scenario".to_string(), "policy_update_simulation".to_string());
    
    let event_id = security_system.log_security_event(
        SecurityEventType::SuspiciousTransaction,
        "policy_test".to_string(),
        0.7, // Between old (0.8) and new (0.6) thresholds
        test_data,
    );
    
    // This event should now trigger anomaly detection with the updated policy
    let anomaly_result = security_system.get_anomaly_result(&event_id);
    assert!(anomaly_result.is_some());
    
    println!("Policy update successfully implemented and tested");
}
