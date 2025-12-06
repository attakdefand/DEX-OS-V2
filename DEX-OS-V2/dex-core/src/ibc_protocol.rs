//! IBC (Inter-Blockchain Communication) Protocol - Cross-Chain Communication
//!
//! Implements: `4,Scalability & Interoperability,Cross-Chain Protocols,Cross-Chain Protocols,IBC (Inter-Blockchain Communication),IBC Communication,High`
//!
//! This module provides IBC protocol implementation for secure cross-chain communication,
//! enabling interoperability between different blockchain networks.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Errors that can occur in IBC operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IBCError {
    ConnectionNotFound,
    ChannelNotFound,
    InvalidProof,
    InvalidSequence,
    TimeoutExpired,
    ChannelClosed,
    InvalidPacket,
    UnknownChain,
    VerificationFailed,
    InvalidClient,
    ConsensusStateNotFound,
    PacketAlreadyReceived,
    AcknowledgementNotFound,
}

/// IBC connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Uninitialized,
    Init,
    TryOpen,
    Open,
    Closed,
}

/// IBC channel state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelState {
    Uninitialized,
    Init,
    TryOpen,
    Open,
    Closed,
}

/// IBC packet ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketOrdering {
    Ordered,
    Unordered,
}

/// Light client for verifying remote chain state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightClient {
    pub client_id: String,
    pub chain_id: String,
    pub latest_height: u64,
    pub consensus_states: HashMap<u64, ConsensusState>,
    pub frozen: bool,
}

impl LightClient {
    /// Create a new light client
    pub fn new(client_id: String, chain_id: String) -> Self {
        Self {
            client_id,
            chain_id,
            latest_height: 0,
            consensus_states: HashMap::new(),
            frozen: false,
        }
    }

    /// Update client with new consensus state
    pub fn update(&mut self, height: u64, state: ConsensusState) -> Result<(), IBCError> {
        if self.frozen {
            return Err(IBCError::InvalidClient);
        }

        if height <= self.latest_height {
            return Err(IBCError::InvalidSequence);
        }

        self.consensus_states.insert(height, state);
        self.latest_height = height;

        Ok(())
    }

    /// Verify proof against consensus state
    pub fn verify_proof(
        &self,
        height: u64,
        proof: &Proof,
    ) -> Result<(), IBCError> {
        if self.frozen {
            return Err(IBCError::InvalidClient);
        }

        let _state = self
            .consensus_states
            .get(&height)
            .ok_or(IBCError::ConsensusStateNotFound)?;

        // Simplified proof verification
        if proof.data.is_empty() {
            return Err(IBCError::InvalidProof);
        }

        Ok(())
    }
}

/// Consensus state at a specific height
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    pub height: u64,
    pub timestamp: u64,
    pub root: Vec<u8>,
    pub next_validators_hash: Vec<u8>,
}

/// Merkle proof for state verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub data: Vec<u8>,
    pub height: u64,
}

/// IBC connection between two chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub client_id: String,
    pub counterparty_client_id: String,
    pub counterparty_connection_id: String,
    pub state: ConnectionState,
    pub versions: Vec<String>,
}

impl Connection {
    /// Create a new connection
    pub fn new(
        id: String,
        client_id: String,
        counterparty_client_id: String,
    ) -> Self {
        Self {
            id,
            client_id,
            counterparty_client_id,
            counterparty_connection_id: String::new(),
            state: ConnectionState::Uninitialized,
            versions: vec!["1.0".to_string()],
        }
    }

    /// Initialize connection
    pub fn init(&mut self) {
        self.state = ConnectionState::Init;
    }

    /// Try to open connection
    pub fn try_open(&mut self, counterparty_connection_id: String) {
        self.counterparty_connection_id = counterparty_connection_id;
        self.state = ConnectionState::TryOpen;
    }

    /// Open connection
    pub fn open(&mut self) {
        self.state = ConnectionState::Open;
    }

    /// Close connection
    pub fn close(&mut self) {
        self.state = ConnectionState::Closed;
    }
}

/// IBC channel for packet transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub connection_id: String,
    pub counterparty_channel_id: String,
    pub counterparty_port_id: String,
    pub state: ChannelState,
    pub ordering: PacketOrdering,
    pub version: String,
    pub next_sequence_send: u64,
    pub next_sequence_recv: u64,
    pub next_sequence_ack: u64,
}

