# CQRS Command/Query Separation

Priority 3 feature: `Distributed Systems, Distributed Systems, Distributed Systems, CQRS, Command/Query Separation, Medium`

- **Module**: `dex-core/src/cqrs.rs`
- **Capabilities**: strict write/read separation with command validation, append-only event store (sequenced envelopes with timestamps), read-model projections for orders/volume, and replay support to rebuild projections.
- **Concurrency**: command state protected by a mutex; read model uses an `RwLock` so queries stay read-only while writes project events.
- **Recovery**: `rebuild_read_model_from_events` replays the event stream to repair/query nodes without touching the write model.

## Usage

```rust
use dex_core::cqrs::{CqrsCommand, CqrsEngine};
use dex_core::types::{OrderSide, TradingPair};

let engine = CqrsEngine::new();
let pair = TradingPair { base: "ETH".into(), quote: "USDC".into() };

engine.execute_command(CqrsCommand::RegisterAccount { account_id: "alice".into() })?;
engine.execute_command(CqrsCommand::PlaceOrder {
    order_id: 1,
    account_id: "alice".into(),
    pair: pair.clone(),
    side: OrderSide::Buy,
    quantity: 10,
})?;

// Query side reads the projection only
let open_orders = engine.query_open_orders("alice");

// Projection rebuild (useful for new nodes or recovery)
engine.rebuild_read_model_from_events();
```

## Tests

- Unit tests in `dex-core/src/cqrs.rs` cover command validation, state transitions, and projection rebuilds.
- Integration tests in `dex-core/tests/cqrs_command_query_separation.rs` exercise parallel command execution, projection replay, and event-stream sequencing.
