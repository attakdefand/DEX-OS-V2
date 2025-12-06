# Quick Reference: Layer 2 & Cross-Chain Features

## State Channels (`state_channels.rs`)

### Key Types
```rust
StateChannelManager    // Main manager
StateChannel          // Individual channel
OffChainOrder         // Off-chain order
StateUpdate           // State transition
Participant           // Channel participant
```

### Basic Usage
```rust
let manager = StateChannelManager::new(ChannelConfig::default());

// 1. Open channel
manager.open_channel(channel_id, participants)?;

// 2. Deposit funds
manager.deposit(channel_id, user, amount)?;

// 3. Activate
manager.activate_channel(channel_id)?;

// 4. Submit orders
manager.submit_order(channel_id, order)?;

// 5. Close
manager.close_channel(channel_id)?;
manager.finalize_close(channel_id)?;
```

### Statistics
```rust
let stats = manager.get_statistics();
println!("Active channels: {}", stats.active_channels);
println!("Total orders: {}", stats.total_orders);
```

---

## Batch Settlements (`batch_settlements.rs`)

### Key Types
```rust
BatchSettlementManager  // Main manager
SettlementBatch        // Transaction batch
Transaction            // Individual transaction
TransactionType        // Transfer, Trade, Swap, Deposit, Withdrawal
```

### Basic Usage
```rust
let manager = BatchSettlementManager::new(BatchConfig::default());

// 1. Create batch (optional, auto-created)
manager.create_batch(batch_id)?;

// 2. Add transactions (auto-batches)
let tx = Transaction::new(id, tx_type, from, to, asset, amount, fee);
manager.add_transaction(tx)?;

// 3. Settle batch
manager.settle_batch(batch_id, tx_hash)?;

// 4. Get Merkle proof
let batch = manager.get_batch(batch_id)?;
let proof = batch.get_merkle_proof(tx_id)?;
```

### Configuration
```rust
let config = BatchConfig {
    max_batch_size: 1000,
    min_batch_size: 10,
    batch_timeout: 300,  // 5 minutes
    max_pending_batches: 100,
};
```

---

## IBC Protocol (`ibc_protocol.rs`)

### Key Types
```rust
IBCManager         // Main manager
LightClient        // Remote chain verifier
Connection         // Chain connection
Channel            // Communication channel
Packet             // Cross-chain message
TransferData       // ICS-20 token transfer
```

### Basic Usage
```rust
let ibc = IBCManager::new(chain_id);

// 1. Create light client
ibc.create_client(client_id, remote_chain_id)?;

// 2. Update client
ibc.update_client(client_id, height, consensus_state)?;

// 3. Create connection
ibc.create_connection(conn_id, client_id, counterparty_client)?;
ibc.open_connection(conn_id, counterparty_conn_id)?;

// 4. Create channel
ibc.create_channel(channel_id, conn_id, port_id, PacketOrdering::Unordered)?;
ibc.open_channel(channel_id, counterparty_channel_id)?;

// 5. Transfer tokens (ICS-20)
let packet = ibc.transfer(
    channel_id,
    denom,
    amount,
    sender,
    receiver,
    timeout_height,
    timeout_timestamp
)?;
```

### Packet Handling
```rust
// Send packet
let packet = ibc.send_packet(channel_id, data, timeout_h, timeout_t)?;

// Receive packet
let ack = ibc.receive_packet(packet, proof)?;

// Acknowledge packet
ibc.acknowledge_packet(packet, ack, proof)?;
```

---

## Integration Example

### State Channel → Batch Settlement
```rust
// Close state channel
let final_balances = channel_manager.finalize_close(channel_id)?;

// Create settlement batch
for (user, balance) in final_balances {
    let tx = Transaction::new(
        format!("settlement_{}", user),
        TransactionType::Withdrawal,
        channel_id,
        user,
        asset,
        balance,
        0
    );
    batch_manager.add_transaction(tx)?;
}

// Settle on-chain
batch_manager.settle_batch(batch_id, tx_hash)?;
```

### IBC → Batch Settlement
```rust
// Receive IBC transfer
let packet = ibc.receive_packet(incoming_packet, proof)?;
let transfer = TransferData::decode(&packet.data)?;

// Batch the received tokens
let tx = Transaction::new(
    format!("ibc_{}", packet.sequence),
    TransactionType::Deposit,
    "ibc_escrow",
    transfer.receiver,
    format!("ibc/{}", transfer.denom),
    transfer.amount,
    0
);
batch_manager.add_transaction(tx)?;
```

---

## Error Handling

