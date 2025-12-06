//! Command/Query Responsibility Segregation (CQRS) primitives for distributed systems.
//!
//! Implements the Priority 3 feature from DEX-OS-V2.csv:
//! "Distributed Systems,Distributed Systems,Distributed Systems,CQRS,Command/Query Separation,Medium"

use crate::types::{OrderSide, TradingPair};
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Errors surfaced by the CQRS command side.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CqrsError {
    #[error("account already registered: {0}")]
    AccountExists(String),
    #[error("account not found: {0}")]
    AccountNotFound(String),
    #[error("order already exists: {0}")]
    DuplicateOrder(u64),
    #[error("order not found: {0}")]
    OrderNotFound(u64),
    #[error("order is not open: {0}")]
    OrderNotOpen(u64),
    #[error("fill quantity {attempted} exceeds remaining {remaining} for order {order_id}")]
    InvalidFill {
        order_id: u64,
        attempted: u64,
        remaining: u64,
    },
    #[error("order quantity must be greater than zero")]
    InvalidQuantity,
}

/// State tracked on the command side for validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    Open,
    Cancelled,
    Filled,
}

/// Commands that mutate system state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CqrsCommand {
    RegisterAccount {
        account_id: String,
    },
    PlaceOrder {
        order_id: u64,
        account_id: String,
        pair: TradingPair,
        side: OrderSide,
        quantity: u64,
    },
    CancelOrder {
        order_id: u64,
        reason: String,
    },
    RecordFill {
        order_id: u64,
        quantity: u64,
        price: u64,
    },
}

/// Events emitted by the command side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CqrsEvent {
    AccountRegistered {
        account_id: String,
    },
    OrderPlaced {
        order_id: u64,
        account_id: String,
        pair: TradingPair,
        side: OrderSide,
        quantity: u64,
    },
    OrderCancelled {
        order_id: u64,
        account_id: String,
        reason: String,
    },
    OrderFilled {
        order_id: u64,
        account_id: String,
        pair: TradingPair,
        filled: u64,
        remaining: u64,
        price: u64,
    },
}

/// Event plus sequencing metadata for projection replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventEnvelope {
    pub version: u64,
    pub occurred_at_ms: u128,
    pub event: CqrsEvent,
}

#[derive(Debug, Clone)]
struct OrderAggregate {
    account_id: String,
    pair: TradingPair,
    side: OrderSide,
    remaining: u64,
    status: OrderStatus,
}

#[derive(Debug, Clone, Default)]
struct CommandState {
    accounts: HashSet<String>,
    orders: HashMap<u64, OrderAggregate>,
}

impl CommandState {
    fn apply(&mut self, event: &CqrsEvent) {
        match event {
            CqrsEvent::AccountRegistered { account_id } => {
                self.accounts.insert(account_id.clone());
            }
            CqrsEvent::OrderPlaced {
                order_id,
                account_id,
                pair,
                side,
                quantity,
            } => {
                self.orders.insert(
                    *order_id,
                    OrderAggregate {
                        account_id: account_id.clone(),
                        pair: pair.clone(),
                        side: *side,
                        remaining: *quantity,
                        status: OrderStatus::Open,
                    },
                );
            }
            CqrsEvent::OrderCancelled { order_id, .. } => {
                if let Some(order) = self.orders.get_mut(order_id) {
                    order.status = OrderStatus::Cancelled;
                    order.remaining = 0;
                }
            }
            CqrsEvent::OrderFilled {
                order_id,
                remaining,
                ..
            } => {
                if let Some(order) = self.orders.get_mut(order_id) {
                    order.remaining = *remaining;
                    if *remaining == 0 {
                        order.status = OrderStatus::Filled;
                    }
                }
            }
        }
    }
}

/// Read-optimized order view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderView {
    pub order_id: u64,
    pub account_id: String,
    pub pair: TradingPair,
    pub side: OrderSide,
    pub status: OrderStatus,
    pub open_quantity: u64,
    pub filled_quantity: u64,
    pub last_updated_version: u64,
    pub last_updated_at_ms: u128,
}

#[derive(Debug, Clone, Default)]
struct QueryModel {
    version: u64,
    orders: HashMap<u64, OrderView>,
    volume_by_pair: HashMap<TradingPair, u64>,
}

