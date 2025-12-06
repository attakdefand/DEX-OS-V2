//! Zero Gas Execution implementation for the DEX-OS core engine
//!
//! This module implements the Priority 3 features from DEX-OS-V2.csv:
//! - Zero Gas Execution features (AI-Optimized Routing, No Metering, 99.999% Uptime)
//!
//! It provides functionality for gas abstraction, AI-optimized transaction routing,
//! and high-availability execution services.

use crate::types::{TokenId, TraderId, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

const FIVE_NINES_WINDOW_SECS: u64 = 30 * 24 * 60 * 60; // 30 days
const FIVE_NINES_MAX_DOWNTIME_SECS: u64 = 26; // ~25.92s budget for 99.999% uptime

/// Gas abstraction service for zero-gas transactions
#[derive(Debug, Clone)]
pub struct GasAbstractionService {
    /// Relayers that can submit transactions on behalf of users
    relayers: HashMap<String, Relayer>,
    /// User gas sponsorship information
    sponsorships: HashMap<TraderId, GasSponsorship>,
    /// Transaction queue for gas abstraction
    transaction_queue: Vec<AbstractedTransaction>,
    /// AI routing engine for optimal transaction execution
    ai_router: AIRouter,
    /// Service availability tracker
    availability_tracker: AvailabilityTracker,
}

/// Relayer information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relayer {
    /// Relayer identifier
    pub id: String,
    /// Relayer address
    pub address: String,
    /// Relayer reputation score
    pub reputation: f32,
    /// Available balance for gas payments
    pub balance: u64,
    /// Maximum transactions per second
    pub max_tps: u32,
    /// Current transaction count in this period
    pub current_tps: u32,
    /// Whether the relayer is currently active
    pub is_active: bool,
}

/// Gas sponsorship information for a user
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GasSponsorship {
    /// Sponsor identifier (could be DAO, protocol, or individual)
    pub sponsor_id: String,
    /// Maximum gas allowance per day
    pub daily_limit: u64,
    /// Used gas in current period
    pub used_gas: u64,
    /// Last reset timestamp
    pub last_reset: u64,
    /// Whether sponsorship is active
    pub is_active: bool,
}

/// Transaction with gas abstraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractedTransaction {
    /// Original transaction
    pub transaction: Transaction,
    /// User who initiated the transaction
    pub user: TraderId,
    /// Sponsor for gas costs (if any)
    pub sponsor: Option<String>,
    /// Relayer that will submit the transaction
    pub relayer: Option<String>,
    /// Estimated gas cost
    pub estimated_gas: u64,
    /// Priority level (0-10, higher is more urgent)
    pub priority: u8,
    /// Submission timestamp
    pub submitted_at: u64,
    /// Status of the transaction
    pub status: TransactionStatus,
}

/// Status of an abstracted transaction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    InQueue,
    Submitted,
    Confirmed,
    Failed,
    Cancelled,
}

/// AI-powered router for optimal transaction execution
#[derive(Debug, Clone)]
pub struct AIRouter {
    /// Performance metrics for different execution paths
    path_metrics: HashMap<String, PathMetrics>,
    /// Current network conditions
    network_conditions: NetworkConditions,
    /// Historical performance data
    history: Vec<ExecutionRecord>,
}

/// Metrics for an execution path
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathMetrics {
    /// Average execution time in milliseconds
    pub avg_execution_time: u64,
    /// Success rate (0.0 - 1.0)
    pub success_rate: f32,
    /// Average cost in gas units
    pub avg_cost: u64,
    /// Last updated timestamp
    pub last_updated: u64,
}

/// Current network conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkConditions {
    /// Current gas price
    pub gas_price: u64,
    /// Network congestion level (0-100)
    pub congestion: u8,
    /// Block time in seconds
    pub block_time: u64,
    /// Pending transaction count
    pub pending_transactions: u64,
}

