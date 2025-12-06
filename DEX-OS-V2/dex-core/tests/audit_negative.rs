use dex_core::governance::AuditStore;
use ed25519_dalek::{Signer, SigningKey};

#[test]
fn audit_rejects_bad_signature() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = AuditStore::new(dir.path()).expect("store");

    let signing = SigningKey::generate(&mut rand::rngs::OsRng);
    let wrong = SigningKey::generate(&mut rand::rngs::OsRng);
    let content = b"evidence";
    let sig = wrong.sign(content); // signed by wrong key
    let sig_bytes = sig.to_bytes();

    let err = store
        .ingest(
            "E-1",
            "artifact.sig",
            content,
            &sig_bytes,
            signing.verifying_key().as_bytes(),
            None,
        )
        .expect_err("should fail signature");
    let msg = format!("{err:?}");
    assert!(msg.contains("Signature"));
}

#[test]
fn audit_immutable_conflict_on_different_content_same_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = AuditStore::new(dir.path()).expect("store");

    let signing = SigningKey::generate(&mut rand::rngs::OsRng);
    let vk = signing.verifying_key();

    let c1 = b"content-one";
    let s1 = signing.sign(c1);
    let s1_bytes = s1.to_bytes();
    store
        .ingest("ID-1", "a.sig", c1, &s1_bytes, vk.as_bytes(), None)
        .expect("ingest 1");

    // Different content but same id should error (immutable conflict)
    let c2 = b"content-two";
    let s2 = signing.sign(c2);
    let s2_bytes = s2.to_bytes();
    let err = store
        .ingest("ID-1", "a.sig", c2, &s2_bytes, vk.as_bytes(), None)
        .expect_err("should conflict");
    let msg = format!("{err:?}");
    assert!(msg.contains("ImmutableConflict"));
}
