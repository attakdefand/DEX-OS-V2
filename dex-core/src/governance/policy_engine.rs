use super::{GovernanceComponent, GovernanceDomain, GovernanceScenario};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Checkpoint {
    Commit,
    Merge,
    Deploy,
    Ci,
    AfterDeploy,
    OnRequest,
    Quarterly,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyEffect {
    Define,
    Enforce,
    Approve,
    Review,
    Validate,
    Monitor,
    Gate,
    Other(String),
}

/// Parse a behavior like "defines_policy" into a policy effect.
pub fn parse_effect(behavior: &str) -> PolicyEffect {
    let b = behavior.trim().to_lowercase();
    let verb = b.strip_suffix("_policy").unwrap_or(&b).to_string();
    match verb.as_str() {
        "defines" => PolicyEffect::Define,
        "enforces" => PolicyEffect::Enforce,
        "approves" => PolicyEffect::Approve,
        "reviews" => PolicyEffect::Review,
        "validates" => PolicyEffect::Validate,
        "monitors" => PolicyEffect::Monitor,
        "gates" => PolicyEffect::Gate,
        // Allow future verbs without breaking parsing
        other => PolicyEffect::Other(other.to_string()),
    }
}

/// Parse a condition like "during_commit" into a checkpoint.
pub fn parse_checkpoint(condition: &str) -> Checkpoint {
    let c = condition.trim().to_lowercase();
    if let Some(rest) = c.strip_prefix("during_") {
        match rest {
            "commit" => Checkpoint::Commit,
            "merge" => Checkpoint::Merge,
            "deploy" => Checkpoint::Deploy,
            "ci" => Checkpoint::Ci,
            "after_deploy" => Checkpoint::AfterDeploy,
            "on_request" => Checkpoint::OnRequest,
            "quarterly" => Checkpoint::Quarterly,
            other => Checkpoint::Other(other.to_string()),
        }
    } else {
        match c.as_str() {
            "commit" => Checkpoint::Commit,
            "merge" => Checkpoint::Merge,
            "deploy" => Checkpoint::Deploy,
            "ci" => Checkpoint::Ci,
            "after_deploy" => Checkpoint::AfterDeploy,
            "on_request" => Checkpoint::OnRequest,
            "quarterly" => Checkpoint::Quarterly,
            other => Checkpoint::Other(other.to_string()),
        }
    }
}

pub fn applies_at(s: &GovernanceScenario, checkpoint: Checkpoint) -> bool {
    parse_checkpoint(&s.condition) == checkpoint
}

/// Compute the (PolicyEffect, Checkpoint) tuple for a governance scenario.
pub fn policy_for(s: &GovernanceScenario) -> (PolicyEffect, Checkpoint) {
    (parse_effect(&s.behavior), parse_checkpoint(&s.condition))
}

/// Helper to check if a scenario targets the policy engine domain/component.
pub fn is_policy_engine(s: &GovernanceScenario) -> bool {
    s.domain == GovernanceDomain::GovernancePolicyFramework
        && s.component == GovernanceComponent::PolicyEngine
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::reference::GovernanceScenario as S;

    #[test]
    fn parses_effects() {
        assert_eq!(parse_effect("defines_policy"), PolicyEffect::Define);
        assert_eq!(parse_effect("enforces_policy"), PolicyEffect::Enforce);
        assert_eq!(parse_effect("approves_policy"), PolicyEffect::Approve);
        assert_eq!(parse_effect("reviews_policy"), PolicyEffect::Review);
        assert_eq!(parse_effect("validates_policy"), PolicyEffect::Validate);
        assert_eq!(parse_effect("monitors_policy"), PolicyEffect::Monitor);
        assert_eq!(parse_effect("gates_policy"), PolicyEffect::Gate);
        assert_eq!(
            parse_effect("customize_policy"),
            PolicyEffect::Other("customize".into())
        );
    }

    #[test]
    fn parses_checkpoints() {
        assert_eq!(parse_checkpoint("during_commit"), Checkpoint::Commit);
        assert_eq!(parse_checkpoint("during_merge"), Checkpoint::Merge);
        assert_eq!(parse_checkpoint("during_deploy"), Checkpoint::Deploy);
        assert_eq!(parse_checkpoint("during_ci"), Checkpoint::Ci);
        assert_eq!(parse_checkpoint("during_after_deploy"), Checkpoint::AfterDeploy);
        assert_eq!(parse_checkpoint("during_on_request"), Checkpoint::OnRequest);
        assert_eq!(parse_checkpoint("during_quarterly"), Checkpoint::Quarterly);
        assert_eq!(
            parse_checkpoint("during_release"),
            Checkpoint::Other("release".into())
        );
        assert_eq!(parse_checkpoint("commit"), Checkpoint::Commit);
    }

    #[test]
    fn applies_at_matches_condition() {
        let s = S {
            domain: GovernanceDomain::GovernancePolicyFramework,
            component: GovernanceComponent::PolicyEngine,
            behavior: "defines_policy".into(),
            condition: "during_commit".into(),
            test_name: "t".into(),
            enrichment: super::super::reference::Enrichment {
                owner: "o".into(),
                tool: "t".into(),
                metric: "m".into(),
                evidence: "e".into(),
            },
        };
        assert!(applies_at(&s, Checkpoint::Commit));
        assert!(!applies_at(&s, Checkpoint::Deploy));
    }
}