/// Record of an execution for AI learning
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Transaction type
    pub tx_type: String,
    /// Execution path used
    pub path: String,
    /// Actual execution time
    pub execution_time: u64,
    /// Actual cost
    pub cost: u64,
    /// Success or failure
    pub success: bool,
    /// Timestamp
    pub timestamp: u64,
}

/// Availability tracker for 99.999% uptime
#[derive(Debug, Clone)]
pub struct AvailabilityTracker {
    /// Service uptime history
    uptime_history: Vec<AvailabilityRecord>,
    /// Current service status
    current_status: ServiceStatus,
    /// Alert thresholds
    alert_thresholds: AlertThresholds,
}

/// Record of service availability
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AvailabilityRecord {
    /// Timestamp
    pub timestamp: u64,
    /// Service status
    pub status: ServiceStatus,
    /// Any incident details
    pub incident: Option<String>,
    /// Duration in seconds
    pub duration: u64,
}

/// Service status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Operational,
    Degraded,
    Outage,
    Maintenance,
}

/// Alert thresholds for service monitoring
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Maximum acceptable downtime per month (seconds)
    pub max_downtime: u64,
    /// Response time threshold for alerts (milliseconds)
    pub response_time_threshold: u64,
    /// Error rate threshold (0.0 - 1.0)
    pub error_rate_threshold: f32,
    /// Rolling window used for uptime calculations (seconds)
    pub window_secs: u64,
}

impl GasAbstractionService {
    /// Create a new gas abstraction service
    pub fn new() -> Self {
        Self {
            relayers: HashMap::new(),
            sponsorships: HashMap::new(),
            transaction_queue: Vec::new(),
            ai_router: AIRouter::new(),
            availability_tracker: AvailabilityTracker::new(),
        }
    }

    /// Add a relayer to the service
    pub fn add_relayer(&mut self, relayer: Relayer) {
        self.relayers.insert(relayer.id.clone(), relayer);
    }

    /// Remove a relayer from the service
    pub fn remove_relayer(&mut self, relayer_id: &str) -> Result<(), GasAbstractionError> {
        if self.relayers.remove(relayer_id).is_some() {
            Ok(())
        } else {
            Err(GasAbstractionError::RelayerNotFound)
        }
    }

    /// Set gas sponsorship for a user
    pub fn set_sponsorship(&mut self, user: TraderId, sponsorship: GasSponsorship) {
        self.sponsorships.insert(user, sponsorship);
    }

    /// Remove gas sponsorship for a user
    pub fn remove_sponsorship(&mut self, user: &TraderId) -> Result<(), GasAbstractionError> {
        if self.sponsorships.remove(user).is_some() {
            Ok(())
        } else {
            Err(GasAbstractionError::SponsorshipNotFound)
        }
    }

    /// Submit a transaction with gas abstraction
    pub fn submit_abstracted_transaction(
        &mut self,
        transaction: Transaction,
        user: TraderId,
        priority: u8,
    ) -> Result<String, GasAbstractionError> {
        let now = Self::now();

        if !self.availability_tracker.is_service_available(now) {
            return Err(GasAbstractionError::ServiceUnavailable);
        }

        // Check if user has sponsorship
        let sponsor = if let Some(sponsorship) = self.sponsorships.get(&user) {
            if sponsorship.is_active && self.check_sponsorship_limit(&user)? {
                Some(sponsorship.sponsor_id.clone())
            } else {
                None
            }
        } else {
            None
        };

        // Estimate gas cost
        let estimated_gas = self.estimate_gas(&transaction)?;

        // Select optimal relayer using AI router
        let relayer = self.ai_router.select_optimal_relayer(
            &self.relayers,
            estimated_gas,
            priority,
            &self.availability_tracker,
        );

        let abstracted_tx = AbstractedTransaction {
            transaction,
            user: user.clone(),
            sponsor,
            relayer: relayer.clone(),
            estimated_gas,
            priority,
            submitted_at: now,
            status: TransactionStatus::InQueue,
        };

        // Add to queue
        self.transaction_queue.push(abstracted_tx);

        // Update sponsorship usage if applicable
        if let Some(sponsor_id) = relayer {
            if let Some(sponsorship) = self.sponsorships.get_mut(&user) {
                sponsorship.used_gas += estimated_gas;
            }
        }

        // Return transaction ID (in a real implementation, this would be more sophisticated)
        Ok(format!("abs_tx_{}", now))
    }

