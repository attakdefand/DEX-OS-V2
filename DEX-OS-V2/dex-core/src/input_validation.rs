//! Regex-based input validation for application-layer protection
//!
//! Implements the Priority 3 feature from `DEX-OS-V2.csv`:
//! `3,Application,Application,Application,Regex Validation,Input Protection,High`
//!
//! The validator provides:
//! - Allow-listed regex rules for common DEX inputs (trader ids, symbols, addresses, ids).
//! - Optional normalization (trim, lower/upper casing) before validation.
//! - A lightweight denylist to catch obvious injection attempts alongside the allowlist.
//! - Custom rule registration for downstream components.

use regex::Regex;
use std::{collections::HashMap, fmt};
use thiserror::Error;

/// Fields with built-in validation rules
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputField {
    TraderId,
    TokenSymbol,
    WalletAddress,
    OrderId,
    MarketSymbol,
    MetadataTag,
    Custom(String),
}

impl InputField {
    fn key(&self) -> String {
        match self {
            InputField::TraderId => "trader_id".to_string(),
            InputField::TokenSymbol => "token_symbol".to_string(),
            InputField::WalletAddress => "wallet_address".to_string(),
            InputField::OrderId => "order_id".to_string(),
            InputField::MarketSymbol => "market_symbol".to_string(),
            InputField::MetadataTag => "metadata_tag".to_string(),
            InputField::Custom(name) => name.clone(),
        }
    }
}

impl fmt::Display for InputField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputField::Custom(name) => write!(f, "{name}"),
            _ => write!(f, "{}", self.key()),
        }
    }
}

/// How to normalize a string before validating it
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Normalization {
    None,
    Trim,
    LowercaseTrim,
    UppercaseTrim,
}

impl Normalization {
    fn apply(self, value: &str) -> String {
        match self {
            Normalization::None => value.to_string(),
            Normalization::Trim => value.trim().to_string(),
            Normalization::LowercaseTrim => value.trim().to_ascii_lowercase(),
            Normalization::UppercaseTrim => value.trim().to_ascii_uppercase(),
        }
    }
}

/// Single regex-based rule with optional normalization and length cap
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub name: String,
    pub description: String,
    pub pattern: String,
    pub max_length: usize,
    pub normalization: Normalization,
    regex: Regex,
}

impl ValidationRule {
    /// Build a new validation rule from a regex pattern
    pub fn new(
        name: impl Into<String>,
        pattern: impl Into<String>,
        max_length: usize,
        normalization: Normalization,
        description: impl Into<String>,
    ) -> Result<Self, InputValidationError> {
        let pattern = pattern.into();
        let regex = Regex::new(&pattern).map_err(|err| InputValidationError::InvalidPattern {
            pattern: pattern.clone(),
            error: err.to_string(),
        })?;

        Ok(Self {
            name: name.into(),
            description: description.into(),
            pattern,
            max_length,
            normalization,
            regex,
        })
    }

    fn normalize(&self, value: &str) -> String {
        self.normalization.apply(value)
    }
}

/// Result of validating user input
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedInput {
    /// Name of the field/rule validated
    pub field: String,
    /// Normalized, safe value
    pub value: String,
    /// Rule that was applied
    pub rule: String,
}

/// Errors that can occur during input validation
#[derive(Debug, Error, PartialEq, Eq)]
pub enum InputValidationError {
    #[error("no validation rule registered for {field}")]
    UnknownRule { field: String },
    #[error("input for {field} exceeds max length {max_length}")]
    TooLong { field: String, max_length: usize },
    #[error("input for {field} does not match allowed pattern {pattern}")]
    PatternMismatch { field: String, pattern: String },
    #[error("input for {field} matches denied pattern {pattern}")]
    Denylisted { field: String, pattern: String },
    #[error("invalid regex pattern {pattern}: {error}")]
    InvalidPattern { pattern: String, error: String },
}

/// Regex-driven validator with allowlist + denylist protection
#[derive(Debug, Clone)]
pub struct InputValidator {
    allowlist: HashMap<String, ValidationRule>,
    denylist: Vec<Regex>,
}

impl InputValidator {
    /// Construct with default DEX rules and a small denylist
    pub fn new() -> Self {
        let mut allowlist = HashMap::new();
        for rule in default_rules() {
            allowlist.insert(rule.name.clone(), rule);
        }

        // Guard rails for obvious injection attempts
        let denylist = vec![
            Regex::new("(?i)<\\s*script").expect("valid denylist regex"),
            Regex::new("(?i)union\\s+select").expect("valid denylist regex"),
            Regex::new("(?i)or\\s+1=1").expect("valid denylist regex"),
            Regex::new("\\.\\./").expect("valid denylist regex"),
        ];

        Self { allowlist, denylist }
    }

