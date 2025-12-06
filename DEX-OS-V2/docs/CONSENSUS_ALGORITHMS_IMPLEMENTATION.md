# Consensus Algorithms Implementation

This document describes the implementation of consensus algorithms in the DEX-OS system. This implements the Priority 3 features from DEX-OS-V2.csv:

- Distributed Systems,Distributed Systems,Distributed Systems,Consensus,Paxos Algorithm,Medium
- Distributed Systems,Distributed Systems,Distributed Systems,Consensus,Two-Phase Commit,Medium

## Overview

Consensus algorithms are fundamental to distributed systems, ensuring that multiple nodes agree on a single data value or state. The DEX-OS system implements three major consensus algorithms:

1. **Raft**: Already implemented in `dex-core/src/consensus/raft.rs`
2. **Paxos**: Newly implemented in `dex-core/src/consensus/paxos.rs`
3. **Two-Phase Commit**: Newly implemented in `dex-core/src/consensus/two_phase_commit.rs`

## Paxos Consensus Algorithm

### Implementation Details

The Paxos implementation follows the classic Paxos algorithm with three phases:

1. **Prepare Phase**: The proposer sends a prepare request with a proposal number to acceptors
2. **Promise Phase**: Acceptors respond with promises and any previously accepted values
3. **Accept Phase**: The proposer sends accept requests with a value, and acceptors respond with accepted responses

### Core Components

1. **PaxosNode**: The main struct representing a node in the Paxos network
2. **ProposalNumber**: A unique identifier for each proposal with round number and node ID
3. **PaxosValue**: The value being agreed upon with timestamp
4. **NodeRole**: The role of the node (Proposer, Acceptor, or Learner)
5. **PaxosConfig**: Configuration parameters for the Paxos node

### Key Features

#### Proposal Numbering
Paxos uses a two-component proposal number (round, node_id) to ensure uniqueness and ordering:

```rust
let p1 = ProposalNumber::new(1, 1);
let p2 = ProposalNumber::new(1, 2);
let p3 = ProposalNumber::new(2, 1);
// p1 < p2 < p3
```

#### Role-Based Design
Each node can take on one of three roles:
- **Proposer**: Initiates consensus by proposing values
- **Acceptor**: Participates in consensus by accepting proposals
- **Learner**: Learns the final agreed-upon value

#### State Management
Each role maintains its own state:
- **AcceptorState**: Tracks promised and accepted proposals
- **ProposerState**: Tracks current proposal and received responses
- **LearnerState**: Tracks learned values and decided value

### Usage Examples

#### Creating a Paxos Node
```rust
use dex_core::consensus::paxos::{PaxosConfig, PaxosNode, NodeRole};

let config = PaxosConfig::default();
let proposer = PaxosNode::new(config, NodeRole::Proposer);
```

#### Proposing a Value
```rust
use dex_core::consensus::paxos::{PaxosValue};

let value = PaxosValue::new(b"agreed_value".to_vec());
// In an async context:
// let result = proposer.propose(value).await;
```

## Two-Phase Commit Consensus Algorithm

### Implementation Details

The Two-Phase Commit (2PC) implementation follows the classic distributed transaction protocol with two phases:

1. **Prepare Phase**: The coordinator asks all participants if they're ready to commit
2. **Commit/Abort Phase**: Based on participant responses, the coordinator decides to commit or abort

### Core Components

1. **TwoPhaseCommitNode**: The main struct representing a node in the 2PC protocol
2. **TransactionId**: A unique identifier for each transaction
3. **NodeRole**: The role of the node (Coordinator or Participant)
4. **TwoPhaseCommitConfig**: Configuration parameters for the 2PC node

### Key Features

#### Transaction Management
The coordinator maintains state for all active transactions:
- **Active**: Transaction is being processed
- **Prepared**: All participants have agreed to commit
- **Committed**: Transaction has been successfully committed
- **Aborted**: Transaction has been aborted

#### Participant States
Participants maintain their own state:
- **Working**: Processing the transaction
- **Ready**: Ready to commit
- **Committed**: Transaction committed
- **Aborted**: Transaction aborted

#### Error Handling
The implementation includes comprehensive error handling for:
- Network failures
- Timeouts
- Participant failures
- Invalid states

### Usage Examples

#### Creating a Coordinator Node
```rust
use dex_core::consensus::two_phase_commit::{TwoPhaseCommitConfig, TwoPhaseCommitNode, NodeRole};

let config = TwoPhaseCommitConfig::default();
let coordinator = TwoPhaseCommitNode::new(config, NodeRole::Coordinator);
```

#### Beginning a Transaction
```rust
use dex_core::consensus::two_phase_commit::{TransactionId};

let tx_id = TransactionId::new("transaction-1".to_string());
let data = b"transaction_data".to_vec();
// coordinator.begin_transaction(tx_id, data)?;
```

#### Executing a Transaction
```rust
// In an async context:
// coordinator.execute_transaction(tx_id).await?;
```

## Testing

The implementation includes comprehensive tests that validate:

1. Basic functionality for all components
2. Error handling for invalid operations
3. State transitions for all roles
4. Protocol compliance for both algorithms
5. Edge cases and boundary conditions

Tests can be run with:

```bash
cargo test consensus_algorithms_tests
```

## Integration with DEX-OS

The consensus algorithms can be integrated into various components of the DEX-OS system:

1. **Service Coordination**: Raft for leader election and service coordination
2. **Distributed Transactions**: Two-Phase Commit for multi-node transactions
3. **Agreement Protocols**: Paxos for reaching consensus on system state

## Benefits

1. **Fault Tolerance**: All algorithms handle node failures gracefully
2. **Scalability**: Designed to work with varying numbers of nodes
3. **Safety**: Mathematical guarantees for correctness
4. **Flexibility**: Role-based design allows for different deployment scenarios
5. **Monitoring**: Clear state visibility for debugging and monitoring

## Performance Considerations

1. **Network Efficiency**: Minimal message exchange for common cases
2. **State Management**: Efficient state storage and retrieval
3. **Timeout Handling**: Configurable timeouts to balance safety and performance
4. **Concurrency**: Thread-safe design for concurrent operations

## Future Enhancements

Potential future improvements could include:

1. **Multi-Paxos**: Optimizations for running multiple Paxos instances
2. **Byzantine Fault Tolerance**: Extensions to handle malicious nodes
3. **Dynamic Membership**: Support for adding/removing nodes during operation
4. **Performance Metrics**: Built-in metrics collection and reporting
5. **Configuration Management**: Dynamic configuration updates