    /// Process the transaction queue
    pub fn process_queue(&mut self) -> Result<Vec<String>, GasAbstractionError> {
        let now = Self::now();

        if !self.availability_tracker.is_service_available(now) {
            return Err(GasAbstractionError::ServiceUnavailable);
        }

        let mut processed_txs = Vec::new();

        // Sort transactions by priority (higher priority first)
        self.transaction_queue
            .sort_by(|a, b| b.priority.cmp(&a.priority));

        // Process transactions with available relayers
        self.transaction_queue.retain(|tx| {
            if tx.status == TransactionStatus::InQueue {
                // Try to submit with assigned relayer or find a new one
                let relayer_id = if let Some(rid) = &tx.relayer {
                    rid.clone()
                } else {
                    // If no relayer assigned, try to find one
                    if let Some(rid) = self.ai_router.select_optimal_relayer(
                        &self.relayers,
                        tx.estimated_gas,
                        tx.priority,
                        &self.availability_tracker,
                    ) {
                        rid
                    } else {
                        // No relayer available, keep in queue
                        return true;
                    }
                };

                // Check if relayer is available
                if let Some(relayer) = self.relayers.get_mut(&relayer_id) {
                    if relayer.is_active
                        && relayer.balance >= tx.estimated_gas
                        && relayer.current_tps < relayer.max_tps
                    {
                        // Submit transaction (simulated)
                        relayer.balance -= tx.estimated_gas;
                        relayer.current_tps += 1;

                        // Update transaction status
                        // In a real implementation, this would involve actual blockchain submission
                        processed_txs.push(format!("Processed transaction for user: {}", tx.user));
                        false // Remove from queue
                    } else {
                        true // Keep in queue
                    }
                } else {
                    true // Keep in queue
                }
            } else {
                false // Remove from queue
            }
        });

        Ok(processed_txs)
    }

    /// Estimate gas cost for a transaction
    pub fn estimate_gas(&self, transaction: &Transaction) -> Result<u64, GasAbstractionError> {
        // In a real implementation, this would use network data and transaction complexity
        // For now, we'll use a simple estimation based on transaction amount
        let base_gas = 21000u64; // Base cost for a simple transfer
        let amount_gas = (transaction.amount / 1000) as u64; // Additional gas based on amount
        Ok(base_gas + amount_gas)
    }

    /// Check if user is within sponsorship limits
    fn check_sponsorship_limit(&self, user: &TraderId) -> Result<bool, GasAbstractionError> {
        if let Some(sponsorship) = self.sponsorships.get(user) {
            let now = Self::now();

            // Reset daily limit if needed
            if now - sponsorship.last_reset > 86400 {
                // 24 hours
                // In a real implementation, this would be updated in the sponsorship
                return Ok(true);
            }

            Ok(sponsorship.used_gas < sponsorship.daily_limit)
        } else {
            Ok(false)
        }
    }

    /// Get transaction by ID
    pub fn get_transaction(&self, tx_id: &str) -> Option<&AbstractedTransaction> {
        // In a real implementation, we would have a proper ID system
        // For now, we'll just return the first transaction as an example
        self.transaction_queue.first()
    }

    /// Get queue size
    pub fn queue_size(&self) -> usize {
        self.transaction_queue.len()
    }

    /// Update network conditions for AI router
    pub fn update_network_conditions(&mut self, conditions: NetworkConditions) {
        self.ai_router.update_network_conditions(conditions);
    }

