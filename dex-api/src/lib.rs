//! API layer for the DEX-OS core engine
//!
//! This module provides HTTP API endpoints for interacting with the DEX.

pub mod auth;
pub mod challenge;
pub mod config;

pub use auth::Claims;
pub use challenge::ChallengeStore;
pub use config::Config;

use auth::{
    clamp_ttl, normalize_address, verify_wallet_signature, AuthManager, AuthRejection,
};
use challenge::ChallengeError;
use dex_core::{
    orderbook::OrderBook,
    types::{OrderId, Price, Quantity, Trade, TraderId},
};
use dex_db::DatabaseManager;
use futures_util::{SinkExt, StreamExt};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::{broadcast, RwLock};
use warp::{
    filters::body::BodyDeserializeError,
    http::StatusCode,
    reject::{MethodNotAllowed, MissingHeader},
    ws::{Message, WebSocket, Ws},
    Filter,
};

/// Shared state for the API
#[derive(Clone)]
pub struct ApiState {
    pub orderbook: Arc<RwLock<OrderBook>>,
    pub order_id_counter: Arc<AtomicU64>,
    pub trade_id_counter: Arc<AtomicU64>,
    pub database: Arc<DatabaseManager>,
    pub auth: Arc<AuthManager>,
    pub config: Config,
    pub wallet_challenges: Arc<ChallengeStore>,
    pub market_tx: broadcast::Sender<DepthSnapshot>,
}

/// Request to create a new order
#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub trader_id: TraderId,
    pub base_token: String,
    pub quote_token: String,
    pub side: String,
    pub order_type: String,
    pub price: Option<u64>,
    pub quantity: u64,
}

/// Response for order creation
#[derive(Serialize)]
pub struct CreateOrderResponse {
    pub order_id: OrderId,
    pub success: bool,
    pub message: Option<String>,
}

/// Get the best bid and ask prices
#[derive(Serialize)]
pub struct PriceResponse {
    pub best_bid: Option<u64>,
    pub best_ask: Option<u64>,
}

/// Response for trade information
#[derive(Serialize)]
pub struct TradeResponse {
    pub id: u64,
    pub maker_order_id: u64,
    pub taker_order_id: u64,
    pub base_token: String,
    pub quote_token: String,
    pub price: u64,
    pub quantity: u64,
    pub timestamp: u64,
}

impl From<Trade> for TradeResponse {
    fn from(trade: Trade) -> Self {
        Self {
            id: trade.id,
            maker_order_id: trade.maker_order_id,
            taker_order_id: trade.taker_order_id,
            base_token: trade.base_token,
            quote_token: trade.quote_token,
            price: trade.price,
            quantity: trade.quantity,
            timestamp: trade.timestamp,
        }
    }
}

/// Response for getting trades
#[derive(Serialize)]
pub struct GetTradesResponse {
    pub trades: Vec<TradeResponse>,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DepthLevel {
    pub price: Price,
    pub quantity: Quantity,
}

#[derive(Debug, Clone, Serialize)]
pub struct DepthSnapshot {
    pub bids: Vec<DepthLevel>,
    pub asks: Vec<DepthLevel>,
    pub best_bid: Option<Price>,
    pub best_ask: Option<Price>,
    pub timestamp: u64,
}

#[derive(Debug, Default, Deserialize)]
struct DepthQuery {
    levels: Option<usize>,
}

const DEFAULT_DEPTH_LEVELS: usize = 10;
const STREAM_DEPTH_LEVELS: usize = 20;
const MAX_DEPTH_LEVELS: usize = 100;

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_at: u64,
}

#[derive(Serialize)]
pub struct WalletChallengeResponse {
    pub challenge: String,
    pub expires_at: u64,
}

#[derive(Deserialize)]
struct SharedTokenRequest {
    trader_id: String,
    secret: String,
    #[serde(default)]
    ttl_seconds: Option<u64>,
    #[serde(default)]
    audience: Option<String>,
}

