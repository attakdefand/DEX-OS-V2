//! Raft consensus implementation for service coordination and leader election
//!
//! This module implements the Raft consensus algorithm for coordinating services
//! in the DEX-OS infrastructure, covering the Priority 3 distributed-systems feature:
//! - Distributed Systems,Distributed Systems,Distributed Systems,Raft Leader Election,Leader Selection,Medium
//! - Infrastructure,Network,Network,Raft Consensus,Service Coordination,Medium

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::time::sleep;

/// Raft node states
#[derive(Debug, Clone, PartialEq)]
pub enum NodeState {
    Follower,
    Candidate,
    Leader,
}

/// Raft node configuration
#[derive(Debug, Clone)]
pub struct RaftConfig {
    /// Node ID
    pub node_id: String,
    /// List of all node addresses
    pub node_addresses: HashMap<String, String>,
    /// Election timeout in milliseconds
    pub election_timeout_ms: u64,
    /// Heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,
    /// Maximum number of log entries per RPC
    pub max_log_entries_per_rpc: usize,
}

impl Default for RaftConfig {
    fn default() -> Self {
        Self {
            node_id: "node-0".to_string(),
            node_addresses: HashMap::new(),
            election_timeout_ms: 1500,
            heartbeat_interval_ms: 500,
            max_log_entries_per_rpc: 100,
        }
    }
}

/// Raft log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Log index
    pub index: u64,
    /// Term when entry was created
    pub term: u64,
    /// Command to execute
    pub command: Command,
}

/// Command to be executed by the state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// No-op command
    NoOp,
    /// Set key-value pair
    Set { key: String, value: String },
    /// Delete key
    Delete { key: String },
}

/// Raft node persistent state
#[derive(Debug, Clone)]
pub struct PersistentState {
    /// Latest term server has seen
    pub current_term: u64,
    /// Candidate ID that received vote in current term (or None)
    pub voted_for: Option<String>,
    /// Log entries
    pub log: Vec<LogEntry>,
}

/// Raft node volatile state
#[derive(Debug, Clone)]
pub struct VolatileState {
    /// Index of highest log entry known to be committed
    pub commit_index: u64,
    /// Index of highest log entry applied to state machine
    pub last_applied: u64,
    /// For each server, index of the next log entry to send to that server
    pub next_index: HashMap<String, u64>,
    /// For each server, index of highest log entry known to be replicated on server
    pub match_index: HashMap<String, u64>,
}

/// Raft node implementation
pub struct RaftNode {
    /// Node configuration
    config: RaftConfig,
    /// Node state
    state: NodeState,
    /// Persistent state
    persistent_state: PersistentState,
    /// Volatile state
    volatile_state: VolatileState,
    /// Last time election timeout was reset
    last_heartbeat: u64,
    /// State machine
    state_machine: HashMap<String, String>,
    /// Election timeout duration
    election_timeout: Duration,
    /// Heartbeat interval
    heartbeat_interval: Duration,
}

impl RaftNode {
    /// Create a new Raft node
    pub fn new(config: RaftConfig) -> Self {
        let election_timeout = Duration::from_millis(config.election_timeout_ms);
        let heartbeat_interval = Duration::from_millis(config.heartbeat_interval_ms);

        // Initialize next_index and match_index for all nodes
        let mut next_index = HashMap::new();
        let mut match_index = HashMap::new();

        for node_id in config.node_addresses.keys() {
            if node_id != &config.node_id {
                next_index.insert(node_id.clone(), 1);
                match_index.insert(node_id.clone(), 0);
            }
        }

        Self {
            config,
            state: NodeState::Follower,
            persistent_state: PersistentState {
                current_term: 0,
                voted_for: None,
                log: vec![LogEntry {
                    index: 0,
                    term: 0,
                    command: Command::NoOp,
                }],
            },
            volatile_state: VolatileState {
                commit_index: 0,
                last_applied: 0,
                next_index,
                match_index,
            },
            last_heartbeat: Self::current_time(),
            state_machine: HashMap::new(),
            election_timeout,
            heartbeat_interval,
        }
    }

