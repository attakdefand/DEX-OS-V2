//! Certificate Manager for B+ Tree Certificate Management
//!
//! Implements certificate management functionality using a B+ Tree structure.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::security::{Certificate, SecurityError};

/// Certificate Manager for handling certificate operations
#[derive(Debug, Clone)]
pub struct CertificateManager {
    /// Storage for certificates using a HashMap as a simplified B+ Tree
    certificates: HashMap<String, Certificate>,
}

impl CertificateManager {
    /// Create a new CertificateManager
    pub fn new() -> Self {
        Self {
            certificates: HashMap::new(),
        }
    }

    /// Add a certificate to the manager
    pub fn add_certificate(&mut self, certificate: Certificate) -> Result<(), SecurityError> {
        if self.certificates.contains_key(&certificate.id) {
            return Err(SecurityError::CertificateAlreadyExists(certificate.id));
        }
        
        self.certificates.insert(certificate.id.clone(), certificate);
        Ok(())
    }

    /// Get a certificate by ID
    pub fn get_certificate(&self, id: &str) -> Option<&Certificate> {
        self.certificates.get(id)
    }

    /// Check if a certificate is valid
    pub fn is_certificate_valid(&self, id: &str) -> bool {
        if let Some(cert) = self.certificates.get(id) {
            // Check if certificate is revoked
            if cert.revoked {
                return false;
            }
            
            // Check if certificate is expired
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
                
            now >= cert.valid_from && now <= cert.valid_to
        } else {
            false
        }
    }

    /// Revoke a certificate
    pub fn revoke_certificate(&mut self, id: &str) -> Result<(), SecurityError> {
        if let Some(cert) = self.certificates.get_mut(id) {
            if cert.revoked {
                return Err(SecurityError::CertificateAlreadyRevoked(id.to_string()));
            }
            cert.revoked = true;
            Ok(())
        } else {
            Err(SecurityError::CertificateNotFound(id.to_string()))
        }
    }
}

impl Default for CertificateManager {
    fn default() -> Self {
        Self::new()
    }
}