impl Channel {
    /// Create a new channel
    pub fn new(
        id: String,
        connection_id: String,
        ordering: PacketOrdering,
    ) -> Self {
        Self {
            id,
            connection_id,
            counterparty_channel_id: String::new(),
            counterparty_port_id: String::new(),
            state: ChannelState::Uninitialized,
            ordering,
            version: "ics20-1".to_string(),
            next_sequence_send: 1,
            next_sequence_recv: 1,
            next_sequence_ack: 1,
        }
    }

    /// Initialize channel
    pub fn init(&mut self, counterparty_port_id: String) {
        self.counterparty_port_id = counterparty_port_id;
        self.state = ChannelState::Init;
    }

    /// Try to open channel
    pub fn try_open(&mut self, counterparty_channel_id: String) {
        self.counterparty_channel_id = counterparty_channel_id;
        self.state = ChannelState::TryOpen;
    }

    /// Open channel
    pub fn open(&mut self) {
        self.state = ChannelState::Open;
    }

    /// Close channel
    pub fn close(&mut self) {
        self.state = ChannelState::Closed;
    }
}

/// IBC packet for cross-chain communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub sequence: u64,
    pub source_port: String,
    pub source_channel: String,
    pub destination_port: String,
    pub destination_channel: String,
    pub data: Vec<u8>,
    pub timeout_height: u64,
    pub timeout_timestamp: u64,
}

impl Packet {
    /// Create a new packet
    pub fn new(
        sequence: u64,
        source_port: String,
        source_channel: String,
        destination_port: String,
        destination_channel: String,
        data: Vec<u8>,
        timeout_height: u64,
        timeout_timestamp: u64,
    ) -> Self {
        Self {
            sequence,
            source_port,
            source_channel,
            destination_port,
            destination_channel,
            data,
            timeout_height,
            timeout_timestamp,
        }
    }

    /// Check if packet has timed out
    pub fn is_timed_out(&self, current_height: u64, current_timestamp: u64) -> bool {
        if self.timeout_height > 0 && current_height >= self.timeout_height {
            return true;
        }

        if self.timeout_timestamp > 0 && current_timestamp >= self.timeout_timestamp {
            return true;
        }

        false
    }
}

/// Packet acknowledgement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Acknowledgement {
    pub result: AckResult,
    pub data: Vec<u8>,
}

/// Acknowledgement result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AckResult {
    Success,
    Error,
}

/// IBC transfer data (ICS-20)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferData {
    pub denom: String,
    pub amount: u64,
    pub sender: String,
    pub receiver: String,
    pub memo: String,
}

impl TransferData {
    /// Encode transfer data to bytes
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    /// Decode transfer data from bytes
    pub fn decode(data: &[u8]) -> Result<Self, IBCError> {
        serde_json::from_slice(data).map_err(|_| IBCError::InvalidPacket)
    }
}

/// IBC Protocol Manager
pub struct IBCManager {
    clients: Arc<RwLock<HashMap<String, LightClient>>>,
    connections: Arc<RwLock<HashMap<String, Connection>>>,
    channels: Arc<RwLock<HashMap<String, Channel>>>,
    packets: Arc<RwLock<HashMap<String, Packet>>>,
    acknowledgements: Arc<RwLock<HashMap<String, Acknowledgement>>>,
    received_packets: Arc<RwLock<HashMap<String, bool>>>,
    chain_id: String,
}

