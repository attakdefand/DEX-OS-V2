# Rule Base Task Implementation Guide

This document demonstrates how to implement tasks following the rules defined in RULES.md.

## Task: Implement Order Cancellation Feature

### 1. Identify Relevant RULES.md Sections

For implementing an order cancellation feature, we need to reference:

- `@RULES.md ##Code Organization` - Module structure
- `@RULES.md ##Coding Standards` - Rust conventions
- `@RULES.md ##Testing` - Unit testing requirements
- `@RULES.md ##Security Practices` - Input validation
- `@RULES.md ##Feature Implementation Priority` - Check priority level

### 2. Check Priority Level

From DEX-OS-V1.csv:
```
4,Core Trading,Orderbook,Orderbook,Cancellation Mechanisms,Order Cancellation,High
```

This is a Priority 4 feature, so we need to ensure all Priority 1, 2, and 3 features are complete.

### 3. Implementation Plan

Following guidelines from RULES.md:

1. **Code Organization**
   - Implement in dex-core/src/orderbook.rs
   - Add public method `cancel_order`
   - Follow existing module structure

2. **Coding Standards**
   - Use descriptive function name (`cancel_order`)
   - Return Result type for error handling
   - Add documentation comments
   - Use immutable data structures where possible

3. **Error Handling**
   - Use thiserror crate for custom errors
   - Handle cases where order doesn't exist
   - Provide meaningful error messages

4. **Security Practices**
   - Validate order ID input
   - Ensure only order owner can cancel
   - Implement proper access controls

### 4. Implementation Example

```rust
/// Cancel an existing order
/// 
/// # Arguments
/// * `order_id` - The ID of the order to cancel
/// 
/// # Returns
/// * `Ok(Order)` - The cancelled order
/// * `Err(OrderBookError)` - If the order cannot be cancelled
/// 
/// This implements the Priority 4 feature from DEX-OS-V1.csv:
/// "Core Trading,Orderbook,Orderbook,Cancellation Mechanisms,Order Cancellation,High"
pub fn cancel_order(&mut self, order_id: OrderId) -> Result<Order, OrderBookError> {
    // Validate input
    if order_id <= 0 {
        return Err(OrderBookError::InvalidOrderId);
    }
    
    // Check if order exists
    let order = self.orders.get(&order_id)
        .ok_or(OrderBookError::OrderNotFound)?;
    
    // Remove from bids/asks
    match order.side {
        OrderSide::Buy => self.remove_bid(order_id, order.price.unwrap_or(0)),
        OrderSide::Sell => self.remove_ask(order_id, order.price.unwrap_or(0)),
    }
    
    // Remove from orders map
    let removed_order = self.orders.remove(&order_id)
        .ok_or(OrderBookError::OrderNotFound)?;
    
    Ok(removed_order)
}
```

### 5. Testing Requirements

Following `@RULES.md ##Testing` guidelines:

```rust
#[test]
fn test_cancel_order() {
    let mut orderbook = OrderBook::new();
    let pair = TradingPair {
        base: "BTC".to_string(),
        quote: "USD".to_string(),
    };

    let order = Order {
        id: 1,
        trader_id: "trader1".to_string(),
        pair,
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        price: Some(50000),
        quantity: 100,
        timestamp: 1234567890,
    };

    // Add order
    assert!(orderbook.add_order(order).is_ok());
    assert_eq!(orderbook.orders.len(), 1);

    // Cancel order
    let cancelled = orderbook.cancel_order(1);
    assert!(cancelled.is_ok());
    assert_eq!(orderbook.orders.len(), 0);
}

#[test]
fn test_cancel_nonexistent_order() {
    let mut orderbook = OrderBook::new();
    
    // Try to cancel non-existent order
    let result = orderbook.cancel_order(999);
    assert!(result.is_err());
    match result.unwrap_err() {
        OrderBookError::OrderNotFound => (),
        _ => panic!("Expected OrderNotFound error"),
    }
}
```

### 6. Documentation

Following `@RULES.md ##Documentation` guidelines:

- Add documentation comments to all public functions
- Explain purpose and usage
- Include examples when appropriate
- Document error conditions

### 7. Commit Message

Following `@RULES.md ##Git Workflow` guidelines:

```
Implement order cancellation mechanism (Priority 4 feature)

- Add cancel_order method to OrderBook
- This implements the Priority 4 feature from DEX-OS-V1.csv:
  "Core Trading,Orderbook,Orderbook,Cancellation Mechanisms,Order Cancellation,High"
- Include comprehensive unit tests for success and error cases
- Add OrderBookError::InvalidOrderId variant
- Update documentation with usage examples

The implementation follows the guidelines in RULES.md for:
- Code Organization
- Coding Standards
- Error Handling
- Testing
- Security Practices
```

This example demonstrates how to properly reference and apply the rules from RULES.md when implementing features.