    /// Get current time in milliseconds since UNIX epoch
    fn current_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Start the Raft node
    pub async fn start(&mut self) {
        loop {
            match self.state {
                NodeState::Follower => self.run_follower().await,
                NodeState::Candidate => self.run_candidate().await,
                NodeState::Leader => self.run_leader().await,
            }
        }
    }

    /// Run follower logic
    async fn run_follower(&mut self) {
        let timeout = self.election_timeout.as_millis() as u64;
        let elapsed = Self::current_time() - self.last_heartbeat;

        if elapsed > timeout {
            // Election timeout, become candidate
            self.become_candidate();
        } else {
            // Sleep for a short duration and check again
            sleep(Duration::from_millis(50)).await;
        }
    }

    /// Run candidate logic
    async fn run_candidate(&mut self) {
        // Start election
        self.start_election().await;

        // Wait for election results or timeout
        let start_time = Self::current_time();
        while Self::current_time() - start_time < self.election_timeout.as_millis() as u64 {
            // Check if we've received enough votes
            if self.check_election_result() {
                return;
            }

            sleep(Duration::from_millis(50)).await;
        }

        // Election timeout, start new election
        self.become_candidate();
    }

    /// Run leader logic
    async fn run_leader(&mut self) {
        // Send heartbeat to all followers
        self.send_heartbeat().await;

        // Sleep for heartbeat interval
        sleep(self.heartbeat_interval).await;
    }

    /// Become a candidate and start election
    fn become_candidate(&mut self) {
        self.state = NodeState::Candidate;
        self.persistent_state.current_term += 1;
        self.persistent_state.voted_for = Some(self.config.node_id.clone());
        self.last_heartbeat = Self::current_time();

        println!(
            "Node {} became candidate for term {}",
            self.config.node_id, self.persistent_state.current_term
        );
    }

    /// Start election process
    async fn start_election(&self) {
        println!(
            "Node {} starting election for term {}",
            self.config.node_id, self.persistent_state.current_term
        );

        // In a real implementation, we would send RequestVote RPCs to other nodes
        // This is a simplified implementation for demonstration
    }

    /// Check if election was successful
    fn check_election_result(&self) -> bool {
        // In a real implementation, we would check if we received votes from
        // majority of nodes
        // This is a simplified implementation for demonstration
        false
    }

    /// Send heartbeat to all followers
    async fn send_heartbeat(&self) {
        // In a real implementation, we would send AppendEntries RPCs to followers
        // This is a simplified implementation for demonstration
        println!("Leader {} sending heartbeat", self.config.node_id);
    }

    /// Apply log entries to state machine
    fn apply_log_entries(&mut self) {
        while self.volatile_state.commit_index > self.volatile_state.last_applied {
            self.volatile_state.last_applied += 1;

            if let Some(entry) = self
                .persistent_state
                .log
                .get(self.volatile_state.last_applied as usize)
                .cloned()
            {
                self.apply_command(&entry.command);
            }
        }
    }

    /// Apply command to state machine
    fn apply_command(&mut self, command: &Command) {
        match command {
            Command::NoOp => {
                // No operation
            }
            Command::Set { key, value } => {
                self.state_machine.insert(key.clone(), value.clone());
            }
            Command::Delete { key } => {
                self.state_machine.remove(key);
            }
        }
    }

    /// Submit a command to be replicated
    pub async fn submit_command(&mut self, command: Command) -> Result<(), RaftError> {
        if self.state != NodeState::Leader {
            return Err(RaftError::NotLeader);
        }

        let entry = LogEntry {
            index: self.persistent_state.log.len() as u64,
            term: self.persistent_state.current_term,
            command,
        };

        self.persistent_state.log.push(entry);
        Ok(())
    }

