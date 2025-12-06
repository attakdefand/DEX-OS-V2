//! State Channels for Off-chain Orders - Layer 2 Scaling
//!
//! Implements: `4,Scalability & Interoperability,Layer 2 Scaling,Layer 2 Scaling,State Channels for Off-chain Orders,State Channels,High`
//!
//! This module provides state channel functionality for off-chain order processing,
//! enabling high-throughput trading with minimal on-chain settlement.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Errors that can occur in state channel operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateChannelError {
    ChannelNotFound,
    ChannelAlreadyExists,
    ChannelClosed,
    InvalidSignature,
    InsufficientBalance,
    InvalidState,
    InvalidNonce,
    ParticipantNotFound,
    DisputeTimeout,
    InvalidUpdate,
}

/// State channel status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelStatus {
    Opening,
    Active,
    Disputed,
    Closing,
    Closed,
}

/// Participant in a state channel
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Participant {
    pub address: String,
    pub public_key: Vec<u8>,
}

/// State channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub challenge_period: u64,      // Time in seconds for dispute resolution
    pub max_pending_updates: usize, // Maximum pending state updates
    pub settlement_timeout: u64,    // Timeout for final settlement
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            challenge_period: 86400,    // 24 hours
            max_pending_updates: 1000,
            settlement_timeout: 604800, // 7 days
        }
    }
}

/// Off-chain order in a state channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffChainOrder {
    pub id: String,
    pub channel_id: String,
    pub maker: String,
    pub taker: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub amount: u64,
    pub price: u64,
    pub nonce: u64,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

/// State update in a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdate {
    pub nonce: u64,
    pub balances: HashMap<String, u64>,
    pub orders: Vec<OffChainOrder>,
    pub timestamp: u64,
    pub signatures: HashMap<String, Vec<u8>>,
}

/// State channel between participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChannel {
    pub id: String,
    pub participants: Vec<Participant>,
    pub status: ChannelStatus,
    pub config: ChannelConfig,
    pub current_state: StateUpdate,
    pub state_history: VecDeque<StateUpdate>,
    pub deposits: HashMap<String, u64>,
    pub opened_at: u64,
    pub closed_at: Option<u64>,
    pub dispute_deadline: Option<u64>,
}

impl StateChannel {
    /// Create a new state channel
    pub fn new(
        id: String,
        participants: Vec<Participant>,
        config: ChannelConfig,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let initial_state = StateUpdate {
            nonce: 0,
            balances: HashMap::new(),
            orders: Vec::new(),
            timestamp,
            signatures: HashMap::new(),
        };

        Self {
            id,
            participants,
            status: ChannelStatus::Opening,
            config,
            current_state: initial_state,
            state_history: VecDeque::new(),
            deposits: HashMap::new(),
            opened_at: timestamp,
            closed_at: None,
            dispute_deadline: None,
        }
    }

    /// Deposit funds into the channel
    pub fn deposit(&mut self, participant: &str, amount: u64) -> Result<(), StateChannelError> {
        if self.status == ChannelStatus::Closed {
            return Err(StateChannelError::ChannelClosed);
        }

        *self.deposits.entry(participant.to_string()).or_insert(0) += amount;
        *self.current_state.balances.entry(participant.to_string()).or_insert(0) += amount;

        Ok(())
    }

    /// Update channel state with a new state update
    pub fn update_state(&mut self, update: StateUpdate) -> Result<(), StateChannelError> {
        if self.status != ChannelStatus::Active {
            return Err(StateChannelError::InvalidState);
        }

        // Validate nonce
        if update.nonce <= self.current_state.nonce {
            return Err(StateChannelError::InvalidNonce);
        }

        // Validate signatures from all participants
        if update.signatures.len() != self.participants.len() {
            return Err(StateChannelError::InvalidSignature);
        }

        // Store previous state in history
        if self.state_history.len() >= self.config.max_pending_updates {
            self.state_history.pop_front();
        }
        self.state_history.push_back(self.current_state.clone());

        // Update current state
        self.current_state = update;

        Ok(())
    }

