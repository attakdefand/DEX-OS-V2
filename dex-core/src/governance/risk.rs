use super::{GovernanceComponent, GovernanceDomain, GovernanceReferenceIndex};
use crate::security::{EventType, SecurityManager, SeverityLevel};
use serde::{Deserialize, Serialize};
use std::{env, path::Path};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskItem {
    pub id: String,
    pub title: String,
    pub severity: String,
    pub open: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExceptionRequest {
    pub id: String,
    pub description: String,
    pub approved: bool,
    pub approver: Option<String>,
    pub owner: String,
    pub timeframe: String,
    pub justification: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    pub message: String,
}

#[derive(Debug, Default)]
pub struct RiskRegistryState {
    pub risks: Vec<RiskItem>,
    pub exceptions: Vec<ExceptionRequest>,
    pub notifications: Vec<Notification>,
}

#[derive(Debug)]
pub enum RiskError {
    ScenarioMissing,
    EvidenceMismatch { expected: String, got: String },
    MissingApproverRole(&'static str),
    Io(String),
}

pub struct RiskRegistry<'a> {
    index: &'a GovernanceReferenceIndex,
    state: RiskRegistryState,
}

impl<'a> RiskRegistry<'a> {
    pub fn new(index: &'a GovernanceReferenceIndex) -> Result<Self, RiskError> {
        let has = index
            .scenarios()
            .iter()
            .any(|s| s.domain == GovernanceDomain::RiskExceptionManagement
                && s.component == GovernanceComponent::RiskRegistry);
        if !has {
            return Err(RiskError::ScenarioMissing);
        }
        Ok(Self { index, state: RiskRegistryState::default() })
    }

    fn scenario(&self) -> Result<&super::reference::GovernanceScenario, RiskError> {
        self.index
            .scenarios()
            .iter()
            .find(|s| s.domain == GovernanceDomain::RiskExceptionManagement
                && s.component == GovernanceComponent::RiskRegistry)
            .ok_or(RiskError::ScenarioMissing)
    }

    pub fn add_risk(&mut self, id: impl Into<String>, title: impl Into<String>, severity: impl Into<String>) {
        self.state.risks.push(RiskItem { id: id.into(), title: title.into(), severity: severity.into(), open: true });
    }

    pub fn mitigate_risk(&mut self, id: &str) {
        if let Some(r) = self.state.risks.iter_mut().find(|r| r.id == id) {
            r.open = false;
            self.state.notifications.push(Notification { message: format!("Risk {} mitigated", id) });
        }
    }

    pub fn submit_exception(&mut self, id: impl Into<String>, description: impl Into<String>) {
        self.state.exceptions.push(ExceptionRequest {
            id: id.into(), description: description.into(), approved: false, approver: None,
            owner: "".into(), timeframe: "".into(), justification: "".into(), timestamp: current_ts(),
        });
    }

    pub fn submit_exception_detailed(
        &mut self,
        id: impl Into<String>,
        description: impl Into<String>,
        owner: impl Into<String>,
        timeframe: impl Into<String>,
        justification: impl Into<String>,
    ) {
        self.state.exceptions.push(ExceptionRequest {
            id: id.into(), description: description.into(), approved: false, approver: None,
            owner: owner.into(), timeframe: timeframe.into(), justification: justification.into(), timestamp: current_ts(),
        });
    }

    pub fn approve_exception(
        &mut self,
        exception_id: &str,
        approver_login: &str,
        approver_role: &str,
        evidence_artifact: &str,
    ) -> Result<(), RiskError> {
        let scenario = self.scenario()?;
        let owner = scenario.enrichment.owner.to_lowercase();
        if owner.contains("risk officer") && !approver_role.to_lowercase().contains("risk officer") {
            return Err(RiskError::MissingApproverRole("Risk Officer"));
        }
        let expected = &scenario.enrichment.evidence;
        if expected != evidence_artifact {
            return Err(RiskError::EvidenceMismatch { expected: expected.clone(), got: evidence_artifact.to_string() });
        }
        if let Some(ex) = self.state.exceptions.iter_mut().find(|e| e.id == exception_id) {
            ex.approved = true;
            ex.approver = Some(approver_login.to_string());
            self.state.notifications.push(Notification { message: format!("Exception {} approved by {}", exception_id, approver_login) });
        }
        Ok(())
    }

    pub fn approve_exception_with_log(
        &mut self,
        exception_id: &str,
        approver_login: &str,
        approver_role: &str,
        evidence_artifact: &str,
        logger: &mut SecurityManager,
    ) -> Result<(), RiskError> {
        self.approve_exception(exception_id, approver_login, approver_role, evidence_artifact)?;
        let mut data = std::collections::HashMap::new();
        data.insert("component".into(), "risk_registry".into());
        data.insert("exception_id".into(), exception_id.into());
        data.insert("approver".into(), approver_login.into());
        data.insert("evidence".into(), evidence_artifact.into());
        logger.log_event(
            EventType::PolicyViolation,
            format!("Exception {} approved", exception_id),
            None,
            data,
            None,
            SeverityLevel::Info,
        );
        Ok(())
    }

    pub fn save_exception_register(&self) -> Result<(), RiskError> {
        let path = exception_register_path();
        let mut wtr = csv::Writer::from_path(&path).map_err(|e| RiskError::Io(e.to_string()))?;
        wtr.write_record(&["id","description","approved","approver","owner","timeframe","justification","timestamp"]).map_err(|e| RiskError::Io(e.to_string()))?;
        for ex in &self.state.exceptions {
            wtr.write_record(&[
                ex.id.as_str(), ex.description.as_str(), if ex.approved { "true" } else { "false" },
                ex.approver.as_deref().unwrap_or(""), ex.owner.as_str(), ex.timeframe.as_str(), ex.justification.as_str(), &ex.timestamp.to_string(),
            ]).map_err(|e| RiskError::Io(e.to_string()))?;
        }
        wtr.flush().map_err(|e| RiskError::Io(e.to_string()))?;
        Ok(())
    }

    pub fn load_exception_register(&mut self) -> Result<(), RiskError> {
        let path = exception_register_path();
        if !Path::new(&path).exists() { return Ok(()); }
        let mut rdr = csv::Reader::from_path(&path).map_err(|e| RiskError::Io(e.to_string()))?;
        for rec in rdr.deserialize::<ExceptionCsvRow>() {
            let r: ExceptionCsvRow = rec.map_err(|e| RiskError::Io(e.to_string()))?;
            if self.state.exceptions.iter().any(|e| e.id == r.id) { continue; }
            self.state.exceptions.push(ExceptionRequest {
                id: r.id, description: r.description, approved: r.approved,
                approver: if r.approver.is_empty() { None } else { Some(r.approver) }, owner: r.owner,
                timeframe: r.timeframe, justification: r.justification, timestamp: r.timestamp,
            });
        }
        Ok(())
    }

    pub fn open_risks_count(&self) -> usize { self.state.risks.iter().filter(|r| r.open).count() }
    pub fn notifications(&self) -> &[Notification] { &self.state.notifications }
}

#[derive(Deserialize)]
struct ExceptionCsvRow { id: String, description: String, approved: bool, approver: String, owner: String, timeframe: String, justification: String, timestamp: u64 }

fn exception_register_path() -> String {
    env::var("EXCEPTION_REGISTER_PATH").or_else(|_| env::var("CI_EXCEPTION_REGISTER_PATH")).unwrap_or_else(|_| "exception_register.csv".to_string())
}

fn current_ts() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default()
}