    /// Begin a leader election with explicit peers (in-memory coordination)
    pub fn initiate_election(&mut self, peers: &mut [&mut RaftNode]) -> Result<bool, RaftError> {
        if self.state == NodeState::Leader {
            return Ok(true);
        }

        self.become_candidate();
        let mut votes = 1; // self-vote
        let (last_log_index, last_log_term) = self.last_log_info();
        let majority = self.majority_for(peers.len());

        for peer in peers.iter_mut() {
            let response = peer.handle_request_vote(
                self.persistent_state.current_term,
                self.config.node_id.clone(),
                last_log_index,
                last_log_term,
            );

            if response.term > self.persistent_state.current_term {
                self.update_term_and_step_down(response.term);
                return Ok(false);
            }

            if response.vote_granted {
                votes += 1;
            }

            if votes >= majority {
                self.state = NodeState::Leader;
                self.record_heartbeat();
                self.rebuild_replication_indexes(peers);
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Send heartbeats (empty append entries) to followers
    pub fn send_heartbeats(&mut self, peers: &mut [&mut RaftNode]) -> Result<(), RaftError> {
        if self.state != NodeState::Leader {
            return Err(RaftError::NotLeader);
        }

        let (prev_log_index, prev_log_term) = self.last_log_info();
        for peer in peers.iter_mut() {
            let response = peer.handle_append_entries(
                self.persistent_state.current_term,
                self.config.node_id.clone(),
                prev_log_index,
                prev_log_term,
                Vec::new(),
                self.volatile_state.commit_index,
            );

            if response.term > self.persistent_state.current_term {
                self.update_term_and_step_down(response.term);
                return Err(RaftError::NotLeader);
            }
        }

        self.record_heartbeat();
        Ok(())
    }

    /// Submit a command and replicate it to peers, committing once a quorum acknowledges
    pub fn submit_command_to_cluster(
        &mut self,
        command: Command,
        peers: &mut [&mut RaftNode],
    ) -> Result<(), RaftError> {
        if self.state != NodeState::Leader {
            return Err(RaftError::NotLeader);
        }

        let entry = LogEntry {
            index: self.persistent_state.log.len() as u64,
            term: self.persistent_state.current_term,
            command,
        };

        self.persistent_state.log.push(entry.clone());
        let majority = self.majority_for(peers.len());
        let prev_log_index = entry.index - 1;
        let prev_log_term = self.persistent_state.log[prev_log_index as usize].term;
        let mut successful_replicas = 1; // self

        for peer in peers.iter_mut() {
            let response = peer.handle_append_entries(
                self.persistent_state.current_term,
                self.config.node_id.clone(),
                prev_log_index,
                prev_log_term,
                vec![entry.clone()],
                entry.index,
            );

            if response.term > self.persistent_state.current_term {
                self.update_term_and_step_down(response.term);
                return Err(RaftError::NotLeader);
            }

            if response.success {
                successful_replicas += 1;
                self.volatile_state
                    .match_index
                    .insert(peer.config.node_id.clone(), entry.index);
                self.volatile_state
                    .next_index
                    .insert(peer.config.node_id.clone(), entry.index + 1);
            }
        }

        if successful_replicas >= majority {
            self.volatile_state.commit_index = entry.index;
            self.apply_log_entries();
        }

        Ok(())
    }

    /// Get value from state machine
    pub fn get_value(&self, key: &str) -> Option<&String> {
        self.state_machine.get(key)
    }

    /// Get current leader
    pub fn get_leader(&self) -> Option<&String> {
        if self.state == NodeState::Leader {
            Some(&self.config.node_id)
        } else {
            // In a real implementation, we would track the current leader
            None
        }
    }

    /// Handle incoming RequestVote RPC
    pub fn handle_request_vote(
        &mut self,
        term: u64,
        candidate_id: String,
        last_log_index: u64,
        last_log_term: u64,
    ) -> RequestVoteResponse {
        // If candidate's term is less than current term, reject
        if term < self.persistent_state.current_term {
            return RequestVoteResponse {
                term: self.persistent_state.current_term,
                vote_granted: false,
            };
        }

        // If candidate's term is greater, update term and become follower
        if term > self.persistent_state.current_term {
            self.persistent_state.current_term = term;
            self.persistent_state.voted_for = None;
            self.state = NodeState::Follower;
        }

        // Reset election timeout
        self.last_heartbeat = Self::current_time();

        // Check if we've already voted in this term
        if let Some(voted_for) = &self.persistent_state.voted_for {
            if voted_for != &candidate_id {
                return RequestVoteResponse {
                    term: self.persistent_state.current_term,
                    vote_granted: false,
                };
            }
        }

        // Check if candidate's log is at least as up-to-date as ours
        let last_log_entry = self.persistent_state.log.last().unwrap();
        let up_to_date = last_log_term > last_log_entry.term
            || (last_log_term == last_log_entry.term && last_log_index >= last_log_entry.index);

        if up_to_date {
            self.persistent_state.voted_for = Some(candidate_id);
            RequestVoteResponse {
                term: self.persistent_state.current_term,
                vote_granted: true,
            }
        } else {
            RequestVoteResponse {
                term: self.persistent_state.current_term,
                vote_granted: false,
            }
        }
    }

    /// Handle incoming AppendEntries RPC
    pub fn handle_append_entries(
        &mut self,
        term: u64,
        leader_id: String,
        prev_log_index: u64,
        prev_log_term: u64,
        entries: Vec<LogEntry>,
        leader_commit: u64,
    ) -> AppendEntriesResponse {
        // If leader's term is less than current term, reject
        if term < self.persistent_state.current_term {
            return AppendEntriesResponse {
                term: self.persistent_state.current_term,
                success: false,
            };
        }

        // If leader's term is greater, update term and become follower
        if term > self.persistent_state.current_term {
            self.persistent_state.current_term = term;
            self.persistent_state.voted_for = None;
            self.state = NodeState::Follower;
        }

        // Reset election timeout
        self.last_heartbeat = Self::current_time();

        // Check if log contains entry at prev_log_index with matching term
        if prev_log_index >= self.persistent_state.log.len() as u64 {
            return AppendEntriesResponse {
                term: self.persistent_state.current_term,
                success: false,
            };
        }

        if self.persistent_state.log[prev_log_index as usize].term != prev_log_term {
            // Delete conflicting entry and all that follow it
            self.persistent_state.log.truncate(prev_log_index as usize);
            return AppendEntriesResponse {
                term: self.persistent_state.current_term,
                success: false,
            };
        }

        // Append any new entries not already in the log
        for entry in entries {
            if entry.index < self.persistent_state.log.len() as u64 {
                // Entry already exists, check if it conflicts
                if self.persistent_state.log[entry.index as usize].term != entry.term {
                    // Conflict, delete existing entry and all that follow it
                    self.persistent_state.log.truncate(entry.index as usize);
                    self.persistent_state.log.push(entry);
                }
            } else {
                // Entry doesn't exist, append it
                self.persistent_state.log.push(entry);
            }
        }

        // Update commit index
        if leader_commit > self.volatile_state.commit_index {
            self.volatile_state.commit_index = std::cmp::min(
                leader_commit,
                self.persistent_state
                    .log
                    .last()
                    .map(|e| e.index)
                    .unwrap_or(0),
            );
        }

        // Apply committed entries to state machine
        self.apply_log_entries();

        AppendEntriesResponse {
            term: self.persistent_state.current_term,
            success: true,
        }
    }

    /// Update term and move to follower state
    fn update_term_and_step_down(&mut self, new_term: u64) {
        if new_term > self.persistent_state.current_term {
            self.persistent_state.current_term = new_term;
        }
        self.state = NodeState::Follower;
        self.persistent_state.voted_for = None;
        self.record_heartbeat();
    }

    /// Ensure replication indexes exist for the provided peers
    fn rebuild_replication_indexes(&mut self, peers: &mut [&mut RaftNode]) {
        let next_index_value = self.persistent_state.log.len() as u64;
        for peer in peers.iter_mut() {
            let peer_id = peer.config.node_id.clone();
            self.volatile_state
                .next_index
                .insert(peer_id.clone(), next_index_value);
            self.volatile_state.match_index.insert(peer_id, 0);
        }
    }

    /// Helper to record a heartbeat timestamp
    fn record_heartbeat(&mut self) {
        self.last_heartbeat = Self::current_time();
    }

    /// Get the last log index and term
    fn last_log_info(&self) -> (u64, u64) {
        let last = self.persistent_state.log.last().unwrap();
        (last.index, last.term)
    }

    /// Calculate quorum size from peer count
    fn majority_for(&self, peer_count: usize) -> usize {
        (peer_count + 1) / 2 + 1
    }
}

/// Response to RequestVote RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestVoteResponse {
    /// Current term for candidate to update itself
    pub term: u64,
    /// True means candidate received vote
    pub vote_granted: bool,
}

/// Response to AppendEntries RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesResponse {
    /// Current term for leader to update itself
    pub term: u64,
    /// True if follower contained entry matching prev_log_index and prev_log_term
    pub success: bool,
}

/// Errors that can occur in Raft consensus
#[derive(Debug, Error)]
pub enum RaftError {
    #[error("Node is not the leader")]
    NotLeader,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Timeout error")]
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raft_node_creation() {
        let config = RaftConfig::default();
        let node = RaftNode::new(config);

        assert_eq!(node.state, NodeState::Follower);
        assert_eq!(node.persistent_state.current_term, 0);
        assert_eq!(node.persistent_state.log.len(), 1); // Initial NoOp entry
    }