    /// Submit an off-chain order
    pub fn submit_order(&mut self, order: OffChainOrder) -> Result<(), StateChannelError> {
        if self.status != ChannelStatus::Active {
            return Err(StateChannelError::InvalidState);
        }

        // Verify order belongs to this channel
        if order.channel_id != self.id {
            return Err(StateChannelError::InvalidUpdate);
        }

        // Verify maker has sufficient balance
        let maker_balance = self.current_state.balances.get(&order.maker).unwrap_or(&0);
        if *maker_balance < order.amount {
            return Err(StateChannelError::InsufficientBalance);
        }

        // Add order to current state
        self.current_state.orders.push(order);
        self.current_state.nonce += 1;

        Ok(())
    }

    /// Initiate channel closure
    pub fn close(&mut self) -> Result<(), StateChannelError> {
        if self.status == ChannelStatus::Closed {
            return Err(StateChannelError::ChannelClosed);
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.status = ChannelStatus::Closing;
        self.dispute_deadline = Some(timestamp + self.config.challenge_period);

        Ok(())
    }

    /// Finalize channel closure
    pub fn finalize_close(&mut self) -> Result<HashMap<String, u64>, StateChannelError> {
        if self.status != ChannelStatus::Closing {
            return Err(StateChannelError::InvalidState);
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check if dispute period has passed
        if let Some(deadline) = self.dispute_deadline {
            if timestamp < deadline {
                return Err(StateChannelError::DisputeTimeout);
            }
        }

        self.status = ChannelStatus::Closed;
        self.closed_at = Some(timestamp);

        // Return final balances for settlement
        Ok(self.current_state.balances.clone())
    }

    /// Raise a dispute with a different state
    pub fn dispute(&mut self, disputed_state: StateUpdate) -> Result<(), StateChannelError> {
        if self.status != ChannelStatus::Closing {
            return Err(StateChannelError::InvalidState);
        }

        // Validate disputed state has higher nonce
        if disputed_state.nonce <= self.current_state.nonce {
            return Err(StateChannelError::InvalidNonce);
        }

        // Validate signatures
        if disputed_state.signatures.len() != self.participants.len() {
            return Err(StateChannelError::InvalidSignature);
        }

        self.status = ChannelStatus::Disputed;
        self.current_state = disputed_state;

        // Extend dispute deadline
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.dispute_deadline = Some(timestamp + self.config.challenge_period);

        Ok(())
    }
}

/// State Channel Manager
pub struct StateChannelManager {
    channels: Arc<RwLock<HashMap<String, StateChannel>>>,
    default_config: ChannelConfig,
}

impl StateChannelManager {
    /// Create a new state channel manager
    pub fn new(config: ChannelConfig) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            default_config: config,
        }
    }

    /// Open a new state channel
    pub fn open_channel(
        &self,
        channel_id: String,
        participants: Vec<Participant>,
    ) -> Result<(), StateChannelError> {
        let mut channels = self.channels.write().unwrap();

        if channels.contains_key(&channel_id) {
            return Err(StateChannelError::ChannelAlreadyExists);
        }

        let channel = StateChannel::new(
            channel_id.clone(),
            participants,
            self.default_config.clone(),
        );

        channels.insert(channel_id, channel);
        Ok(())
    }

    /// Activate a channel after deposits
    pub fn activate_channel(&self, channel_id: &str) -> Result<(), StateChannelError> {
        let mut channels = self.channels.write().unwrap();

        let channel = channels
            .get_mut(channel_id)
            .ok_or(StateChannelError::ChannelNotFound)?;

        if channel.status != ChannelStatus::Opening {
            return Err(StateChannelError::InvalidState);
        }

        channel.status = ChannelStatus::Active;
        Ok(())
    }

    /// Deposit funds into a channel
    pub fn deposit(
        &self,
        channel_id: &str,
        participant: &str,
        amount: u64,
    ) -> Result<(), StateChannelError> {
        let mut channels = self.channels.write().unwrap();

        let channel = channels
            .get_mut(channel_id)
            .ok_or(StateChannelError::ChannelNotFound)?;

        channel.deposit(participant, amount)
    }

