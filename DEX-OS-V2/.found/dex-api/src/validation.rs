//! Request validation logic for the DEX API.
//!
//! Harden order handling by validating incoming payloads before they reach
//! business logic in `dex-core`.

use crate::CreateOrderRequest;
use dex_core::types::{OrderSide, OrderType, TraderId, TradingPair};
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
pub fn validate_create_order(req: CreateOrderRequest) -> Result<ValidatedCreateOrder, ValidationError> {
    let trader_id = normalize_trader_id(&req.trader_id)?;
    let base_token = normalize_token(&req.base_token, TokenRole::Base)?;
    let quote_token = normalize_token(&req.quote_token, TokenRole::Quote)?;

    if base_token == quote_token {
        return Err(ValidationError::IdenticalTokens);
    }

    let side = parse_side(&req.side)?;
    let order_type = parse_order_type(&req.order_type)?;

    let quantity = req
        .quantity
        .checked_sub(0)
        .filter(|q| *q > 0)
        .ok_or(ValidationError::InvalidQuantity)?;

    let price = match req.price {
        Some(p) if p == 0 => return Err(ValidationError::InvalidPrice),
        Some(p) => Some(p),
        None => None,
    };

    if matches!(order_type, OrderType::Limit) && price.is_none() {
        return Err(ValidationError::MissingLimitPrice);
    }

    Ok(ValidatedCreateOrder {
        trader_id,
        pair: TradingPair {
            base: base_token,
            quote: quote_token,
        },
        side,
        order_type,
        price,
        quantity,
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
        static ref TOKEN_RE: Regex = Regex::new(r"^[A-Za-z0-9_-]{2,16}$").expect("valid token regex");
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