impl QueryModel {
    fn project(&mut self, envelope: &EventEnvelope) {
        match &envelope.event {
            CqrsEvent::AccountRegistered { .. } => {
                // Accounts are not materialized on the read side, but the event is kept for replay.
            }
            CqrsEvent::OrderPlaced {
                order_id,
                account_id,
                pair,
                side,
                quantity,
            } => {
                self.orders.insert(
                    *order_id,
                    OrderView {
                        order_id: *order_id,
                        account_id: account_id.clone(),
                        pair: pair.clone(),
                        side: *side,
                        status: OrderStatus::Open,
                        open_quantity: *quantity,
                        filled_quantity: 0,
                        last_updated_version: envelope.version,
                        last_updated_at_ms: envelope.occurred_at_ms,
                    },
                );
            }
            CqrsEvent::OrderCancelled { order_id, .. } => {
                if let Some(view) = self.orders.get_mut(order_id) {
                    view.status = OrderStatus::Cancelled;
                    view.open_quantity = 0;
                    view.last_updated_version = envelope.version;
                    view.last_updated_at_ms = envelope.occurred_at_ms;
                }
            }
            CqrsEvent::OrderFilled {
                order_id,
                pair,
                filled,
                remaining,
                ..
            } => {
                if let Some(view) = self.orders.get_mut(order_id) {
                    view.filled_quantity = view
                        .filled_quantity
                        .saturating_add(*filled);
                    view.open_quantity = *remaining;
                    if *remaining == 0 {
                        view.status = OrderStatus::Filled;
                    }
                    view.last_updated_version = envelope.version;
                    view.last_updated_at_ms = envelope.occurred_at_ms;
                }
                let entry = self.volume_by_pair.entry(pair.clone()).or_insert(0);
                *entry = entry.saturating_add(*filled);
            }
        }
        self.version = envelope.version;
    }

    fn reset(&mut self) {
        *self = QueryModel::default();
    }
}

/// Result of executing a command on the write side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutcome {
    pub events: Vec<EventEnvelope>,
}

/// CQRS engine that separates command processing from query projections.
#[derive(Debug, Default)]
pub struct CqrsEngine {
    command_state: Mutex<CommandState>,
    read_model: RwLock<QueryModel>,
    event_store: Mutex<Vec<EventEnvelope>>,
}

impl CqrsEngine {
    /// Create a new CQRS engine instance.
    pub fn new() -> Self {
        Self {
            command_state: Mutex::new(CommandState::default()),
            read_model: RwLock::new(QueryModel::default()),
            event_store: Mutex::new(Vec::new()),
        }
    }

    /// Execute a command on the write side, returning the events that were applied.
    pub fn execute_command(
        &self,
        command: CqrsCommand,
    ) -> Result<CommandOutcome, CqrsError> {
        let events = {
            let mut state = self.command_state.lock().unwrap();
            let events = match command {
                CqrsCommand::RegisterAccount { account_id } => {
                    if state.accounts.contains(&account_id) {
                        return Err(CqrsError::AccountExists(account_id));
                    }
                    vec![CqrsEvent::AccountRegistered { account_id }]
                }
                CqrsCommand::PlaceOrder {
                    order_id,
                    account_id,
                    pair,
                    side,
                    quantity,
                } => {
                    if quantity == 0 {
                        return Err(CqrsError::InvalidQuantity);
                    }
                    if !state.accounts.contains(&account_id) {
                        return Err(CqrsError::AccountNotFound(account_id));
                    }
                    if state.orders.contains_key(&order_id) {
                        return Err(CqrsError::DuplicateOrder(order_id));
                    }
                    vec![CqrsEvent::OrderPlaced {
                        order_id,
                        account_id,
                        pair,
                        side,
                        quantity,
                    }]
                }
                CqrsCommand::CancelOrder { order_id, reason } => {
                    let aggregate =
                        state
                            .orders
                            .get(&order_id)
                            .ok_or(CqrsError::OrderNotFound(order_id))?;
                    if aggregate.status != OrderStatus::Open {
                        return Err(CqrsError::OrderNotOpen(order_id));
                    }
                    vec![CqrsEvent::OrderCancelled {
                        order_id,
                        account_id: aggregate.account_id.clone(),
                        reason,
                    }]
                }
                CqrsCommand::RecordFill {
                    order_id,
                    quantity,
                    price,
                } => {
                    if quantity == 0 {
                        return Err(CqrsError::InvalidQuantity);
                    }
                    let aggregate =
                        state
                            .orders
                            .get(&order_id)
                            .ok_or(CqrsError::OrderNotFound(order_id))?;
                    if aggregate.status != OrderStatus::Open {
                        return Err(CqrsError::OrderNotOpen(order_id));
                    }
                    if quantity > aggregate.remaining {
                        return Err(CqrsError::InvalidFill {
                            order_id,
                            attempted: quantity,
                            remaining: aggregate.remaining,
                        });
                    }
                    let remaining = aggregate.remaining - quantity;
                    vec![CqrsEvent::OrderFilled {
                        order_id,
                        account_id: aggregate.account_id.clone(),
                        pair: aggregate.pair.clone(),
                        filled: quantity,
                        remaining,
                        price,
                    }]
                }
            };
            for event in &events {
                state.apply(event);
            }
            events
        };

        let mut applied_events = Vec::new();
        for event in events {
            applied_events.push(self.append_and_project(event));
        }

        Ok(CommandOutcome {
            events: applied_events,
        })
    }