### State Channels
```rust
match manager.submit_order(channel_id, order) {
    Ok(_) => println!("Order submitted"),
    Err(StateChannelError::ChannelNotFound) => println!("Channel doesn't exist"),
    Err(StateChannelError::InsufficientBalance) => println!("Not enough balance"),
    Err(StateChannelError::ChannelClosed) => println!("Channel is closed"),
    Err(e) => println!("Error: {:?}", e),
}
```

### Batch Settlements
```rust
match manager.add_transaction(tx) {
    Ok(batch_id) => println!("Added to batch: {}", batch_id),
    Err(BatchSettlementError::BatchFull) => println!("Batch is full"),
    Err(BatchSettlementError::InvalidTransaction) => println!("Invalid tx"),
    Err(e) => println!("Error: {:?}", e),
}
```

### IBC Protocol
```rust
match ibc.transfer(channel_id, denom, amount, sender, receiver, h, t) {
    Ok(packet) => println!("Transfer sent: seq {}", packet.sequence),
    Err(IBCError::ChannelNotFound) => println!("Channel doesn't exist"),
    Err(IBCError::ChannelClosed) => println!("Channel is closed"),
    Err(IBCError::TimeoutExpired) => println!("Packet timed out"),
    Err(e) => println!("Error: {:?}", e),
}
```

---

## Testing

### Run All Tests
```bash
# Test individual modules
cargo test --lib state_channels
cargo test --lib batch_settlements
cargo test --lib ibc_protocol

# Test integration
cargo test --test layer2_crosschain_tests

# Run specific test
cargo test test_state_channel_full_lifecycle
```

### Test Categories
- **Unit tests**: In each module file
- **Integration tests**: `tests/layer2_crosschain_tests.rs`
- **Performance tests**: High-throughput scenarios
- **Cross-module tests**: Integration between features

---

## Performance Tips

### State Channels
- Keep max_pending_updates reasonable (default: 1000)
- Close inactive channels to free memory
- Use challenge_period wisely (default: 24h)

### Batch Settlements
- Tune batch size based on gas prices
- Use min_batch_size to avoid tiny batches
- Monitor batch_timeout for optimal settlement

### IBC Protocol
- Reuse connections for multiple channels
- Set appropriate timeouts (height + time)
- Use unordered channels when possible for better throughput

---

## Common Patterns

### High-Frequency Trading
```rust
// Use state channels for rapid order matching
let channel = manager.open_channel(id, traders)?;
for order in high_freq_orders {
    manager.submit_order(channel_id, order)?;
}
// Settle periodically via batch
```

### Cross-Chain Swaps
```rust
// Lock on source chain
let packet = ibc.transfer(channel, token, amount, user, escrow, h, t)?;

// Receive on destination
let ack = ibc.receive_packet(packet, proof)?;

// Batch settle on destination
batch_manager.add_transaction(settlement_tx)?;
```

### Gas Optimization
```rust
// Accumulate transactions
for tx in transactions {
    batch_manager.add_transaction(tx)?;
}

// Single on-chain settlement
batch_manager.settle_batch(batch_id, tx_hash)?;
```

---

## Monitoring

### State Channels
```rust
let stats = manager.get_statistics();
println!("Total channels: {}", stats.total_channels);
println!("Active channels: {}", stats.active_channels);
println!("Total orders: {}", stats.total_orders);
println!("Total volume: {}", stats.total_volume);
```

### Batch Settlements
```rust
let stats = manager.get_statistics();
println!("Total batches: {}", stats.total_batches);
println!("Settled batches: {}", stats.settled_batches);
println!("Total transactions: {}", stats.total_transactions);
println!("Total fees: {}", stats.total_fees);
```

### IBC Protocol
```rust
let stats = ibc.get_statistics();
println!("Total clients: {}", stats.total_clients);
println!("Open connections: {}", stats.open_connections);
println!("Open channels: {}", stats.open_channels);
println!("Pending packets: {}", stats.pending_packets);
```

---

## Security Checklist

### State Channels
- ✅ Verify all participant signatures
- ✅ Validate nonce ordering
- ✅ Check balance sufficiency
- ✅ Enforce challenge periods
- ✅ Audit state history

### Batch Settlements
- ✅ Validate all transactions
- ✅ Verify Merkle proofs
- ✅ Check net balances
- ✅ Enforce batch size limits
- ✅ Track settlement hashes

### IBC Protocol
- ✅ Verify consensus states
- ✅ Validate proofs
- ✅ Check packet timeouts
- ✅ Enforce sequence ordering (ordered channels)
- ✅ Prevent packet replay

---

**Quick Start**: See `LAYER2_CROSSCHAIN_IMPLEMENTATION.md` for detailed documentation  
**Full Implementation**: See individual module files for complete API reference  
**Tests**: See `tests/layer2_crosschain_tests.rs` for usage examples
