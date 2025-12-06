use dex_core::security::{
    SecurityManager, Vulnerability, Severity, VulnerabilityStatus, LoadTestConfig,
    ThreatAssessmentManager, LoadValidationManager,
};
use dex_core::governance::iam::{IAM, Role, Permission};

#[test]
fn test_threat_assessment_manager() {
    let security_manager = SecurityManager::new();
    let threat_manager = ThreatAssessmentManager::new();

    // 1. Register a vulnerability
    let vuln = Vulnerability {
        id: "VULN-001".to_string(),
        title: "SQL Injection".to_string(),
        description: "Potential SQL injection in login".to_string(),
        severity: Severity::High,
        status: VulnerabilityStatus::Open,
        affected_component: "LoginService".to_string(),
        detected_at: 1000,
        resolved_at: None,
        remediation_plan: Some("Use parameterized queries".to_string()),
    };

    assert!(threat_manager.register_vulnerability(vuln.clone()).is_ok());

    // 2. Run assessment
    let report = threat_manager.run_assessment().unwrap();
    assert_eq!(report.vulnerabilities.len(), 1);
    assert_eq!(report.risk_score, 20); // High = 20

    // 3. Update status
    assert!(threat_manager.update_vulnerability_status("VULN-001", VulnerabilityStatus::Resolved).is_ok());

    // 4. Run assessment again
    let report_2 = threat_manager.run_assessment().unwrap();
    assert_eq!(report_2.vulnerabilities.len(), 0); // Resolved vulns not active
    assert_eq!(report_2.risk_score, 0);
}

#[test]
fn test_load_validation_manager() {
    let security_manager = SecurityManager::new();
    let load_manager = LoadValidationManager::new();

    // 1. Run a load test
    let config = LoadTestConfig {
        name: "LoginLoadTest".to_string(),
        target_component: "AuthService".to_string(),
        users: 100,
        duration_seconds: 10,
        requests_per_second: 50,
        ramp_up_seconds: 1,
    };

    let result = load_manager.run_load_test(config.clone()).unwrap();
    
    assert!(result.passed);
    assert!(result.success_rate > 0.99);
    assert_eq!(result.config.name, "LoginLoadTest");

    // 2. Check history
    let results = load_manager.get_results();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_iam_user_management_hashmap() {
    // This test verifies the User Management implementation which uses HashMaps
    let mut iam = IAM::new();
    let trader_id = "user_123";

    // 1. Register user (adds to internal HashMap)
    assert!(iam.register_user(&trader_id.to_string()).is_ok());

    // 2. Assign role (updates HashMap)
    assert!(iam.assign_role(trader_id, Role::SecurityAdmin).is_ok());

    // 3. Verify roles
    let roles = iam.get_user_roles(trader_id).unwrap();
    assert!(roles.contains(&Role::Trader)); // Default
    assert!(roles.contains(&Role::SecurityAdmin)); // Assigned

    // 4. Check permissions (lookup via HashMap)
    assert!(iam.has_permission(trader_id, &Permission::ManageKeys));
}