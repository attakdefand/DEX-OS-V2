//! Tests for Security Layer 22 - Kyber Encryption (Quantum-Resistant Security)

use dex_core::security::{
    KyberEncryptionManager, KyberError, KyberEncryptionOutput,
};

fn encrypt_round_trip(
    sender: &KyberEncryptionManager,
    recipient: &KyberEncryptionManager,
    payload: &[u8],
) -> (KyberEncryptionOutput, Vec<u8>) {
    let encryption = sender
        .encrypt_for(&recipient.public_key(), recipient.current_key_id(), payload)
        .expect("encryption should succeed");

    let decrypted = recipient.decrypt(&encryption.package).expect("decryption should succeed");
    (encryption, decrypted.plaintext)
}

#[test]
fn kyber_encryption_round_trip_and_shared_secret_match() {
    let sender = KyberEncryptionManager::new();
    let recipient = KyberEncryptionManager::new();

    let payload = b"quantum-safe settlement instruction";
    let (encryption, plaintext) = encrypt_round_trip(&sender, &recipient, payload);

    assert_eq!(plaintext, payload);
    let decrypted = recipient.decrypt(&encryption.package).unwrap();
    assert_eq!(encryption.shared_secret.key, decrypted.shared_secret.key);
    assert_eq!(encryption.package.key_id, recipient.current_key_id());
}

#[test]
fn kyber_decrypt_fails_with_wrong_private_key() {
    let sender = KyberEncryptionManager::new();
    let recipient = KyberEncryptionManager::new();
    let wrong_recipient = KyberEncryptionManager::new();

    let payload = b"route-to-right-recipient";
    let encryption = sender
        .encrypt_for(&recipient.public_key(), recipient.current_key_id(), payload)
        .unwrap();

    let err = wrong_recipient.decrypt(&encryption.package).unwrap_err();
    assert!(
        matches!(err, KyberError::AuthenticationFailed | KyberError::DecryptionFailed(_)),
        "unexpected error variant: {err:?}"
    );
}

#[test]
fn kyber_key_rotation_changes_identity_and_tracks_stats() {
    let manager = KyberEncryptionManager::new();
    let before_public = manager.public_key();
    let before_stats = manager.get_statistics();

    let rotated = manager.rotate_key();
    let after_public = manager.public_key();
    let after_stats = manager.get_statistics();

    assert_ne!(before_public, after_public, "public key should change after rotation");
    assert_eq!(rotated.key_id, manager.current_key_id());
    assert_eq!(after_stats.key_rotations, before_stats.key_rotations + 1);
}

#[test]
fn kyber_detects_tampered_encapsulated_key() {
    let sender = KyberEncryptionManager::new();
    let recipient = KyberEncryptionManager::new();

    let payload = b"integrity check payload";
    let mut encryption = sender
        .encrypt_for(&recipient.public_key(), recipient.current_key_id(), payload)
        .unwrap()
        .package;

    // Tamper with the KEM ciphertext to invalidate the authentication tag
    if let Some(first) = encryption.kem_ciphertext.first_mut() {
        *first ^= 0xFF;
    }

    let err = recipient.decrypt(&encryption).unwrap_err();
    assert!(matches!(err, KyberError::AuthenticationFailed));
}

#[test]
fn kyber_statistics_are_updated_for_success_and_failure_paths() {
    let manager = KyberEncryptionManager::new();

    let encryption = manager.encrypt_to_self(b"self-encrypted data").unwrap();
    let decrypted = manager.decrypt(&encryption.package).unwrap();
    assert_eq!(decrypted.plaintext, b"self-encrypted data");

    let mut tampered = encryption.package.clone();
    tampered.shared_secret_fingerprint[0] ^= 0x01;
    let _ = manager.decrypt(&tampered).unwrap_err();

    let stats = manager.get_statistics();
    assert!(stats.encryptions >= 1);
    assert!(stats.decryptions >= 1);
    assert!(stats.failures >= 1);
}