    #[test]
    fn test_become_candidate() {
        let config = RaftConfig::default();
        let mut node = RaftNode::new(config);

        node.become_candidate();

        assert_eq!(node.state, NodeState::Candidate);
        assert_eq!(node.persistent_state.current_term, 1);
        assert_eq!(node.persistent_state.voted_for, Some("node-0".to_string()));
    }

    #[test]
    fn test_handle_request_vote() {
        let config = RaftConfig::default();
        let mut node = RaftNode::new(config);

        let response = node.handle_request_vote(1, "candidate-1".to_string(), 0, 0);

        assert_eq!(response.term, 1);
        assert!(response.vote_granted);
        assert_eq!(
            node.persistent_state.voted_for,
            Some("candidate-1".to_string())
        );
    }

    #[test]
    fn test_handle_append_entries() {
        let config = RaftConfig::default();
        let mut node = RaftNode::new(config);

        let entries = vec![LogEntry {
            index: 1,
            term: 1,
            command: Command::NoOp,
        }];

        let response = node.handle_append_entries(1, "leader-1".to_string(), 0, 0, entries, 0);

        assert_eq!(response.term, 1);
        assert!(response.success);
        assert_eq!(node.persistent_state.log.len(), 2);
    }

    #[test]
    fn test_candidate_wins_quorum_election() {
        let mut config_a = RaftConfig::default();
        config_a.node_id = "node-a".to_string();
        config_a
            .node_addresses
            .insert("node-b".to_string(), "addr-b".to_string());
        config_a
            .node_addresses
            .insert("node-c".to_string(), "addr-c".to_string());

        let mut config_b = RaftConfig::default();
        config_b.node_id = "node-b".to_string();
        config_b
            .node_addresses
            .insert("node-a".to_string(), "addr-a".to_string());
        config_b
            .node_addresses
            .insert("node-c".to_string(), "addr-c".to_string());

        let mut config_c = RaftConfig::default();
        config_c.node_id = "node-c".to_string();
        config_c
            .node_addresses
            .insert("node-a".to_string(), "addr-a".to_string());
        config_c
            .node_addresses
            .insert("node-b".to_string(), "addr-b".to_string());

        let mut node_a = RaftNode::new(config_a);
        let mut node_b = RaftNode::new(config_b);
        let mut node_c = RaftNode::new(config_c);

        let election_won = {
            let peers: &mut [&mut RaftNode] = &mut [&mut node_b, &mut node_c];
            node_a.initiate_election(peers).unwrap()
        };

        assert!(election_won);
        assert_eq!(node_a.state, NodeState::Leader);
        assert_eq!(node_a.get_leader(), Some(&"node-a".to_string()));
        assert_eq!(node_b.state, NodeState::Follower);
        assert_eq!(node_c.state, NodeState::Follower);
        assert_eq!(node_b.persistent_state.voted_for, Some("node-a".to_string()));
        assert_eq!(node_c.persistent_state.voted_for, Some("node-a".to_string()));
    }

