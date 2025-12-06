use dex_core::input_validation::{InputField, InputValidationError, InputValidator, Normalization, ValidationRule};

#[test]
fn security_manager_logs_invalid_inputs() {
    let validator = InputValidator::new();
    let err = validator
        .validate(InputField::TraderId, "bad value with $$")
        .expect_err("invalid trader id should be rejected");

    // The input fails the pattern match because it contains spaces and $ characters
    assert!(matches!(err, InputValidationError::PatternMismatch { .. }));
}

#[test]
fn security_manager_supports_custom_rules() {
    let mut validator = InputValidator::new();
    let rule = ValidationRule::new(
        "bridge_channel",
        r"^chan-[0-9]{3}$",
        16,
        Normalization::Trim,
        "Bridge channel identifier",
    )
    .expect("custom rule should compile");

    validator
        .register_rule(rule)
        .expect("custom rule should register");

    let validated = validator
        .validate(
            InputField::Custom("bridge_channel".to_string()),
            "  chan-123 ",
        )
        .expect("custom rule should validate");
    assert_eq!(validated.value, "chan-123");
}