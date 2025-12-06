use crate::types::TraderId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Errors related to threat assessment
#[derive(Debug, Error)]
pub enum ThreatAssessmentError {
    #[error("Vulnerability not found: {0}")]
    VulnerabilityNotFound(String),
    #[error("Assessment failed: {0}")]
    AssessmentFailed(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

/// Severity level of a vulnerability
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Status of a vulnerability
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VulnerabilityStatus {
    Open,
    InProgress,
    Resolved,
    FalsePositive,
    Ignored,
}

/// Represents a detected or known vulnerability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub status: VulnerabilityStatus,
    pub affected_component: String,
    pub detected_at: u64,
    pub resolved_at: Option<u64>,
    pub remediation_plan: Option<String>,
}

/// Report generated after a threat assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAssessmentReport {
    pub id: String,
    pub timestamp: u64,
    pub vulnerabilities: Vec<Vulnerability>,
    pub summary: String,
    pub risk_score: u32, // 0-100
}

/// Manager for Threat Assessment and Vulnerability Management (Security Layer 13)
#[derive(Debug, Clone)]
pub struct ThreatAssessmentManager {
    /// Known vulnerabilities database (in-memory for now)
    vulnerabilities: Arc<RwLock<HashMap<String, Vulnerability>>>,
    /// History of assessment reports
    reports: Arc<RwLock<Vec<ThreatAssessmentReport>>>,
}

impl ThreatAssessmentManager {
    pub fn new() -> Self {
        Self {
            vulnerabilities: Arc::new(RwLock::new(HashMap::new())),
            reports: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a new vulnerability
    pub fn register_vulnerability(&self, vulnerability: Vulnerability) -> Result<(), ThreatAssessmentError> {
        let mut vulns = self.vulnerabilities.write().map_err(|_| ThreatAssessmentError::AssessmentFailed("Lock error".into()))?;
        vulns.insert(vulnerability.id.clone(), vulnerability);
        Ok(())
    }

    /// Update the status of a vulnerability
    pub fn update_vulnerability_status(&self, id: &str, status: VulnerabilityStatus) -> Result<(), ThreatAssessmentError> {
        let mut vulns = self.vulnerabilities.write().map_err(|_| ThreatAssessmentError::AssessmentFailed("Lock error".into()))?;
        if let Some(vuln) = vulns.get_mut(id) {
            vuln.status = status;
            if matches!(vuln.status, VulnerabilityStatus::Resolved) {
                vuln.resolved_at = Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
            }
            Ok(())
        } else {
            Err(ThreatAssessmentError::VulnerabilityNotFound(id.to_string()))
        }
    }

    /// Run a system-wide threat assessment (simulated)
    pub fn run_assessment(&self) -> Result<ThreatAssessmentReport, ThreatAssessmentError> {
        // In a real system, this would scan code, dependencies, and configuration.
        // Here we simulate a scan based on registered vulnerabilities and some heuristics.
        
        let vulns = self.vulnerabilities.read().map_err(|_| ThreatAssessmentError::AssessmentFailed("Lock error".into()))?;
        let active_vulns: Vec<Vulnerability> = vulns.values()
            .filter(|v| v.status == VulnerabilityStatus::Open || v.status == VulnerabilityStatus::InProgress)
            .cloned()
            .collect();

        let risk_score = self.calculate_risk_score(&active_vulns);
        
        let report = ThreatAssessmentReport {
            id: format!("report_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()),
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            vulnerabilities: active_vulns,
            summary: format!("Assessment completed. Risk Score: {}", risk_score),
            risk_score,
        };

        let mut reports = self.reports.write().map_err(|_| ThreatAssessmentError::AssessmentFailed("Lock error".into()))?;
        reports.push(report.clone());

        Ok(report)
    }

    /// Calculate risk score based on active vulnerabilities
    fn calculate_risk_score(&self, vulnerabilities: &[Vulnerability]) -> u32 {
        let mut score = 0;
        for vuln in vulnerabilities {
            match vuln.severity {
                Severity::Critical => score += 40,
                Severity::High => score += 20,
                Severity::Medium => score += 10,
                Severity::Low => score += 5,
            }
        }
        std::cmp::min(score, 100)
    }

    /// Get all reports
    pub fn get_reports(&self) -> Vec<ThreatAssessmentReport> {
        self.reports.read().unwrap().clone()
    }
    
    /// Get a specific vulnerability
    pub fn get_vulnerability(&self, id: &str) -> Option<Vulnerability> {
        self.vulnerabilities.read().unwrap().get(id).cloned()
    }
}

impl Default for ThreatAssessmentManager {
    fn default() -> Self {
        Self::new()
    }
}
