use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use dex_core::reference_common::{reference_root, slugify};

#[derive(Debug, Deserialize)]
struct GovernanceRow {
    main_type: String,
    component: String,
    behavior: String,
    condition: String,
    test_name: String,
    owner: String,
    tool: String,
    metric: String,
    evidence: String,
}

const EXPECTED_ROW_COUNT: usize = 5_400;
const ROWS_PER_MAIN_TYPE: usize = 540;
const ROWS_PER_COMPONENT: usize = 600;

const EXPECTED_MAIN_TYPES: [&str; 10] = [
    "Governance Policy & Framework",
    "Access & Authorization Governance",
    "Change Management & Approval Flow",
    "Compliance & Regulatory Alignment",
    "Risk & Exception Management",
    "Audit & Evidence Management",
    "Policy-as-Code & Automation",
    "Transparency & Reporting",
    "DAO / On-Chain Governance",
    "Education & Culture of Accountability",
];

const EXPECTED_COMPONENTS: [&str; 9] = [
    "policy_engine",
    "role_manager",
    "approval_gate",
    "compliance_mapper",
    "risk_registry",
    "audit_logger",
    "rego_validator",
    "report_dashboard",
    "dao_governor",
];

struct Enrichment<'a> {
    owner: &'a str,
    tool: &'a str,
    metric: &'a str,
    evidence: &'a str,
}

fn expected_enrichment(main_type: &str) -> Enrichment<'_> {
    match main_type {
        "Governance Policy & Framework" => Enrichment {
            owner: "Security Governance Lead",
            tool: "OPA, CODEOWNERS, GitHub Protected Branches",
            metric: "policy_coverage_pct",
            evidence: "codeowners_version_hash",
        },
        "Access & Authorization Governance" => Enrichment {
            owner: "IAM Lead",
            tool: "Keycloak, OPA (Rego), josekit",
            metric: "least_privilege_violations",
            evidence: "access_matrix.json",
        },
        "Change Management & Approval Flow" => Enrichment {
            owner: "Release Manager + Security Reviewer",
            tool: "GitHub Actions, OPA Gate, Cosign",
            metric: "approval_latency_hours",
            evidence: "signed_change_approval.json",
        },
        "Compliance & Regulatory Alignment" => Enrichment {
            owner: "Compliance Officer",
            tool: "Drata/Vanta, OPA mappings",
            metric: "control_mapping_coverage",
            evidence: "iso_nist_mapping.yaml",
        },
        "Risk & Exception Management" => Enrichment {
            owner: "Risk Officer",
            tool: "Jira/Notion Risk DB",
            metric: "open_risks_count",
            evidence: "exception_register.csv",
        },
        "Audit & Evidence Management" => Enrichment {
            owner: "Internal Audit",
            tool: "immudb / AWS QLDB, Cosign",
            metric: "evidence_integrity_pct",
            evidence: "sbom_attestation.sig",
        },
        "Policy-as-Code & Automation" => Enrichment {
            owner: "Platform Security Engineer",
            tool: "OPA/Conftest, Terraform Sentinel",
            metric: "policy_gate_pass_rate",
            evidence: "policy_bundle_digest.txt",
        },
        "Transparency & Reporting" => Enrichment {
            owner: "Compliance Program Manager",
            tool: "Grafana, Loki, ClickHouse",
            metric: "audit_log_completeness_pct",
            evidence: "governance_dashboard_snapshot.png",
        },
        "DAO / On-Chain Governance" => Enrichment {
            owner: "DAO Council / Multisig",
            tool: "Snapshot, Governor Bravo, Gnosis Safe",
            metric: "quorum_participation_pct",
            evidence: "proposal_receipt.tx",
        },
        "Education & Culture of Accountability" => Enrichment {
            owner: "Security Awareness Lead",
            tool: "LMS (SecurityHub), Onboarding Portal",
            metric: "training_completion_pct",
            evidence: "training_completion_report.csv",
        },
        other => panic!("Unexpected main_type in CSV: {other}"),
    }
}

#[test]
fn governance_compliance_reference_is_consistent() {
    let csv_path = reference_root().join("governance_compliance_full_enriched.csv");
    assert!(
        csv_path.exists(),
        "missing governance reference file at {}",
        csv_path.display()
    );

    let mut reader =
        csv::Reader::from_path(&csv_path).expect("failed to open governance compliance CSV");

    let mut test_names = HashSet::new();
    let mut main_type_counts: HashMap<String, usize> = HashMap::new();
    let mut component_counts: HashMap<String, usize> = HashMap::new();
    let mut row_count = 0usize;

    for row in reader.deserialize::<GovernanceRow>() {
        row_count += 1;
        let record = row.expect("failed to deserialize governance CSV row");

        assert!(
            EXPECTED_MAIN_TYPES.contains(&record.main_type.as_str()),
            "unexpected main_type: {}",
            record.main_type
        );
        assert!(
            EXPECTED_COMPONENTS.contains(&record.component.as_str()),
            "unexpected component: {}",
            record.component
        );

        let enrichment = expected_enrichment(&record.main_type);
        assert_eq!(
            record.owner, enrichment.owner,
            "owner mismatch for {}",
            record.main_type
        );
        assert_eq!(
            record.tool, enrichment.tool,
            "tool mismatch for {}",
            record.main_type
        );
        assert_eq!(
            record.metric, enrichment.metric,
            "metric mismatch for {}",
            record.main_type
        );
        assert_eq!(
            record.evidence, enrichment.evidence,
            "evidence mismatch for {}",
            record.main_type
        );

        assert!(
            !record.behavior.trim().is_empty(),
            "behavior cannot be empty for test {}",
            record.test_name
        );
        assert!(
            !record.condition.trim().is_empty(),
            "condition cannot be empty for test {}",
            record.test_name
        );

        let expected_name = expected_test_name(&record);
        assert_eq!(
            record.test_name, expected_name,
            "test_name does not match canonical slug for row {:?}",
            record
        );

        assert!(
            test_names.insert(record.test_name.clone()),
            "duplicate test_name found: {}",
            record.test_name
        );

        *main_type_counts
            .entry(record.main_type.clone())
            .or_insert(0) += 1;
        *component_counts
            .entry(record.component.clone())
            .or_insert(0) += 1;
    }

    assert_eq!(
        row_count, EXPECTED_ROW_COUNT,
        "expected {EXPECTED_ROW_COUNT} rows but found {row_count}"
    );

    for main_type in EXPECTED_MAIN_TYPES {
        let count = main_type_counts.get(main_type).copied().unwrap_or(0);
        assert_eq!(
            count, ROWS_PER_MAIN_TYPE,
            "main_type {main_type} should have {ROWS_PER_MAIN_TYPE} rows but has {count}"
        );
    }

    for component in EXPECTED_COMPONENTS {
        let count = component_counts.get(component).copied().unwrap_or(0);
        assert_eq!(
            count, ROWS_PER_COMPONENT,
            "component {component} should have {ROWS_PER_COMPONENT} rows but has {count}"
        );
    }
}

fn expected_test_name(row: &GovernanceRow) -> String {
    format!(
        "test_governance_compliance__{}__{}__{}__{}",
        slugify(&row.main_type),
        slugify(&row.component),
        slugify(&row.behavior),
        slugify(&row.condition),
    )
}