    /// Return the current read model version (the last applied event sequence).
    pub fn current_version(&self) -> u64 {
        self.read_model.read().unwrap().version
    }

    /// Read-only query for open orders belonging to a specific account.
    pub fn query_open_orders(&self, account_id: &str) -> Vec<OrderView> {
        self.read_model
            .read()
            .unwrap()
            .orders
            .values()
            .filter(|order| order.account_id == account_id && order.status == OrderStatus::Open)
            .cloned()
            .collect()
    }

    /// Read-only query for an order snapshot by id.
    pub fn query_order(&self, order_id: u64) -> Option<OrderView> {
        self.read_model
            .read()
            .unwrap()
            .orders
            .get(&order_id)
            .cloned()
    }

    /// Read-only query for cumulative executed volume per trading pair.
    pub fn query_volume_for_pair(&self, pair: &TradingPair) -> u64 {
        *self
            .read_model
            .read()
            .unwrap()
            .volume_by_pair
            .get(pair)
            .unwrap_or(&0)
    }

    /// Retrieve a copy of the event store to seed downstream projections.
    pub fn event_stream(&self) -> Vec<EventEnvelope> {
        self.event_store.lock().unwrap().clone()
    }

    /// Rebuild the read model from the persisted event store.
    pub fn rebuild_read_model_from_events(&self) {
        let events = self.event_store.lock().unwrap().clone();
        let mut read_model = self.read_model.write().unwrap();
        read_model.reset();
        for envelope in events {
            read_model.project(&envelope);
        }
    }