    #[test]
    fn test_election_rejected_by_higher_term_peer() {
        let mut node_a = RaftNode::new(RaftConfig {
            node_id: "node-a".to_string(),
            ..Default::default()
        });

        let mut node_b = RaftNode::new(RaftConfig {
            node_id: "node-b".to_string(),
            ..Default::default()
        });

        // Simulate node_b being ahead in term (already a leader elsewhere)
        node_b.persistent_state.current_term = 3;
        node_b.state = NodeState::Leader;

        let result = {
            let peers: &mut [&mut RaftNode] = &mut [&mut node_b];
            node_a.initiate_election(peers).unwrap()
        };

        assert!(!result);
        assert_eq!(node_a.state, NodeState::Follower);
        assert_eq!(node_a.persistent_state.current_term, 3);
    }

    #[test]
    fn test_leader_sends_heartbeats() {
        let mut leader = RaftNode::new(RaftConfig {
            node_id: "node-a".to_string(),
            ..Default::default()
        });
        let mut follower = RaftNode::new(RaftConfig {
            node_id: "node-b".to_string(),
            ..Default::default()
        });

        leader.state = NodeState::Leader;
        let previous_heartbeat = follower.last_heartbeat;

        {
            let peers: &mut [&mut RaftNode] = &mut [&mut follower];
            leader.send_heartbeats(peers).unwrap();
        }

        assert_eq!(follower.state, NodeState::Follower);
        assert!(follower.last_heartbeat >= previous_heartbeat);
    }

    #[test]
    fn test_command_replication_and_commit() {
        let mut node_a = RaftNode::new(RaftConfig {
            node_id: "node-a".to_string(),
            ..Default::default()
        });
        let mut node_b = RaftNode::new(RaftConfig {
            node_id: "node-b".to_string(),
            ..Default::default()
        });
        let mut node_c = RaftNode::new(RaftConfig {
            node_id: "node-c".to_string(),
            ..Default::default()
        });

        {
            let peers: &mut [&mut RaftNode] = &mut [&mut node_b, &mut node_c];
            assert!(node_a.initiate_election(peers).unwrap());
        }

        {
            let peers: &mut [&mut RaftNode] = &mut [&mut node_b, &mut node_c];
            node_a
                .submit_command_to_cluster(
                    Command::Set {
                        key: "orders".to_string(),
                        value: "open".to_string(),
                    },
                    peers,
                )
                .unwrap();
        }

        assert_eq!(node_a.get_value("orders"), Some(&"open".to_string()));
        assert_eq!(node_b.get_value("orders"), Some(&"open".to_string()));
        assert_eq!(node_c.get_value("orders"), Some(&"open".to_string()));
        assert!(node_a.volatile_state.commit_index >= 1);
    }
}