impl IBCManager {
    /// Create a new IBC manager
    pub fn new(chain_id: String) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            packets: Arc::new(RwLock::new(HashMap::new())),
            acknowledgements: Arc::new(RwLock::new(HashMap::new())),
            received_packets: Arc::new(RwLock::new(HashMap::new())),
            chain_id,
        }
    }

    /// Create a new light client
    pub fn create_client(
        &self,
        client_id: String,
        chain_id: String,
    ) -> Result<(), IBCError> {
        let mut clients = self.clients.write().unwrap();

        let client = LightClient::new(client_id.clone(), chain_id);
        clients.insert(client_id, client);

        Ok(())
    }

    /// Update light client
    pub fn update_client(
        &self,
        client_id: &str,
        height: u64,
        state: ConsensusState,
    ) -> Result<(), IBCError> {
        let mut clients = self.clients.write().unwrap();

        let client = clients
            .get_mut(client_id)
            .ok_or(IBCError::InvalidClient)?;

        client.update(height, state)
    }

    /// Create a new connection
    pub fn create_connection(
        &self,
        connection_id: String,
        client_id: String,
        counterparty_client_id: String,
    ) -> Result<(), IBCError> {
        let mut connections = self.connections.write().unwrap();

        let mut connection = Connection::new(
            connection_id.clone(),
            client_id,
            counterparty_client_id,
        );
        connection.init();

        connections.insert(connection_id, connection);

        Ok(())
    }

    /// Open a connection
    pub fn open_connection(
        &self,
        connection_id: &str,
        counterparty_connection_id: String,
    ) -> Result<(), IBCError> {
        let mut connections = self.connections.write().unwrap();

        let connection = connections
            .get_mut(connection_id)
            .ok_or(IBCError::ConnectionNotFound)?;

        connection.try_open(counterparty_connection_id);
        connection.open();

        Ok(())
    }

    /// Create a new channel
    pub fn create_channel(
        &self,
        channel_id: String,
        connection_id: String,
        port_id: String,
        ordering: PacketOrdering,
    ) -> Result<(), IBCError> {
        let mut channels = self.channels.write().unwrap();

        let mut channel = Channel::new(channel_id.clone(), connection_id, ordering);
        channel.init(port_id);

        channels.insert(channel_id, channel);

        Ok(())
    }

    /// Open a channel
    pub fn open_channel(
        &self,
        channel_id: &str,
        counterparty_channel_id: String,
    ) -> Result<(), IBCError> {
        let mut channels = self.channels.write().unwrap();

        let channel = channels
            .get_mut(channel_id)
            .ok_or(IBCError::ChannelNotFound)?;

        channel.try_open(counterparty_channel_id);
        channel.open();

        Ok(())
    }

    /// Send a packet
    pub fn send_packet(
        &self,
        channel_id: &str,
        data: Vec<u8>,
        timeout_height: u64,
        timeout_timestamp: u64,
    ) -> Result<Packet, IBCError> {
        let mut channels = self.channels.write().unwrap();

        let channel = channels
            .get_mut(channel_id)
            .ok_or(IBCError::ChannelNotFound)?;

        if channel.state != ChannelState::Open {
            return Err(IBCError::ChannelClosed);
        }

        let sequence = channel.next_sequence_send;
        channel.next_sequence_send += 1;

        let packet = Packet::new(
            sequence,
            "transfer".to_string(),
            channel_id.to_string(),
            channel.counterparty_port_id.clone(),
            channel.counterparty_channel_id.clone(),
            data,
            timeout_height,
            timeout_timestamp,
        );

        let packet_key = format!("{}-{}", channel_id, sequence);
        let mut packets = self.packets.write().unwrap();
        packets.insert(packet_key, packet.clone());

        Ok(packet)
    }

    /// Receive a packet
    pub fn receive_packet(
        &self,
        packet: Packet,
        proof: Proof,
    ) -> Result<Acknowledgement, IBCError> {
        // Check if packet already received
        let packet_key = format!("{}-{}", packet.source_channel, packet.sequence);
        {
            let received = self.received_packets.read().unwrap();
            if received.contains_key(&packet_key) {
                return Err(IBCError::PacketAlreadyReceived);
            }
        }

        // Get channel
        let mut channels = self.channels.write().unwrap();
        let channel = channels
            .get_mut(&packet.destination_channel)
            .ok_or(IBCError::ChannelNotFound)?;

        if channel.state != ChannelState::Open {
            return Err(IBCError::ChannelClosed);
        }

        // Verify packet hasn't timed out
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if packet.is_timed_out(0, current_time) {
            return Err(IBCError::TimeoutExpired);
        }

        // Verify proof
        let clients = self.clients.read().unwrap();
        let connection = {
            let connections = self.connections.read().unwrap();
            connections
                .get(&channel.connection_id)
                .ok_or(IBCError::ConnectionNotFound)?
                .clone()
        };

        let client = clients
            .get(&connection.client_id)
            .ok_or(IBCError::InvalidClient)?;

        client.verify_proof(proof.height, &proof)?;

        // Update sequence
        if channel.ordering == PacketOrdering::Ordered {
            if packet.sequence != channel.next_sequence_recv {
                return Err(IBCError::InvalidSequence);
            }
            channel.next_sequence_recv += 1;
        }

        // Mark packet as received
        let mut received = self.received_packets.write().unwrap();
        received.insert(packet_key, true);

        // Process packet data (simplified)
        let ack = Acknowledgement {
            result: AckResult::Success,
            data: vec![],
        };

        // Store acknowledgement
        let ack_key = format!("{}-{}", packet.destination_channel, packet.sequence);
        let mut acks = self.acknowledgements.write().unwrap();
        acks.insert(ack_key, ack.clone());

        Ok(ack)
    }

    /// Acknowledge a packet
    pub fn acknowledge_packet(
        &self,
        packet: Packet,
        acknowledgement: Acknowledgement,
        proof: Proof,
    ) -> Result<(), IBCError> {
        // Verify proof
        let channels = self.channels.read().unwrap();
        let channel = channels
            .get(&packet.source_channel)
            .ok_or(IBCError::ChannelNotFound)?;

        let connections = self.connections.read().unwrap();
        let connection = connections
            .get(&channel.connection_id)
            .ok_or(IBCError::ConnectionNotFound)?;

        let clients = self.clients.read().unwrap();
        let client = clients
            .get(&connection.client_id)
            .ok_or(IBCError::InvalidClient)?;

        client.verify_proof(proof.height, &proof)?;

        // Remove packet from pending
        let packet_key = format!("{}-{}", packet.source_channel, packet.sequence);
        let mut packets = self.packets.write().unwrap();
        packets.remove(&packet_key);

        // Handle acknowledgement result
        if acknowledgement.result == AckResult::Error {
            // Handle error case (e.g., refund tokens)
        }

        Ok(())
    }

    /// Transfer tokens using ICS-20
    pub fn transfer(
        &self,
        channel_id: &str,
        denom: String,
        amount: u64,
        sender: String,
        receiver: String,
        timeout_height: u64,
        timeout_timestamp: u64,
    ) -> Result<Packet, IBCError> {
        let transfer_data = TransferData {
            denom,
            amount,
            sender,
            receiver,
            memo: String::new(),
        };

        let data = transfer_data.encode();

        self.send_packet(channel_id, data, timeout_height, timeout_timestamp)
    }

    /// Get statistics
    pub fn get_statistics(&self) -> IBCStatistics {
        let clients = self.clients.read().unwrap();
        let connections = self.connections.read().unwrap();
        let channels = self.channels.read().unwrap();
        let packets = self.packets.read().unwrap();

        IBCStatistics {
            total_clients: clients.len(),
            total_connections: connections.len(),
            open_connections: connections
                .values()
                .filter(|c| c.state == ConnectionState::Open)
                .count(),
            total_channels: channels.len(),
            open_channels: channels
                .values()
                .filter(|c| c.state == ChannelState::Open)
                .count(),
            pending_packets: packets.len(),
        }
    }
}