#[derive(Deserialize)]
struct WalletChallengeRequest {
    address: String,
}

#[derive(Deserialize)]
struct WalletTokenRequest {
    address: String,
    signature: String,
    #[serde(default)]
    ttl_seconds: Option<u64>,
    #[serde(default)]
    audience: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    code: &'static str,
    message: String,
}

/// Create the API routes
pub fn routes(
    state: ApiState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let orderbook = warp::path("orderbook");

    // Create order endpoint
    let create_order = orderbook
        .and(warp::path("orders"))
        .and(warp::post())
        .and(authenticated(state.clone()))
        .and(warp::body::content_length_limit(8 * 1024))
        .and(warp::body::json())
        .and_then(handle_create_order)
        .boxed();

    // Get prices endpoint
    let get_prices = orderbook
        .and(warp::path("prices"))
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_get_prices)
        .boxed();

    let get_depth = orderbook
        .and(warp::path("depth"))
        .and(warp::get())
        .and(with_state(state.clone()))
        .and(optional_depth_query())
        .and_then(handle_get_depth)
        .boxed();

    // Get trades for order endpoint
    let get_trades_for_order = orderbook
        .and(warp::path("orders"))
        .and(warp::path::param::<u64>())
        .and(warp::path("trades"))
        .and(warp::get())
        .and(authenticated(state.clone()))
        .and_then(handle_get_trades_for_order)
        .boxed();

    // Get trades for trader endpoint
    let get_trades_for_trader = orderbook
        .and(warp::path("traders"))
        .and(warp::path::param::<String>())
        .and(warp::path("trades"))
        .and(warp::get())
        .and(authenticated(state.clone()))
        .and_then(handle_get_trades_for_trader)
        .boxed();

    let depth_ws = warp::path("ws")
        .and(warp::path("depth"))
        .and(with_state(state.clone()))
        .and(optional_depth_query())
        .and(warp::ws())
        .and_then(handle_depth_ws)
        .boxed();

    let auth_endpoints = auth_routes(state.clone()).boxed();

    create_order
        .or(get_prices)
        .or(get_trades_for_order)
        .or(get_trades_for_trader)
        .or(get_depth)
        .or(depth_ws)
        .or(auth_endpoints)
        .recover(handle_rejection)
}

fn auth_routes(
    state: ApiState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let shared = warp::path("auth")
        .and(warp::path("token"))
        .and(warp::path("shared"))
        .and(warp::post())
        .and(with_state(state.clone()))
        .and(warp::body::content_length_limit(4 * 1024))
        .and(warp::body::json())
        .and_then(handle_shared_token);

    let challenge = warp::path("auth")
        .and(warp::path("challenge"))
        .and(warp::post())
        .and(with_state(state.clone()))
        .and(warp::body::content_length_limit(2 * 1024))
        .and(warp::body::json())
        .and_then(handle_wallet_challenge);

    let wallet_token = warp::path("auth")
        .and(warp::path("token"))
        .and(warp::path("wallet"))
        .and(warp::post())
        .and(with_state(state))
        .and(warp::body::content_length_limit(4 * 1024))
        .and(warp::body::json())
        .and_then(handle_wallet_token);

    shared.or(challenge).or(wallet_token)
}

