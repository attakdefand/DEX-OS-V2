use crate::security::{EventType, SecurityManager, SeverityLevel};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("io: {0}")]
    Io(String),
    #[error("signature verification failed")]
    Signature,
    #[error("evidence id already exists with different content hash")]
    ImmutableConflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub id: String,
    pub filename: String,
    pub content_hash: String,
    pub signature_hex: String,
    pub public_key_hex: String,
    pub timestamp: u64,
}

pub struct AuditStore {
    base_dir: PathBuf,
    index_path: PathBuf,
}

impl AuditStore {
    pub fn new<P: Into<PathBuf>>(base_dir: P) -> Result<Self, AuditError> {
        let base_dir = base_dir.into();
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).map_err(|e| AuditError::Io(e.to_string()))?;
        }
        let index_path = base_dir.join("index.jsonl");
        if !index_path.exists() {
            File::create(&index_path).map_err(|e| AuditError::Io(e.to_string()))?;
        }
        Ok(Self { base_dir, index_path })
    }

    pub fn from_env() -> Result<Self, AuditError> {
        let dir = env::var("EVIDENCE_DIR").unwrap_or_else(|_| "evidence".to_string());
        Self::new(dir)
    }

    fn content_hash(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let digest = hasher.finalize();
        encode_hex(&digest)
    }

    pub fn ingest(
        &self,
        id: &str,
        filename: &str,
        content: &[u8],
        signature: &[u8],
        public_key: &[u8],
        log: Option<&mut SecurityManager>,
    ) -> Result<EvidenceRecord, AuditError> {
        let pk_arr: [u8; 32] = public_key.try_into().map_err(|_| AuditError::Signature)?;
        let sig_arr: [u8; 64] = signature.try_into().map_err(|_| AuditError::Signature)?;
        let vk = VerifyingKey::from_bytes(&pk_arr).map_err(|_| AuditError::Signature)?;
        let sig = Signature::from_bytes(&sig_arr);
        vk.verify_strict(content, &sig).map_err(|_| AuditError::Signature)?;

        let hash = Self::content_hash(content);
        let blob = self.base_dir.join(&hash);
        if !blob.exists() {
            let mut f = File::create(&blob).map_err(|e| AuditError::Io(e.to_string()))?;
            f.write_all(content).map_err(|e| AuditError::Io(e.to_string()))?;
        } else if let Some(existing) = self.find_by_id(id)? {
            if existing.content_hash != hash { return Err(AuditError::ImmutableConflict); }
        }

        let rec = EvidenceRecord {
            id: id.to_string(), filename: filename.to_string(), content_hash: hash.clone(),
            signature_hex: encode_hex(signature), public_key_hex: encode_hex(public_key),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
        };

        let mut idx = OpenOptions::new().create(true).append(true).open(&self.index_path).map_err(|e| AuditError::Io(e.to_string()))?;
        let line = serde_json::to_string(&rec).map_err(|e| AuditError::Io(e.to_string()))? + "\n";
        idx.write_all(line.as_bytes()).map_err(|e| AuditError::Io(e.to_string()))?;

        if let Some(logger) = log {
            let mut data = std::collections::HashMap::new();
            data.insert("evidence_id".into(), id.to_string());
            data.insert("filename".into(), filename.to_string());
            data.insert("hash".into(), rec.content_hash.clone());
            logger.log_event(
                EventType::AuditTrail,
                format!("Evidence {} ingested", id),
                None,
                data,
                None,
                SeverityLevel::Info,
            );
        }

        Ok(rec)
    }

    pub fn verify(&self, id: &str) -> Result<(), AuditError> {
        let rec = self.find_by_id(id)?.ok_or_else(|| AuditError::Io("evidence id not found".into()))?;
        let mut file = File::open(self.base_dir.join(&rec.content_hash)).map_err(|e| AuditError::Io(e.to_string()))?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).map_err(|e| AuditError::Io(e.to_string()))?;
        let hash = Self::content_hash(&buf);
        if hash != rec.content_hash { return Err(AuditError::Io("content hash mismatch".into())); }
        let pk_vec = decode_hex(&rec.public_key_hex).map_err(|e| AuditError::Io(e.to_string()))?;
        let sig_vec = decode_hex(&rec.signature_hex).map_err(|e| AuditError::Io(e.to_string()))?;
        let pk_arr: [u8; 32] = pk_vec.as_slice().try_into().map_err(|_| AuditError::Signature)?;
        let sig_arr: [u8; 64] = sig_vec.as_slice().try_into().map_err(|_| AuditError::Signature)?;
        let vk = VerifyingKey::from_bytes(&pk_arr).map_err(|_| AuditError::Signature)?;
        let sig = Signature::from_bytes(&sig_arr);
        vk.verify_strict(&buf, &sig).map_err(|_| AuditError::Signature)?;
        Ok(())
    }

    fn find_by_id(&self, id: &str) -> Result<Option<EvidenceRecord>, AuditError> {
        let data = fs::read_to_string(&self.index_path).map_err(|e| AuditError::Io(e.to_string()))?;
        for line in data.lines() {
            if line.trim().is_empty() { continue; }
            let rec: EvidenceRecord = serde_json::from_str(line).map_err(|e| AuditError::Io(e.to_string()))?;
            if rec.id == id { return Ok(Some(rec)); }
        }
        Ok(None)
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes { use std::fmt::Write as _; let _ = write!(&mut out, "{:02x}", b); }
    out
}

fn decode_hex(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim(); if s.len() % 2 != 0 { return Err("odd hex length".into()); }
    let mut out = Vec::with_capacity(s.len() / 2); let bytes = s.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        let hi = (bytes[i] as char).to_digit(16).ok_or("invalid hex")?;
        let lo = (bytes[i+1] as char).to_digit(16).ok_or("invalid hex")?;
        out.push(((hi << 4) | lo) as u8);
    }
    Ok(out)
}