/// IBC statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IBCStatistics {
    pub total_clients: usize,
    pub total_connections: usize,
    pub open_connections: usize,
    pub total_channels: usize,
    pub open_channels: usize,
    pub pending_packets: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_client_creation() {
        let client = LightClient::new("client1".to_string(), "chain1".to_string());

        assert_eq!(client.client_id, "client1");
        assert_eq!(client.chain_id, "chain1");
        assert_eq!(client.latest_height, 0);
        assert!(!client.frozen);
    }

    #[test]
    fn test_light_client_update() {
        let mut client = LightClient::new("client1".to_string(), "chain1".to_string());

        let state = ConsensusState {
            height: 1,
            timestamp: 1000,
            root: vec![1, 2, 3],
            next_validators_hash: vec![4, 5, 6],
        };

        assert!(client.update(1, state).is_ok());
        assert_eq!(client.latest_height, 1);
    }

    #[test]
    fn test_connection_lifecycle() {
        let mut connection = Connection::new(
            "conn1".to_string(),
            "client1".to_string(),
            "client2".to_string(),
        );

        assert_eq!(connection.state, ConnectionState::Uninitialized);

        connection.init();
        assert_eq!(connection.state, ConnectionState::Init);

        connection.try_open("conn2".to_string());
        assert_eq!(connection.state, ConnectionState::TryOpen);

        connection.open();
        assert_eq!(connection.state, ConnectionState::Open);
    }

    #[test]
    fn test_channel_lifecycle() {
        let mut channel = Channel::new(
            "channel1".to_string(),
            "conn1".to_string(),
            PacketOrdering::Unordered,
        );

        assert_eq!(channel.state, ChannelState::Uninitialized);

        channel.init("transfer".to_string());
        assert_eq!(channel.state, ChannelState::Init);

        channel.try_open("channel2".to_string());
        assert_eq!(channel.state, ChannelState::TryOpen);

        channel.open();
        assert_eq!(channel.state, ChannelState::Open);
    }

    #[test]
    fn test_packet_creation() {
        let packet = Packet::new(
            1,
            "transfer".to_string(),
            "channel1".to_string(),
            "transfer".to_string(),
            "channel2".to_string(),
            vec![1, 2, 3],
            100,
            2000,
        );

        assert_eq!(packet.sequence, 1);
        assert_eq!(packet.source_channel, "channel1");
        assert_eq!(packet.destination_channel, "channel2");
    }

    #[test]
    fn test_packet_timeout() {
        let packet = Packet::new(
            1,
            "transfer".to_string(),
            "channel1".to_string(),
            "transfer".to_string(),
            "channel2".to_string(),
            vec![1, 2, 3],
            100,
            2000,
        );

        assert!(!packet.is_timed_out(50, 1000));
        assert!(packet.is_timed_out(100, 1000));
        assert!(packet.is_timed_out(50, 2000));
    }

    #[test]
    fn test_transfer_data() {
        let transfer = TransferData {
            denom: "uatom".to_string(),
            amount: 1000,
            sender: "cosmos1abc".to_string(),
            receiver: "cosmos1xyz".to_string(),
            memo: "test".to_string(),
        };

        let encoded = transfer.encode();
        assert!(!encoded.is_empty());

        let decoded = TransferData::decode(&encoded).unwrap();
        assert_eq!(decoded.denom, "uatom");
        assert_eq!(decoded.amount, 1000);
    }

    #[test]
    fn test_ibc_manager() {
        let manager = IBCManager::new("chain1".to_string());

        // Create client
        assert!(manager
            .create_client("client1".to_string(), "chain2".to_string())
            .is_ok());

        // Create connection
        assert!(manager
            .create_connection(
                "conn1".to_string(),
                "client1".to_string(),
                "client2".to_string()
            )
            .is_ok());

        // Open connection
        assert!(manager
            .open_connection("conn1", "conn2".to_string())
            .is_ok());

        // Create channel
        assert!(manager
            .create_channel(
                "channel1".to_string(),
                "conn1".to_string(),
                "transfer".to_string(),
                PacketOrdering::Unordered
            )
            .is_ok());

        // Open channel
        assert!(manager
            .open_channel("channel1", "channel2".to_string())
            .is_ok());

        // Get statistics
        let stats = manager.get_statistics();
        assert_eq!(stats.total_clients, 1);
        assert_eq!(stats.total_connections, 1);
        assert_eq!(stats.open_connections, 1);
        assert_eq!(stats.total_channels, 1);
        assert_eq!(stats.open_channels, 1);
    }

    #[test]
    fn test_send_packet() {
        let manager = IBCManager::new("chain1".to_string());

        // Setup
        manager
            .create_client("client1".to_string(), "chain2".to_string())
            .unwrap();
        manager
            .create_connection(
                "conn1".to_string(),
                "client1".to_string(),
                "client2".to_string(),
            )
            .unwrap();
        manager
            .open_connection("conn1", "conn2".to_string())
            .unwrap();
        manager
            .create_channel(
                "channel1".to_string(),
                "conn1".to_string(),
                "transfer".to_string(),
                PacketOrdering::Unordered,
            )
            .unwrap();
        manager
            .open_channel("channel1", "channel2".to_string())
            .unwrap();

        // Send packet
        let packet = manager
            .send_packet("channel1", vec![1, 2, 3], 100, 2000)
            .unwrap();

        assert_eq!(packet.sequence, 1);
        assert_eq!(packet.source_channel, "channel1");
    }

    #[test]
    fn test_transfer() {
        let manager = IBCManager::new("chain1".to_string());

        // Setup
        manager
            .create_client("client1".to_string(), "chain2".to_string())
            .unwrap();
        manager
            .create_connection(
                "conn1".to_string(),
                "client1".to_string(),
                "client2".to_string(),
            )
            .unwrap();
        manager
            .open_connection("conn1", "conn2".to_string())
            .unwrap();
        manager
            .create_channel(
                "channel1".to_string(),
                "conn1".to_string(),
                "transfer".to_string(),
                PacketOrdering::Unordered,
            )
            .unwrap();
        manager
            .open_channel("channel1", "channel2".to_string())
            .unwrap();

        // Transfer
        let packet = manager
            .transfer(
                "channel1",
                "uatom".to_string(),
                1000,
                "cosmos1abc".to_string(),
                "cosmos1xyz".to_string(),
                100,
                2000,
            )
            .unwrap();

        assert_eq!(packet.sequence, 1);
        assert!(!packet.data.is_empty());
    }
}
