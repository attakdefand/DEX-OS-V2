use dex_core::governance::{load_governance_reference, GovernanceComponent, GovernanceDomain};

const TARGET_TEST_NAME: &str =
    "test_governance_compliance__governance_policy_and_framework__policy_engine__defines_policy__during_commit";

#[test]
fn governance_reference_loader_returns_typed_scenarios() {
    let scenarios =
        load_governance_reference().expect("failed to load governance reference scenarios");

    let scenario = scenarios
        .iter()
        .find(|scenario| scenario.test_name == TARGET_TEST_NAME)
        .expect("expected to find canonical governance scenario");

    assert_eq!(scenario.domain, GovernanceDomain::GovernancePolicyFramework);
    assert_eq!(scenario.component, GovernanceComponent::PolicyEngine);
    assert_eq!(scenario.behavior, "defines_policy");
    assert_eq!(scenario.condition, "during_commit");

    let enrichment = &scenario.enrichment;
    assert_eq!(enrichment.owner, "Security Governance Lead");
    assert_eq!(
        enrichment.tool,
        "OPA, CODEOWNERS, GitHub Protected Branches"
    );
    assert_eq!(enrichment.metric, "policy_coverage_pct");
    assert_eq!(enrichment.evidence, "codeowners_version_hash");
}