/// Helper to pass state to handlers
fn with_state(state: ApiState) -> impl Filter<Extract = (ApiState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

fn authenticated(
    state: ApiState,
) -> impl Filter<Extract = (Claims, ApiState), Error = warp::Rejection> + Clone {
    warp::header::header::<String>("authorization")
        .and(with_state(state))
        .and_then(|auth_header: String, state: ApiState| async move {
            match state.auth.verify_bearer(&auth_header) {
                Ok(claims) => Ok((claims, state)),
                Err(err) => Err(warp::reject::custom(AuthRejection(err))),
            }
        })
        .untuple_one()
}

fn optional_depth_query() -> impl Filter<Extract = (Option<String>,), Error = warp::Rejection> + Clone {
    warp::query::raw().map(Some)
}

/// Handler for creating orders
async fn handle_create_order(
    claims: Claims,
    state: ApiState,
    req: CreateOrderRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    let validated = validation::validate_create_order(req)
        .map_err(|err| warp::reject::custom(ValidationRejection(err)))?;
    let order_id = state.order_id_counter.fetch_add(1, Ordering::Relaxed);
    let timestamp = current_unix_timestamp().map_err(|_| warp::reject::custom(InternalError))?;
    let order = validated.into_order(order_id, timestamp);
    let order_for_storage = order.clone();

    if claims.sub != order_for_storage.trader_id {
        return Ok(error_reply(
            "forbidden",
            "trader_id does not match authenticated subject",
            StatusCode::FORBIDDEN,
        ));
    }

    let mut orderbook = state.orderbook.write().await;
    let result = orderbook.add_order(order);
    drop(orderbook);

    let mut trades = match result {
        Ok(trades) => trades,
        Err(err) => {
            return Ok(error_reply(
                "order_book_error",
                err.to_string(),
                StatusCode::CONFLICT,
            ))
        }
    };

    if let Err(err) = state.database.save_order(&order_for_storage).await {
        eprintln!("failed to persist order {}: {}", order_id, err);
        return Ok(error_reply(
            "storage_error",
            "failed to persist order",
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let mut executed_trades = 0usize;
    for trade in trades.iter_mut() {
        executed_trades += 1;
        let trade_id = state.trade_id_counter.fetch_add(1, Ordering::Relaxed);
        trade.id = trade_id;
        if let Err(err) = state.database.save_trade(trade).await {
            eprintln!("failed to persist trade {}: {}", trade_id, err);
            return Ok(error_reply(
                "storage_error",
                "failed to persist trade",
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }

    let message = if executed_trades == 0 {
        None
    } else {
        Some(format!(
            "Order created and matched, {} trades executed",
            executed_trades
        ))
    };

    let response = CreateOrderResponse {
        order_id,
        success: true,
        message,
    };

    broadcast_depth_snapshot(&state).await;

    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::CREATED,
    ))
}

/// Handler for getting prices
async fn handle_get_prices(state: ApiState) -> Result<impl warp::Reply, warp::Rejection> {
    let orderbook = state.orderbook.read().await;
    let response = PriceResponse {
        best_bid: orderbook.best_bid(),
        best_ask: orderbook.best_ask(),
    };
    Ok(warp::reply::json(&response))
}

async fn handle_get_depth(
    state: ApiState,
    raw_query: Option<String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let levels = parse_depth_levels(raw_query);
    let snapshot = {
        let orderbook = state.orderbook.read().await;
        depth_snapshot(&orderbook, levels)
    };
    Ok(warp::reply::json(&snapshot))
}

async fn handle_depth_ws(
    state: ApiState,
    raw_query: Option<String>,
    ws: Ws,
) -> Result<impl warp::Reply, warp::Rejection> {
    let levels = parse_depth_levels(raw_query);
    Ok(ws.on_upgrade(move |socket| depth_ws_session(socket, state, levels)))
}

/// Handler for getting trades for an order
async fn handle_get_trades_for_order(
    order_id: u64,
    _claims: Claims,
    state: ApiState,
) -> Result<impl warp::Reply, warp::Rejection> {
    match state.database.get_trades_for_order(order_id).await {
        Ok(trades) => {
            let response = GetTradesResponse {
                trades: trades.into_iter().map(TradeResponse::from).collect(),
                success: true,
                message: None,
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::OK,
            ))
        }
        Err(err) => {
            eprintln!("failed to load trades for order {}: {}", order_id, err);
            Ok(error_reply(
                "storage_error",
                "failed to load trades",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

/// Handler for getting trades for a trader
async fn handle_get_trades_for_trader(
    trader_id: String,
    claims: Claims,
    state: ApiState,
) -> Result<impl warp::Reply, warp::Rejection> {
    if claims.sub != trader_id {
        return Ok(error_reply(
            "forbidden",
            "requested trader does not match authenticated subject",
            StatusCode::FORBIDDEN,
        ));
    }
    match state.database.get_trades_for_trader(&trader_id).await {
        Ok(trades) => {
            let response = GetTradesResponse {
                trades: trades.into_iter().map(TradeResponse::from).collect(),
                success: true,
                message: None,
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::OK,
            ))
        }
        Err(err) => {
            eprintln!("failed to load trades for trader {}: {}", trader_id, err);
            Ok(error_reply(
                "storage_error",
                "failed to load trades",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn handle_shared_token(
    state: ApiState,
    req: SharedTokenRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    if state.config.trader_secrets.is_empty() {
        return Ok(error_reply(
            "unauthorized",
            "shared secret token issuance is disabled",
            StatusCode::UNAUTHORIZED,
        ));
    }
    let secret = match state.config.trader_secrets.get(&req.trader_id) {
        Some(secret) => secret,
        None => {
            return Ok(error_reply(
                "unauthorized",
                "invalid trader credentials",
                StatusCode::UNAUTHORIZED,
            ))
        }
    };

    if *secret.expose_secret() != req.secret {
        return Ok(error_reply(
            "unauthorized",
            "invalid trader credentials",
            StatusCode::UNAUTHORIZED,
        ));
    }

    let ttl = clamp_ttl(
        req.ttl_seconds,
        state.config.jwt_default_ttl_seconds,
        state.config.jwt_max_ttl_seconds,
    );
    let issued = match state
        .auth
        .issue_token(req.trader_id.clone(), ttl, req.audience.clone())
    {
        Ok(token) => token,
        Err(err) => {
            eprintln!("failed to issue shared token: {}", err);
            return Ok(error_reply(
                "internal_error",
                "failed to issue token",
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let response = TokenResponse {
        token: issued.token,
        expires_at: issued.expires_at,
    };
    Ok(warp::reply::json(&response))
}

async fn handle_wallet_challenge(
    state: ApiState,
    req: WalletChallengeRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    let address = match normalize_address(&req.address) {
        Ok(addr) => addr,
        Err(_) => {
            return Ok(error_reply(
                "invalid_address",
                "wallet address must be a 0x-prefixed hex string",
                StatusCode::BAD_REQUEST,
            ))
        }
    };
    let issued = state.wallet_challenges.issue(&address).await;
    let response = WalletChallengeResponse {
        challenge: issued.challenge,
        expires_at: issued.expires_at,
    };
    Ok(warp::reply::json(&response))
}

async fn handle_wallet_token(
    state: ApiState,
    req: WalletTokenRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    let address = match normalize_address(&req.address) {
        Ok(addr) => addr,
        Err(_) => {
            return Ok(error_reply(
                "invalid_address",
                "wallet address must be a 0x-prefixed hex string",
                StatusCode::BAD_REQUEST,
            ))
        }
    };

    let message = match state.wallet_challenges.take(&address).await {
        Ok(message) => message,
        Err(err) => {
            let (code, status, msg) = match err {
                ChallengeError::Missing => (
                    "challenge_missing",
                    StatusCode::BAD_REQUEST,
                    "no pending challenge for this address",
                ),
                ChallengeError::Expired => (
                    "challenge_expired",
                    StatusCode::BAD_REQUEST,
                    "challenge expired, request a new one",
                ),
            };
            return Ok(error_reply(code, msg, status));
        }
    };

    if let Err(err) = verify_wallet_signature(&address, &message, &req.signature) {
        return Ok(error_reply(
            "invalid_signature",
            err.to_string(),
            StatusCode::UNAUTHORIZED,
        ));
    }

    let ttl = clamp_ttl(
        req.ttl_seconds,
        state.config.jwt_default_ttl_seconds,
        state.config.jwt_max_ttl_seconds,
    );

    let issued = match state
        .auth
        .issue_token(address.clone(), ttl, req.audience.clone())
    {
        Ok(token) => token,
        Err(err) => {
            eprintln!("failed to issue wallet token: {}", err);
            return Ok(error_reply(
                "internal_error",
                "failed to issue token",
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let response = TokenResponse {
        token: issued.token,
        expires_at: issued.expires_at,
    };
    Ok(warp::reply::json(&response))
}

async fn depth_ws_session(socket: WebSocket, state: ApiState, levels: usize) {
    let (mut sender, mut receiver) = socket.split();
    let mut subscriber = state.market_tx.subscribe();

    // Drain any client messages to detect disconnects
    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if msg.is_close() {
                break;
            }
        }
    });

    let initial_snapshot = {
        let orderbook = state.orderbook.read().await;
        depth_snapshot(&orderbook, levels)
    };

    if send_depth_message(&mut sender, &initial_snapshot, levels)
        .await
        .is_err()
    {
        return;
    }

    loop {
        match subscriber.recv().await {
            Ok(snapshot) => {
                if send_depth_message(&mut sender, &snapshot, levels)
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(_)) => {
                // Skip missed updates; loop continues to receive latest snapshot
                continue;
            }
            Err(_) => break,
        }
    }
}

async fn send_depth_message(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    snapshot: &DepthSnapshot,
    levels: usize,
) -> Result<(), warp::Error> {
    let mut payload = snapshot.clone();
    payload.bids.truncate(levels);
    payload.asks.truncate(levels);
    let text = match serde_json::to_string(&payload) {
        Ok(text) => text,
        Err(_) => return Ok(()),
    };
    sender.send(Message::text(text)).await
}

fn clamp_depth_levels(levels: Option<usize>) -> usize {
    let requested = levels.unwrap_or(DEFAULT_DEPTH_LEVELS);
    requested.max(1).min(MAX_DEPTH_LEVELS)
}

fn parse_depth_levels(raw: Option<String>) -> usize {
    if let Some(raw) = raw {
        for pair in raw.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                if key == "levels" {
                    if let Ok(parsed) = value.parse::<usize>() {
                        return clamp_depth_levels(Some(parsed));
                    }
                }
            }
        }
    }
    clamp_depth_levels(None)
}

fn depth_snapshot(orderbook: &OrderBook, levels: usize) -> DepthSnapshot {
    let best_bid = orderbook.best_bid();
    let best_ask = orderbook.best_ask();
    let bids = orderbook
        .bids
        .iter()
        .rev()
        .take(levels)
        .map(|(&price, level)| DepthLevel {
            price,
            quantity: level.total_quantity,
        })
        .collect();
    let asks = orderbook
        .asks
        .iter()
        .take(levels)
        .map(|(&price, level)| DepthLevel {
            price,
            quantity: level.total_quantity,
        })
        .collect();
    let timestamp = current_unix_timestamp().unwrap_or_default();
    DepthSnapshot {
        bids,
        asks,
        best_bid,
        best_ask,
        timestamp,
    }
}

async fn broadcast_depth_snapshot(state: &ApiState) {
    let snapshot = {
        let orderbook = state.orderbook.read().await;
        depth_snapshot(&orderbook, STREAM_DEPTH_LEVELS)
    };
    let _ = state.market_tx.send(snapshot);
}

async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    if let Some(auth) = err.find::<AuthRejection>() {
        return Ok(error_reply(
            "unauthorized",
            auth.0.to_string(),
            StatusCode::UNAUTHORIZED,
        ));
    }

    if let Some(_missing) = err.find::<MissingHeader>() {
        return Ok(error_reply(
            "unauthorized",
            "authorization header is required",
            StatusCode::UNAUTHORIZED,
        ));
    }

    if let Some(validation) = err.find::<ValidationRejection>() {
        return Ok(error_reply(
            "validation_error",
            validation.0.to_string(),
            StatusCode::BAD_REQUEST,
        ));
    }

    if let Some(body_err) = err.find::<BodyDeserializeError>() {
        return Ok(error_reply(
            "invalid_payload",
            format!("invalid request body: {}", body_err),
            StatusCode::BAD_REQUEST,
        ));
    }

    if err.is_not_found() {
        return Ok(error_reply(
            "not_found",
            "endpoint not found",
            StatusCode::NOT_FOUND,
        ));
    }

    if let Some(_) = err.find::<MethodNotAllowed>() {
        return Ok(error_reply(
            "method_not_allowed",
            "HTTP method not allowed",
            StatusCode::METHOD_NOT_ALLOWED,
        ));
    }

    if let Some(_) = err.find::<InternalError>() {
        return Ok(error_reply(
            "internal_error",
            "internal server error",
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    eprintln!("unhandled rejection: {:?}", err);
    Ok(error_reply(
        "internal_error",
        "internal server error",
        StatusCode::INTERNAL_SERVER_ERROR,
    ))
}

fn error_reply(
    code: &'static str,
    message: impl Into<String>,
    status: StatusCode,
) -> warp::reply::WithStatus<warp::reply::Json> {
    let body = ErrorResponse {
        code,
        message: message.into(),
    };
    warp::reply::with_status(warp::reply::json(&body), status)
}

fn current_unix_timestamp() -> Result<u64, std::time::SystemTimeError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
}

#[derive(Debug)]
struct ValidationRejection(validation::ValidationError);

impl warp::reject::Reject for ValidationRejection {}

#[derive(Debug)]
struct InternalError;

impl warp::reject::Reject for InternalError {}

mod validation {
    use super::CreateOrderRequest;
    use dex_core::types::{Order, OrderId, OrderSide, OrderType, TraderId, TradingPair};
    use lazy_static::lazy_static;
    use regex::Regex;
    use thiserror::Error;

    /// Result of validating a `CreateOrderRequest`.
    pub struct ValidatedCreateOrder {
        pub trader_id: TraderId,
        pub pair: TradingPair,
        pub side: OrderSide,
        pub order_type: OrderType,
        pub price: Option<u64>,
        pub quantity: u64,
    }

    impl ValidatedCreateOrder {
        /// Convert the validated payload into a fully formed `Order`.
        pub fn into_order(self, order_id: OrderId, timestamp: u64) -> Order {
            Order {
                id: order_id,
                trader_id: self.trader_id,
                pair: self.pair,
                side: self.side,
                order_type: self.order_type,
                price: self.price,
                quantity: self.quantity,
                timestamp,
            }
        }
    }

    #[derive(Debug, Error)]
    pub enum ValidationError {
        #[error("trader_id must be between 3 and 64 visible characters")]
        InvalidTraderId,
        #[error("base_token must be 2-16 characters from [A-Za-z0-9_-]")]
        InvalidBaseToken,
        #[error("quote_token must be 2-16 characters from [A-Za-z0-9_-]")]
        InvalidQuoteToken,
        #[error("base_token and quote_token must differ")]
        IdenticalTokens,
        #[error("quantity must be greater than zero")]
        InvalidQuantity,
        #[error("limit orders require a positive price")]
        MissingLimitPrice,
        #[error("price must be greater than zero")]
        InvalidPrice,
        #[error("side must be `buy` or `sell`")]
        InvalidSide,
        #[error("order_type must be `market` or `limit`")]
        InvalidOrderType,
    }

    /// Validate a create order request.
    pub fn validate_create_order(
        req: CreateOrderRequest,
    ) -> Result<ValidatedCreateOrder, ValidationError> {
        let trader_id = normalize_trader_id(&req.trader_id)?;
        let base_token = normalize_token(&req.base_token, TokenRole::Base)?;
        let quote_token = normalize_token(&req.quote_token, TokenRole::Quote)?;

        if base_token == quote_token {
            return Err(ValidationError::IdenticalTokens);
        }

        let side = parse_side(&req.side)?;
        let order_type = parse_order_type(&req.order_type)?;

        if req.quantity == 0 {
            return Err(ValidationError::InvalidQuantity);
        }

        let price = match order_type {
            OrderType::Limit => match req.price {
                Some(p) if p > 0 => Some(p),
                Some(_) => return Err(ValidationError::InvalidPrice),
                None => return Err(ValidationError::MissingLimitPrice),
            },
            OrderType::Market => {
                if let Some(p) = req.price {
                    if p == 0 {
                        return Err(ValidationError::InvalidPrice);
                    }
                }
                None
            }
        };

        Ok(ValidatedCreateOrder {
            trader_id,
            pair: TradingPair {
                base: base_token,
                quote: quote_token,
            },
            side,
            order_type,
            price,
            quantity: req.quantity,
        })
    }

    enum TokenRole {
        Base,
        Quote,
    }

    fn normalize_trader_id(raw: &str) -> Result<TraderId, ValidationError> {
        let trimmed = raw.trim();
        if trimmed.len() < 3 || trimmed.len() > 64 || !trimmed.is_ascii() {
            return Err(ValidationError::InvalidTraderId);
        }
        Ok(trimmed.to_string())
    }

    fn normalize_token(raw: &str, role: TokenRole) -> Result<String, ValidationError> {
        lazy_static! {
            static ref TOKEN_RE: Regex =
                Regex::new(r"^[A-Za-z0-9_-]{2,16}$").expect("valid token regex");
        }

        let trimmed = raw.trim();
        if !TOKEN_RE.is_match(trimmed) {
            return Err(match role {
                TokenRole::Base => ValidationError::InvalidBaseToken,
                TokenRole::Quote => ValidationError::InvalidQuoteToken,
            });
        }
        Ok(trimmed.to_string())
    }

    fn parse_side(raw: &str) -> Result<OrderSide, ValidationError> {
        match raw.to_ascii_lowercase().as_str() {
            "buy" => Ok(OrderSide::Buy),
            "sell" => Ok(OrderSide::Sell),
            _ => Err(ValidationError::InvalidSide),
        }
    }

    fn parse_order_type(raw: &str) -> Result<OrderType, ValidationError> {
        match raw.to_ascii_lowercase().as_str() {
            "limit" => Ok(OrderType::Limit),
            "market" => Ok(OrderType::Market),
            _ => Err(ValidationError::InvalidOrderType),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::CreateOrderRequest;

        fn base_request() -> CreateOrderRequest {
            CreateOrderRequest {
                trader_id: "alice".into(),
                base_token: "ETH".into(),
                quote_token: "USDC".into(),
                side: "buy".into(),
                order_type: "limit".into(),
                price: Some(1000),
                quantity: 10,
            }
        }

        #[test]
        fn validates_happy_path() {
            let req = base_request();
            let validated = validate_create_order(req).expect("valid request");
            assert_eq!(validated.trader_id, "alice");
            assert_eq!(validated.pair.base, "ETH");
            assert_eq!(validated.pair.quote, "USDC");
            assert!(validated.price.is_some());
        }

        #[test]
        fn rejects_identical_tokens() {
            let mut req = base_request();
            req.quote_token = "ETH".into();
            let err = validate_create_order(req).unwrap_err();
            assert!(matches!(err, ValidationError::IdenticalTokens));
        }

        #[test]
        fn requires_price_for_limit_order() {
            let mut req = base_request();
            req.price = None;
            let err = validate_create_order(req).unwrap_err();
            assert!(matches!(err, ValidationError::MissingLimitPrice));
        }

        #[test]
        fn rejects_zero_quantity() {
            let mut req = base_request();
            req.quantity = 0;
            let err = validate_create_order(req).unwrap_err();
            assert!(matches!(err, ValidationError::InvalidQuantity));
        }

        #[test]
        fn rejects_bad_token_chars() {
            let mut req = base_request();
            req.base_token = "E TH".into();
            let err = validate_create_order(req).unwrap_err();
            assert!(matches!(err, ValidationError::InvalidBaseToken));
        }
    }

    #[cfg(test)]
    mod auth_filter_tests {
        use crate::{
            authenticated,
            handle_rejection,
            auth::AuthManager,
            challenge::ChallengeStore,
            ApiState,
            Claims,
            Config,
        };
        use dex_core::orderbook::OrderBook;
        use dex_db::DatabaseManager;
        use jsonwebtoken::{encode, EncodingKey, Header};
        use secrecy::{ExposeSecret, SecretString};
        use std::{
            collections::HashMap,
            sync::{atomic::AtomicU64, Arc},
            time::{SystemTime, UNIX_EPOCH},
        };
        use tokio::sync::{broadcast, RwLock};
        use warp::http::StatusCode;
        use warp::Filter;

        const TEST_DB_URL: &str = "postgres://user:password@localhost/test";
        const TEST_SECRET: &str = "super-secret-signing-key";

        #[tokio::test]
        async fn missing_token_returns_401() {
            let state = test_state();
            let filter = protected_filter(state);

            let response = warp::test::request().reply(&filter).await;

            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }

        #[tokio::test]
        async fn invalid_token_returns_401() {
            let state = test_state();
            let filter = protected_filter(state);

            let response = warp::test::request()
                .header("authorization", "Bearer totally-invalid")
                .reply(&filter)
                .await;

            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }

        #[tokio::test]
        async fn expired_token_returns_401() {
            let secret = SecretString::from(TEST_SECRET.to_string());
            let token = build_token(&secret, -60);
            let state = test_state();
            let filter = protected_filter(state);

            let response = warp::test::request()
                .header("authorization", format!("Bearer {}", token))
                .reply(&filter)
                .await;

            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }

        #[tokio::test]
        async fn valid_token_returns_200() {
            let secret = SecretString::from(TEST_SECRET.to_string());
            let token = build_token(&secret, 300);
            let state = test_state();
            let filter = protected_filter(state);

            let response = warp::test::request()
                .header("authorization", format!("Bearer {}", token))
                .reply(&filter)
                .await;

            assert_eq!(response.status(), StatusCode::OK);
        }

        fn protected_filter(
            state: ApiState,
        ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
            authenticated(state)
                .and_then(|claims: Claims, _state: ApiState| async move {
                    let reply =
                        warp::reply::with_status(warp::reply::json(&claims.sub), StatusCode::OK);
                    Ok::<_, warp::Rejection>(reply)
                })
                .recover(handle_rejection)
        }

        fn test_state() -> ApiState {
            let secret = SecretString::from(TEST_SECRET.to_string());
            let auth = Arc::new(AuthManager::new(&secret, "test-issuer"));
            let mut trader_secrets = HashMap::new();
            trader_secrets.insert("alice".into(), SecretString::from("shared-secret"));
            let config = Config {
                database_url: SecretString::from(TEST_DB_URL.to_string()),
                jwt_secret: secret.clone(),
                jwt_issuer: "test-issuer".into(),
                jwt_default_ttl_seconds: 900,
                jwt_max_ttl_seconds: 3600,
                wallet_challenge_ttl_seconds: 300,
                trader_secrets,
                server_port: 3030,
            };
            let (market_tx, _) = broadcast::channel(16);

            ApiState {
                orderbook: Arc::new(RwLock::new(OrderBook::new())),
                order_id_counter: Arc::new(AtomicU64::new(1)),
                trade_id_counter: Arc::new(AtomicU64::new(1)),
                database: Arc::new(
                    DatabaseManager::connect_lazy(TEST_DB_URL).expect("lazy db pool"),
                ),
                auth,
                config,
                wallet_challenges: Arc::new(ChallengeStore::new(300)),
                market_tx,
            }
        }

        fn build_token(secret: &SecretString, offset_seconds: i64) -> String {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards")
                .as_secs() as i64;
            let claims = Claims {
                sub: "alice".into(),
                exp: (now + offset_seconds) as usize,
                aud: None,
                iss: None,
                iat: None,
            };
            encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(secret.expose_secret().as_bytes()),
            )
            .expect("token encoding")
        }
    }
}