    /// Register or override a validation rule
    pub fn register_rule(&mut self, rule: ValidationRule) -> Result<(), InputValidationError> {
        self.allowlist.insert(rule.name.clone(), rule);
        Ok(())
    }

    /// Validate a value for a given field
    pub fn validate(
        &self,
        field: InputField,
        value: &str,
    ) -> Result<ValidatedInput, InputValidationError> {
        let key = field.key();
        let rule = self
            .allowlist
            .get(&key)
            .ok_or_else(|| InputValidationError::UnknownRule { field: key.clone() })?;

        let normalized = rule.normalize(value);

        if normalized.len() > rule.max_length {
            return Err(InputValidationError::TooLong {
                field: key,
                max_length: rule.max_length,
            });
        }

        if !rule.regex.is_match(&normalized) {
            return Err(InputValidationError::PatternMismatch {
                field: rule.name.clone(),
                pattern: rule.pattern.clone(),
            });
        }

        if let Some(pattern) = self.matches_denylist(&normalized) {
            return Err(InputValidationError::Denylisted {
                field: rule.name.clone(),
                pattern,
            });
        }

        Ok(ValidatedInput {
            field: key,
            value: normalized,
            rule: rule.name.clone(),
        })
    }

    fn matches_denylist(&self, value: &str) -> Option<String> {
        self.denylist
            .iter()
            .find(|pat| pat.is_match(value))
            .map(|pat| pat.as_str().to_string())
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

fn default_rules() -> Vec<ValidationRule> {
    vec![
        ValidationRule::new(
            "trader_id",
            r"^[A-Za-z0-9._-]{3,64}$",
            64,
            Normalization::Trim,
            "Human trader identifiers trimmed to ASCII word chars",
        )
        .expect("valid trader_id rule"),
        ValidationRule::new(
            "token_symbol",
            r"^[A-Z0-9_-]{2,16}$",
            16,
            Normalization::UppercaseTrim,
            "Token symbols enforced to uppercase allowlist",
        )
        .expect("valid token_symbol rule"),
        ValidationRule::new(
            "wallet_address",
            r"^0x[a-f0-9]{40}$",
            42,
            Normalization::LowercaseTrim,
            "EVM-style addresses (0x + 40 hex chars)",
        )
        .expect("valid wallet_address rule"),
        ValidationRule::new(
            "order_id",
            r"^[0-9]{1,20}$",
            20,
            Normalization::Trim,
            "Order identifiers are numeric and bounded",
        )
        .expect("valid order_id rule"),
        ValidationRule::new(
            "market_symbol",
            r"^[A-Z0-9]{2,16}/[A-Z0-9]{2,16}$",
            35,
            Normalization::UppercaseTrim,
            "Market symbols in BASE/QUOTE format",
        )
        .expect("valid market_symbol rule"),
        ValidationRule::new(
            "metadata_tag",
            r"^[A-Za-z0-9][A-Za-z0-9 _-]{0,63}$",
            64,
            Normalization::Trim,
            "Safe metadata tags for dashboards and logs",
        )
        .expect("valid metadata_tag rule"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_default_fields_and_normalization() {
        let validator = InputValidator::new();

        let trader = validator
            .validate(InputField::TraderId, "   alice_01 ")
            .expect("trader id valid");
        assert_eq!(trader.value, "alice_01");

        let token = validator
            .validate(InputField::TokenSymbol, "eth")
            .expect("token valid");
        assert_eq!(token.value, "ETH");

        let address = validator
            .validate(InputField::WalletAddress, "0xDeaDbeef00000000000000000000000000000000")
            .expect("address valid");
        assert_eq!(address.value, "0xdeadbeef00000000000000000000000000000000");
    }

    #[test]
    fn rejects_injection_like_payloads() {
        let validator = InputValidator::new();
        let err = validator
            .validate(
                InputField::TraderId,
                "alice<script>alert(1)</script>",
            )
            .expect_err("script payload should be denied");

        assert!(matches!(err, InputValidationError::Denylisted { .. }));
    }

    #[test]
    fn allows_custom_rules() {
        let mut validator = InputValidator::new();
        let rollup_rule = ValidationRule::new(
            "rollup_id",
            r"^rollup_[a-z0-9]{4}$",
            16,
            Normalization::LowercaseTrim,
            "Custom rollup identifier",
        )
        .unwrap();

        validator.register_rule(rollup_rule).unwrap();

        let validated = validator
            .validate(InputField::Custom("rollup_id".to_string()), "  Rollup_ab12 ")
            .expect("custom rule should be applied");

        assert_eq!(validated.value, "rollup_ab12");
    }
}
