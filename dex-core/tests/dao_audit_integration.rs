use dex_core::governance::{AuditInputs, AuditStore, GovernanceComponent, GovernanceDomain, GovernanceReferenceIndex, GlobalDAO, ProposalType, Proposer};
use ed25519_dalek::{Signer, SigningKey};

#[test]
fn dao_submit_with_audit_ingests_and_activates() {
    // Set up DAO and member
    let mut dao = GlobalDAO::new();
    let trader = "t1".to_string();
    dao.add_member(trader.clone(), 1000, false);

    // Create a proposal whose required reference is AuditLogger (ProtocolUpgrade)
    let pid = dao
        .create_proposal(
            "Upgrade Protocol".into(),
            "Upgrade requires audit evidence".into(),
            ProposalType::ProtocolUpgrade,
            Proposer::Human { trader_id: trader },
        )
        .expect("proposal");

    // Attach the required reference control
    dao.attach_reference_control(
        &pid,
        GovernanceDomain::AuditEvidenceManagement,
        GovernanceComponent::AuditLogger,
        "validates_policy",
        "during_deploy",
    )
    .expect("attach control");

    // Prepare an AuditStore and evidence matching enrichment filename
    let dir = tempfile::tempdir().expect("tempdir");
    let store = AuditStore::new(dir.path()).expect("store");
    let signing = SigningKey::generate(&mut rand::rngs::OsRng);
    let verify = signing.verifying_key();
    let content = b"example artifact";
    let sig = signing.sign(content);

    // Use the enrichment evidence filename from the attached reference for correctness
    let index = GovernanceReferenceIndex::shared().expect("reference index");
    let ref_scenario = index
        .scenarios()
        .iter()
        .find(|s| s.domain == GovernanceDomain::AuditEvidenceManagement && s.component == GovernanceComponent::AuditLogger)
        .expect("scenario");
    let filename = ref_scenario.enrichment.evidence.clone();

    let inputs = AuditInputs {
        id: "EVID-DAO-1".into(),
        filename,
        content: content.to_vec(),
        signature: sig.to_bytes().to_vec(),
        public_key: verify.as_bytes().to_vec(),
    };

    // Submit with audit; evidence is ingested and proposal becomes Active
    dao.submit_proposal_with_audit(&pid, &store, inputs)
        .expect("submit with audit");
    let p = dao.get_proposal(&pid).unwrap();
    assert_eq!(p.status, dex_core::governance::ProposalStatus::Active);
}

