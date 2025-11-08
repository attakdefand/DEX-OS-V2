use super::{load_governance_reference, GovernanceDomain, GovernanceScenario};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FrameworkRef {
    pub name: String,
    pub controls: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceEntry {
    pub test_name: String,
    pub behavior: String,
    pub condition: String,
    pub metric: String,
    pub evidence: String,
    pub frameworks: Vec<FrameworkRef>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceReport {
    pub entries: Vec<ComplianceEntry>,
}

fn map_frameworks(s: &GovernanceScenario) -> Vec<FrameworkRef> {
    if s.domain != GovernanceDomain::ComplianceRegulatoryAlignment {
        return vec![];
    }
    let mut frameworks = Vec::new();
    frameworks.push(FrameworkRef { name: "ISO 27001".to_string(), controls: vec!["A.6.1".into(), "A.12.1".into()] });
    frameworks.push(FrameworkRef { name: "NIST SP 800-53".to_string(), controls: vec!["AC-2".into(), "CM-3".into()] });
    if s.enrichment.evidence.to_lowercase().contains("iso") {
        frameworks.push(FrameworkRef { name: "ISO Mapping".to_string(), controls: vec!["iso_nist_mapping.yaml".into()] });
    }
    frameworks
}

pub fn build_compliance_report() -> Result<ComplianceReport, crate::governance::GovernanceReferenceError> {
    let scenarios = load_governance_reference()?;
    let mut entries = Vec::new();
    for s in scenarios.into_iter().filter(|s| s.domain == GovernanceDomain::ComplianceRegulatoryAlignment) {
        let frameworks = map_frameworks(&s);
        entries.push(ComplianceEntry {
            test_name: s.test_name.clone(),
            behavior: s.behavior.clone(),
            condition: s.condition.clone(),
            metric: s.enrichment.metric.clone(),
            evidence: s.enrichment.evidence.clone(),
            frameworks,
        });
    }
    Ok(ComplianceReport { entries })
}

pub fn render_report_json(report: &ComplianceReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
}