    fn append_and_project(&self, event: CqrsEvent) -> EventEnvelope {
        let envelope = {
            let mut store = self.event_store.lock().unwrap();
            let version = store.last().map(|e| e.version).unwrap_or(0) + 1;
            let envelope = EventEnvelope {
                version,
                occurred_at_ms: now_ms(),
                event,
            };
            store.push(envelope.clone());
            envelope
        };

        {
            let mut read_model = self.read_model.write().unwrap();
            read_model.project(&envelope);
        }

        envelope
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pair() -> TradingPair {
        TradingPair {
            base: "ETH".to_string(),
            quote: "USDC".to_string(),
        }
    }

    #[test]
    fn command_and_query_paths_are_decoupled() {
        let engine = CqrsEngine::new();

        // Write side: register account + place order
        engine
            .execute_command(CqrsCommand::RegisterAccount {
                account_id: "alice".to_string(),
            })
            .unwrap();

        let place_result = engine
            .execute_command(CqrsCommand::PlaceOrder {
                order_id: 1,
                account_id: "alice".to_string(),
                pair: sample_pair(),
                side: OrderSide::Buy,
                quantity: 50,
            })
            .unwrap();

        assert_eq!(place_result.events.len(), 1);
        assert_eq!(engine.current_version(), 2);

        // Read side: query open orders without touching command state.
        let open_orders = engine.query_open_orders("alice");
        assert_eq!(open_orders.len(), 1);
        let order = &open_orders[0];
        assert_eq!(order.order_id, 1);
        assert_eq!(order.open_quantity, 50);
        assert_eq!(order.status, OrderStatus::Open);
    }

    #[test]
    fn command_validation_prevents_invalid_state_transitions() {
        let engine = CqrsEngine::new();
        let pair = sample_pair();

        // Register an account and place an order.
        engine
            .execute_command(CqrsCommand::RegisterAccount {
                account_id: "bob".to_string(),
            })
            .unwrap();
        engine
            .execute_command(CqrsCommand::PlaceOrder {
                order_id: 7,
                account_id: "bob".to_string(),
                pair: pair.clone(),
                side: OrderSide::Sell,
                quantity: 10,
            })
            .unwrap();

        // Attempt to overfill the order.
        let err = engine
            .execute_command(CqrsCommand::RecordFill {
                order_id: 7,
                quantity: 15,
                price: 2000,
            })
            .unwrap_err();
        assert_eq!(
            err,
            CqrsError::InvalidFill {
                order_id: 7,
                attempted: 15,
                remaining: 10
            }
        );

        // Cancel the order and ensure further fills are rejected.
        engine
            .execute_command(CqrsCommand::CancelOrder {
                order_id: 7,
                reason: "user request".to_string(),
            })
            .unwrap();
        let err = engine
            .execute_command(CqrsCommand::RecordFill {
                order_id: 7,
                quantity: 1,
                price: 2000,
            })
            .unwrap_err();
        assert_eq!(err, CqrsError::OrderNotOpen(7));

        // Read side reflects the cancelled status.
        let order = engine.query_order(7).expect("order should exist");
        assert_eq!(order.status, OrderStatus::Cancelled);
        assert_eq!(order.open_quantity, 0);
    }

    #[test]
    fn projections_can_be_rebuilt_from_events() {
        let engine = CqrsEngine::new();
        let pair = sample_pair();

        engine
            .execute_command(CqrsCommand::RegisterAccount {
                account_id: "carol".to_string(),
            })
            .unwrap();
        engine
            .execute_command(CqrsCommand::PlaceOrder {
                order_id: 100,
                account_id: "carol".to_string(),
                pair: pair.clone(),
                side: OrderSide::Buy,
                quantity: 25,
            })
            .unwrap();
        engine
            .execute_command(CqrsCommand::RecordFill {
                order_id: 100,
                quantity: 10,
                price: 1900,
            })
            .unwrap();

        // Corrupt the read model intentionally to ensure replay works.
        {
            let mut read_model = engine.read_model.write().unwrap();
            read_model.reset();
        }
        engine.rebuild_read_model_from_events();

        let order = engine.query_order(100).expect("projection rebuilt");
        assert_eq!(order.status, OrderStatus::Open);
        assert_eq!(order.open_quantity, 15);
        assert_eq!(engine.query_volume_for_pair(&pair), 10);
        assert_eq!(engine.current_version(), 3);
    }

    #[test]
    fn multiple_fills_accumulate_volume_and_complete_order() {
        let engine = CqrsEngine::new();
        let pair = sample_pair();

        engine
            .execute_command(CqrsCommand::RegisterAccount {
                account_id: "dana".to_string(),
            })
            .unwrap();
        engine
            .execute_command(CqrsCommand::PlaceOrder {
                order_id: 55,
                account_id: "dana".to_string(),
                pair: pair.clone(),
                side: OrderSide::Sell,
                quantity: 12,
            })
            .unwrap();

        engine
            .execute_command(CqrsCommand::RecordFill {
                order_id: 55,
                quantity: 5,
                price: 2100,
            })
            .unwrap();
        engine
            .execute_command(CqrsCommand::RecordFill {
                order_id: 55,
                quantity: 7,
                price: 2100,
            })
            .unwrap();

        let order = engine.query_order(55).expect("order should exist");
        assert_eq!(order.status, OrderStatus::Filled);
        assert_eq!(order.open_quantity, 0);
        assert_eq!(order.filled_quantity, 12);
        assert_eq!(engine.query_volume_for_pair(&pair), 12);
        assert_eq!(engine.current_version(), 4);
    }
}