    /// Record execution for AI learning
    pub fn record_execution(&mut self, record: ExecutionRecord) {
        self.ai_router.record_execution(record);
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl Default for GasAbstractionService {
    fn default() -> Self {
        Self::new()
    }
}

impl AIRouter {
    /// Create a new AI router
    pub fn new() -> Self {
        Self {
            path_metrics: HashMap::new(),
            network_conditions: NetworkConditions {
                gas_price: 20,
                congestion: 30,
                block_time: 15,
                pending_transactions: 1000,
            },
            history: Vec::new(),
        }
    }

    /// Select the optimal relayer based on AI analysis
    pub fn select_optimal_relayer(
        &self,
        relayers: &HashMap<String, Relayer>,
        estimated_gas: u64,
        priority: u8,
        availability: &AvailabilityTracker,
    ) -> Option<String> {
        // Check if service is operational
        if availability.current_status != ServiceStatus::Operational
            && availability.current_status != ServiceStatus::Degraded
        {
            return None;
        }

        // Filter active relayers with sufficient balance
        let mut candidates: Vec<(&String, &Relayer)> = relayers
            .iter()
            .filter(|(_, relayer)| {
                relayer.is_active
                    && relayer.balance >= estimated_gas
                    && relayer.current_tps < relayer.max_tps
            })
            .collect();

        // If no candidates, return None
        if candidates.is_empty() {
            return None;
        }

        // Sort by reputation (higher reputation first)
        candidates.sort_by(|a, b| b.1.reputation.partial_cmp(&a.1.reputation).unwrap());

        // For high priority transactions, select the highest reputation relayer
        if priority > 7 {
            Some(candidates[0].0.clone())
        } else {
            // For lower priority, consider cost and performance
            // This is a simplified selection algorithm
            // In a real implementation, this would use machine learning
            Some(candidates[0].0.clone())
        }
    }

    /// Update network conditions
    pub fn update_network_conditions(&mut self, conditions: NetworkConditions) {
        self.network_conditions = conditions;
    }

    /// Record execution for learning
    pub fn record_execution(&mut self, record: ExecutionRecord) {
        self.history.push(record.clone());

        // Update path metrics
        let metrics = self
            .path_metrics
            .entry(record.path.clone())
            .or_insert_with(|| PathMetrics {
                avg_execution_time: 0,
                success_rate: 0.0,
                avg_cost: 0,
                last_updated: 0,
            });

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Update metrics (simplified averaging)
        metrics.avg_execution_time = (metrics.avg_execution_time + record.execution_time) / 2;
        metrics.avg_cost = (metrics.avg_cost + record.cost) / 2;
        metrics.success_rate =
            (metrics.success_rate + if record.success { 1.0 } else { 0.0 }) / 2.0;
        metrics.last_updated = now;
    }
}

impl Default for AIRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl AvailabilityTracker {
    /// Create a new availability tracker
    pub fn new() -> Self {
        Self::new_at(Self::now())
    }

    /// Create a tracker anchored at a custom start time (useful for tests)
    pub fn new_at(start_timestamp: u64) -> Self {
        let mut tracker = Self {
            uptime_history: Vec::new(),
            current_status: ServiceStatus::Operational,
            alert_thresholds: AlertThresholds {
                max_downtime: FIVE_NINES_MAX_DOWNTIME_SECS,
                response_time_threshold: 5000, // 5 seconds
                error_rate_threshold: 0.001,   // 0.1%
                window_secs: FIVE_NINES_WINDOW_SECS,
            },
        };

        tracker.uptime_history.push(AvailabilityRecord {
            timestamp: start_timestamp,
            status: ServiceStatus::Operational,
            incident: None,
            duration: 0,
        });
        tracker
    }

    /// Record service status
    pub fn record_status(&mut self, status: ServiceStatus, incident: Option<String>) {
        let now = Self::now();
        self.record_status_at(status, incident, now);
    }

    /// Record service status with explicit timestamp (test-friendly)
    pub fn record_status_at(
        &mut self,
        status: ServiceStatus,
        incident: Option<String>,
        timestamp: u64,
    ) {
        self.finalize_last_record(timestamp);

        self.uptime_history.push(AvailabilityRecord {
            timestamp,
            status: status.clone(),
            incident,
            duration: 0,
        });

        self.current_status = status;
        self.prune_history(timestamp);
    }

    /// Check if service meets uptime requirements
    pub fn check_uptime_requirements(&self) -> bool {
        self.check_uptime_requirements_at(Self::now())
    }

    /// Check uptime against five-nines SLO at a specific point in time
    pub fn check_uptime_requirements_at(&self, now: u64) -> bool {
        self.downtime_in_window(now) <= self.alert_thresholds.max_downtime
    }

    /// Whether the service is considered available for handling new work
    pub fn is_service_available(&self, now: u64) -> bool {
        matches!(
            self.current_status,
            ServiceStatus::Operational | ServiceStatus::Degraded
        ) && self.check_uptime_requirements_at(now)
    }

    /// Get current service status
    pub fn get_current_status(&self) -> &ServiceStatus {
        &self.current_status
    }

    fn downtime_in_window(&self, now: u64) -> u64 {
        let window_start = now.saturating_sub(self.alert_thresholds.window_secs);
        let mut downtime = 0u64;

        for record in &self.uptime_history {
            let record_start = record.timestamp;
            let record_end = if record.duration == 0 {
                now
            } else {
                record.timestamp.saturating_add(record.duration)
            };

            if record_end <= window_start || record_start >= now {
                continue;
            }

            let overlap_start = record_start.max(window_start);
            let overlap_end = record_end.min(now);
            let overlap = overlap_end.saturating_sub(overlap_start);

            if matches!(
                record.status,
                ServiceStatus::Outage | ServiceStatus::Maintenance
            ) {
                downtime = downtime.saturating_add(overlap);
            }
        }

        downtime
    }

    fn finalize_last_record(&mut self, now: u64) {
        if let Some(last) = self.uptime_history.last_mut() {
            last.duration = now.saturating_sub(last.timestamp);
        }
    }

    fn prune_history(&mut self, now: u64) {
        let window_start = now.saturating_sub(self.alert_thresholds.window_secs);
        self.uptime_history
            .retain(|record| record.timestamp >= window_start);
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl Default for AvailabilityTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during gas abstraction operations
#[derive(Debug, Error)]
pub enum GasAbstractionError {
    #[error("Relayer not found")]
    RelayerNotFound,
    #[error("Sponsorship not found")]
    SponsorshipNotFound,
    #[error("Insufficient sponsorship limit")]
    InsufficientSponsorshipLimit,
    #[error("Transaction estimation failed")]
    EstimationFailed,
    #[error("Service unavailable")]
    ServiceUnavailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_abstraction_service_creation() {
        let service = GasAbstractionService::new();
        assert!(service.relayers.is_empty());
        assert!(service.sponsorships.is_empty());
        assert!(service.transaction_queue.is_empty());
    }

    #[test]
    fn test_add_relayer() {
        let mut service = GasAbstractionService::new();
        let relayer = Relayer {
            id: "relayer1".to_string(),
            address: "0x1234...".to_string(),
            reputation: 0.95,
            balance: 1000000,
            max_tps: 100,
            current_tps: 0,
            is_active: true,
        };

        service.add_relayer(relayer.clone());
        assert_eq!(service.relayers.len(), 1);

        let stored_relayer = service.relayers.get("relayer1").unwrap();
        assert_eq!(stored_relayer.id, relayer.id);
        assert_eq!(stored_relayer.address, relayer.address);
        assert_eq!(stored_relayer.reputation, relayer.reputation);
        assert_eq!(stored_relayer.balance, relayer.balance);
        assert_eq!(stored_relayer.max_tps, relayer.max_tps);
        assert_eq!(stored_relayer.current_tps, relayer.current_tps);
        assert_eq!(stored_relayer.is_active, relayer.is_active);
    }

    #[test]
    fn test_remove_relayer() {
        let mut service = GasAbstractionService::new();
        let relayer = Relayer {
            id: "relayer1".to_string(),
            address: "0x1234...".to_string(),
            reputation: 0.95,
            balance: 1000000,
            max_tps: 100,
            current_tps: 0,
            is_active: true,
        };

        service.add_relayer(relayer);
        assert_eq!(service.relayers.len(), 1);

        assert!(service.remove_relayer("relayer1").is_ok());
        assert_eq!(service.relayers.len(), 0);

        // Try to remove non-existent relayer
        assert!(service.remove_relayer("nonexistent").is_err());
    }

    #[test]
    fn test_set_sponsorship() {
        let mut service = GasAbstractionService::new();
        let user = "user1".to_string();
        let sponsorship = GasSponsorship {
            sponsor_id: "dao".to_string(),
            daily_limit: 100000,
            used_gas: 0,
            last_reset: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_active: true,
        };

        service.set_sponsorship(user.clone(), sponsorship.clone());
        assert_eq!(service.sponsorships.len(), 1);

        let stored_sponsorship = service.sponsorships.get(&user).unwrap();
        assert_eq!(stored_sponsorship.sponsor_id, sponsorship.sponsor_id);
        assert_eq!(stored_sponsorship.daily_limit, sponsorship.daily_limit);
        assert_eq!(stored_sponsorship.used_gas, sponsorship.used_gas);
        assert_eq!(stored_sponsorship.last_reset, sponsorship.last_reset);
        assert_eq!(stored_sponsorship.is_active, sponsorship.is_active);
    }

    #[test]
    fn test_submit_abstracted_transaction() {
        let mut service = GasAbstractionService::new();
        let user = "user1".to_string();

        // Add a relayer
        let relayer = Relayer {
            id: "relayer1".to_string(),
            address: "0x1234...".to_string(),
            reputation: 0.95,
            balance: 1000000,
            max_tps: 100,
            current_tps: 0,
            is_active: true,
        };
        service.add_relayer(relayer);

        // Add sponsorship
        let sponsorship = GasSponsorship {
            sponsor_id: "dao".to_string(),
            daily_limit: 100000,
            used_gas: 0,
            last_reset: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_active: true,
        };
        service.set_sponsorship(user.clone(), sponsorship);

        // Create a transaction
        let transaction = Transaction {
            from: user.clone(),
            to: "recipient".to_string(),
            amount: 1000,
            nonce: 1,
            signature: vec![],
        };

        // Submit abstracted transaction
        let result = service.submit_abstracted_transaction(transaction, user.clone(), 5);
        assert!(result.is_ok());

        // Check that transaction was added to queue
        assert_eq!(service.queue_size(), 1);
    }

    #[test]
    fn test_process_queue() {
        let mut service = GasAbstractionService::new();
        let user = "user1".to_string();

        // Add a relayer
        let relayer = Relayer {
            id: "relayer1".to_string(),
            address: "0x1234...".to_string(),
            reputation: 0.95,
            balance: 1000000,
            max_tps: 100,
            current_tps: 0,
            is_active: true,
        };
        service.add_relayer(relayer);

        // Create a transaction
        let transaction = Transaction {
            from: user.clone(),
            to: "recipient".to_string(),
            amount: 1000,
            nonce: 1,
            signature: vec![],
        };

        // Submit abstracted transaction
        let tx_id = service.submit_abstracted_transaction(transaction, user.clone(), 5);
        assert!(tx_id.is_ok());
        assert_eq!(service.queue_size(), 1);

        // Process queue
        let result = service.process_queue();
        assert!(result.is_ok());

        // Queue should now be empty
        assert_eq!(service.queue_size(), 0);
    }

    #[test]
    fn test_estimate_gas() {
        let service = GasAbstractionService::new();

        let transaction = Transaction {
            from: "user1".to_string(),
            to: "recipient".to_string(),
            amount: 5000,
            nonce: 1,
            signature: vec![],
        };

        let estimated_gas = service.estimate_gas(&transaction);
        assert!(estimated_gas.is_ok());

        // Base gas (21000) + amount-based gas (5000/1000 = 5)
        assert_eq!(estimated_gas.unwrap(), 21005);
    }

    #[test]
    fn test_availability_tracker_enforces_five_nines_budget() {
        let mut tracker = AvailabilityTracker::new_at(0);

        // Two short incidents totaling 25 seconds (within 26s budget)
        tracker.record_status_at(
            ServiceStatus::Outage,
            Some("planned maintenance".to_string()),
            100,
        );
        tracker.record_status_at(ServiceStatus::Operational, None, 110);
        tracker.record_status_at(ServiceStatus::Outage, None, 200);
        tracker.record_status_at(ServiceStatus::Operational, None, 215);
        assert!(tracker.check_uptime_requirements_at(300));

        // A longer incident pushes downtime over the 26s budget
        tracker.record_status_at(ServiceStatus::Outage, None, 400);
        tracker.record_status_at(ServiceStatus::Operational, None, 450);
        assert!(!tracker.check_uptime_requirements_at(500));
    }

    #[test]
    fn test_gas_abstraction_rejects_when_unavailable() {
        let mut service = GasAbstractionService::new();
        service.availability_tracker = AvailabilityTracker::new_at(0);
        service
            .availability_tracker
            .record_status(ServiceStatus::Outage, Some("network outage".to_string()));

        let transaction = Transaction {
            from: "user1".to_string(),
            to: "recipient".to_string(),
            amount: 1000,
            nonce: 1,
            signature: vec![],
        };

        let result = service.submit_abstracted_transaction(transaction, "user1".to_string(), 5);
        assert!(matches!(
            result,
            Err(GasAbstractionError::ServiceUnavailable)
        ));
    }

    #[test]
    fn test_ai_router_selection() {
        let router = AIRouter::new();
        let mut relayers = HashMap::new();

        // Add relayers
        relayers.insert(
            "relayer1".to_string(),
            Relayer {
                id: "relayer1".to_string(),
                address: "0x1234...".to_string(),
                reputation: 0.95,
                balance: 1000000,
                max_tps: 100,
                current_tps: 0,
                is_active: true,
            },
        );

        relayers.insert(
            "relayer2".to_string(),
            Relayer {
                id: "relayer2".to_string(),
                address: "0x5678...".to_string(),
                reputation: 0.85,
                balance: 1000000,
                max_tps: 100,
                current_tps: 0,
                is_active: true,
            },
        );

        let availability = AvailabilityTracker::new();

        // Select relayer for normal priority transaction
        let selected = router.select_optimal_relayer(&relayers, 50000, 5, &availability);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap(), "relayer1"); // Higher reputation should be selected

        // Select relayer for high priority transaction
        let selected = router.select_optimal_relayer(&relayers, 50000, 9, &availability);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap(), "relayer1");
    }

    #[test]
    fn test_availability_tracker() {
        let mut tracker = AvailabilityTracker::new();

        // Check initial status
        assert_eq!(tracker.get_current_status(), &ServiceStatus::Operational);

        // Record an incident
        tracker.record_status(
            ServiceStatus::Degraded,
            Some("Network congestion".to_string()),
        );

        assert_eq!(tracker.get_current_status(), &ServiceStatus::Degraded);

        // Check uptime requirements (should pass with no downtime recorded)
        assert!(tracker.check_uptime_requirements());
    }
}
