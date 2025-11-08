use std::collections::HashMap;

use dex_core::{
    reference_common::reference_root,
    security::{Certificate, EventType, SecurityManager},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SecurityRow {
    layer: String,
    component: String,
    behavior: String,
    condition: String,
    test_name: String,
}

fn load_security_row(component: &str, behavior: &str) -> SecurityRow {
    let csv_path = reference_root().join("security_tests_full.csv");
    let mut reader =
        csv::Reader::from_path(&csv_path).expect("failed to open security reference CSV");

    for row in reader.deserialize::<SecurityRow>() {
        let record = row.expect("failed to parse security CSV row");
        if record.component == component && record.behavior == behavior {
            return record;
        }
    }

    panic!("could not find security reference row for component {component} behavior {behavior}");
}

#[test]
fn security_reference_key_manager_rotates_keys() {
    let row = load_security_row("key_manager", "rotates");
    let mut manager = SecurityManager::new();

    let user_id = format!("{}_{}", row.component, row.condition);
    let first_key = manager
        .rotate_keys(&user_id)
        .expect("initial rotation should succeed");
    assert_eq!(first_key.algorithm, "Ed25519");

    // Rotate again to ensure history tracking matches the reference scenario.
    manager
        .rotate_keys(&user_id)
        .expect("subsequent rotation should succeed");
    let history = manager
        .key_rotation_history(&user_id)
        .expect("rotation history should exist");
    assert!(
        !history.is_empty(),
        "rotation history must record previous keys for reference {}",
        row.test_name
    );
}

#[test]
fn security_reference_vault_logs_evidence_events() {
    let row = load_security_row("vault", "logs_evidence");
    let mut manager = SecurityManager::new();

    // Certificates are the artifacts that the vault component manages.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let certificate = Certificate {
        id: format!("cert_{}", row.condition),
        data: vec![1, 2, 3],
        issuer: "SecurityOps".to_string(),
        valid_from: now - 10,
        valid_to: now + 600,
        signature: vec![0; 64],
        revoked: false,
    };
    manager
        .add_certificate(certificate.clone())
        .expect("certificate should be stored for evidence logging");

    let mut data = HashMap::new();
    data.insert("component".to_string(), row.component.clone());
    data.insert("behavior".to_string(), row.behavior.clone());
    manager.log_event(
        EventType::SecurityAlert,
        format!("{} evidence recorded", row.test_name),
        None,
        data,
    );

    let events = manager.get_events();
    let last_event = events.last().expect("expected at least one logged event");
    assert!(
        last_event.description.contains(&row.test_name),
        "event description should include reference slug {}",
        row.test_name
    );

    // Clean up by revoking to mirror the evidence lifecycle.
    manager
        .revoke_certificate(&certificate.id)
        .expect("revocation should succeed per security reference");
}