    /// Submit an off-chain order
    pub fn submit_order(
        &self,
        channel_id: &str,
        order: OffChainOrder,
    ) -> Result<(), StateChannelError> {
        let mut channels = self.channels.write().unwrap();

        let channel = channels
            .get_mut(channel_id)
            .ok_or(StateChannelError::ChannelNotFound)?;

        channel.submit_order(order)
    }

    /// Update channel state
    pub fn update_state(
        &self,
        channel_id: &str,
        update: StateUpdate,
    ) -> Result<(), StateChannelError> {
        let mut channels = self.channels.write().unwrap();

        let channel = channels
            .get_mut(channel_id)
            .ok_or(StateChannelError::ChannelNotFound)?;

        channel.update_state(update)
    }

    /// Close a channel
    pub fn close_channel(&self, channel_id: &str) -> Result<(), StateChannelError> {
        let mut channels = self.channels.write().unwrap();

        let channel = channels
            .get_mut(channel_id)
            .ok_or(StateChannelError::ChannelNotFound)?;

        channel.close()
    }

    /// Finalize channel closure
    pub fn finalize_close(
        &self,
        channel_id: &str,
    ) -> Result<HashMap<String, u64>, StateChannelError> {
        let mut channels = self.channels.write().unwrap();

        let channel = channels
            .get_mut(channel_id)
            .ok_or(StateChannelError::ChannelNotFound)?;

        channel.finalize_close()
    }

    /// Dispute a channel state
    pub fn dispute(
        &self,
        channel_id: &str,
        disputed_state: StateUpdate,
    ) -> Result<(), StateChannelError> {
        let mut channels = self.channels.write().unwrap();

        let channel = channels
            .get_mut(channel_id)
            .ok_or(StateChannelError::ChannelNotFound)?;

        channel.dispute(disputed_state)
    }

    /// Get channel information
    pub fn get_channel(&self, channel_id: &str) -> Result<StateChannel, StateChannelError> {
        let channels = self.channels.read().unwrap();

        channels
            .get(channel_id)
            .cloned()
            .ok_or(StateChannelError::ChannelNotFound)
    }

    /// Get all active channels
    pub fn get_active_channels(&self) -> Vec<StateChannel> {
        let channels = self.channels.read().unwrap();

        channels
            .values()
            .filter(|c| c.status == ChannelStatus::Active)
            .cloned()
            .collect()
    }

    /// Get channel statistics
    pub fn get_statistics(&self) -> ChannelStatistics {
        let channels = self.channels.read().unwrap();

        let total_channels = channels.len();
        let active_channels = channels
            .values()
            .filter(|c| c.status == ChannelStatus::Active)
            .count();
        let total_orders: usize = channels
            .values()
            .map(|c| c.current_state.orders.len())
            .sum();
        let total_volume: u64 = channels
            .values()
            .flat_map(|c| c.current_state.balances.values())
            .sum();

        ChannelStatistics {
            total_channels,
            active_channels,
            total_orders,
            total_volume,
        }
    }
}

