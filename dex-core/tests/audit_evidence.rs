use dex_core::governance::{AuditStore, GovernanceComponent, GovernanceDomain, GovernanceReferenceIndex};
use dex_core::security::SecurityManager;
use ed25519_dalek::{Signer, SigningKey};

#[test]
fn audit_store_ingests_verifies_and_logs() {
    // Locate AuditLogger scenario
    let index = GovernanceReferenceIndex::shared().expect("reference index");
    let scenario = index
        .scenarios()
        .iter()
        .find(|s| {
            s.domain == GovernanceDomain::AuditEvidenceManagement
                && s.component == GovernanceComponent::AuditLogger
        })
        .expect("audit_logger scenario present");

    let dir = tempfile::tempdir().expect("tempdir");
    let store = AuditStore::new(dir.path()).expect("store");

    // Generate key and sign content
    let signing = SigningKey::generate(&mut rand::rngs::OsRng);
    let verify = signing.verifying_key();
    let content = b"example sbom attestation";
    let sig = signing.sign(content);

    // Enforce filename from enrichment (e.g., sbom_attestation.sig)
    let filename = &scenario.enrichment.evidence;
    let mut sec = SecurityManager::new();
    let rec = store
        .ingest(
            "EVID-1",
            filename,
            content,
            sig.as_ref(),
            verify.as_bytes(),
            Some(&mut sec),
        )
        .expect("ingest");
    assert_eq!(&rec.filename, filename);

    // Verify signature and hash
    store.verify("EVID-1").expect("verify");

    // Ensure an audit trail event was logged
    assert!(sec
        .get_events()
        .iter()
        .any(|e| e.description.contains("Evidence EVID-1 ingested")));
}