/// Statistics for state channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStatistics {
    pub total_channels: usize,
    pub active_channels: usize,
    pub total_orders: usize,
    pub total_volume: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_participants() -> Vec<Participant> {
        vec![
            Participant {
                address: "alice".to_string(),
                public_key: vec![1, 2, 3],
            },
            Participant {
                address: "bob".to_string(),
                public_key: vec![4, 5, 6],
            },
        ]
    }

    #[test]
    fn test_channel_creation() {
        let config = ChannelConfig::default();
        let participants = create_test_participants();
        let channel = StateChannel::new("channel1".to_string(), participants.clone(), config);

        assert_eq!(channel.id, "channel1");
        assert_eq!(channel.participants.len(), 2);
        assert_eq!(channel.status, ChannelStatus::Opening);
    }

    #[test]
    fn test_deposit() {
        let config = ChannelConfig::default();
        let participants = create_test_participants();
        let mut channel = StateChannel::new("channel1".to_string(), participants, config);

        assert!(channel.deposit("alice", 1000).is_ok());
        assert_eq!(channel.deposits.get("alice"), Some(&1000));
        assert_eq!(channel.current_state.balances.get("alice"), Some(&1000));
    }

    #[test]
    fn test_state_update() {
        let config = ChannelConfig::default();
        let participants = create_test_participants();
        let mut channel = StateChannel::new("channel1".to_string(), participants.clone(), config);

        channel.status = ChannelStatus::Active;

        let mut balances = HashMap::new();
        balances.insert("alice".to_string(), 900);
        balances.insert("bob".to_string(), 100);

        let mut signatures = HashMap::new();
        signatures.insert("alice".to_string(), vec![1, 2, 3]);
        signatures.insert("bob".to_string(), vec![4, 5, 6]);

        let update = StateUpdate {
            nonce: 1,
            balances,
            orders: Vec::new(),
            timestamp: 1000,
            signatures,
        };

        assert!(channel.update_state(update).is_ok());
        assert_eq!(channel.current_state.nonce, 1);
    }

    #[test]
    fn test_off_chain_order() {
        let config = ChannelConfig::default();
        let participants = create_test_participants();
        let mut channel = StateChannel::new("channel1".to_string(), participants, config);

        channel.status = ChannelStatus::Active;
        channel.deposit("alice", 1000).unwrap();

        let order = OffChainOrder {
            id: "order1".to_string(),
            channel_id: "channel1".to_string(),
            maker: "alice".to_string(),
            taker: "bob".to_string(),
            base_asset: "ETH".to_string(),
            quote_asset: "USDT".to_string(),
            amount: 100,
            price: 2000,
            nonce: 1,
            timestamp: 1000,
            signature: vec![1, 2, 3],
        };

        assert!(channel.submit_order(order).is_ok());
        assert_eq!(channel.current_state.orders.len(), 1);
    }

    #[test]
    fn test_channel_closure() {
        let config = ChannelConfig::default();
        let participants = create_test_participants();
        let mut channel = StateChannel::new("channel1".to_string(), participants, config);

        channel.status = ChannelStatus::Active;

        assert!(channel.close().is_ok());
        assert_eq!(channel.status, ChannelStatus::Closing);
        assert!(channel.dispute_deadline.is_some());
    }

    #[test]
    fn test_manager_operations() {
        let config = ChannelConfig::default();
        let manager = StateChannelManager::new(config);
        let participants = create_test_participants();

        // Open channel
        assert!(manager
            .open_channel("channel1".to_string(), participants)
            .is_ok());

        // Deposit
        assert!(manager.deposit("channel1", "alice", 1000).is_ok());

        // Activate
        assert!(manager.activate_channel("channel1").is_ok());

        // Get channel
        let channel = manager.get_channel("channel1").unwrap();
        assert_eq!(channel.status, ChannelStatus::Active);

        // Get statistics
        let stats = manager.get_statistics();
        assert_eq!(stats.total_channels, 1);
        assert_eq!(stats.active_channels, 1);
    }

    #[test]
    fn test_insufficient_balance() {
        let config = ChannelConfig::default();
        let participants = create_test_participants();
        let mut channel = StateChannel::new("channel1".to_string(), participants, config);

        channel.status = ChannelStatus::Active;
        channel.deposit("alice", 50).unwrap();

        let order = OffChainOrder {
            id: "order1".to_string(),
            channel_id: "channel1".to_string(),
            maker: "alice".to_string(),
            taker: "bob".to_string(),
            base_asset: "ETH".to_string(),
            quote_asset: "USDT".to_string(),
            amount: 100,
            price: 2000,
            nonce: 1,
            timestamp: 1000,
            signature: vec![1, 2, 3],
        };

        assert_eq!(
            channel.submit_order(order),
            Err(StateChannelError::InsufficientBalance)
        );
    }
}
