# DEX-OS Feature Implementation Walkthrough

This document provides a step-by-step guide for implementing features from DEX-OS-V1.csv while following the guidelines in RULES.md.

## Practical Application Process

When working on any feature from DEX-OS-V1.csv, follow these steps:

### 1. Check Priority Level
Determine the priority level of your feature from DEX-OS-V1.csv:
- **Priority 1**: Core functionality that must be implemented first
- **Priority 2**: Important features that build upon core functionality
- **Priority 3**: Useful enhancements and additional capabilities
- **Priority 4**: Advanced features and integrations
- **Priority 5**: Infrastructure and supporting components

### 2. Reference Required DSAs
Use exactly the data structures and algorithms specified in DEX-OS-V1.csv for your feature. Each feature entry specifies the required DSA that must be used.

### 3. Apply RULES.md Guidelines
Follow these key guidelines from RULES.md:

- **Coding Standards**: Follow Rust coding conventions and naming standards
- **Error Handling**: Use `Result` and `Option` types properly, avoid `unwrap()`
- **Testing**: Write comprehensive unit tests covering happy path and error cases
- **Documentation**: Document all public functions, structs, and traits

### 4. Verify Compliance
Ensure your implementation follows all relevant sections of RULES.md:
- Code organization
- Security practices
- Performance considerations
- Testing requirements

### 5. Document References
Include references to both DEX-OS-V1.csv and RULES.md in your commit messages and code comments.

## Hands-On Example: Implementing Order ID Lookup (Priority 2 Feature)

Let's walk through implementing a Priority 2 feature from DEX-OS-V1.csv:

```
2,Core Trading,Orderbook,Orderbook,Hash Map,Order ID Lookup,Medium
```

### Step 1: Check Priority Level
This is a Priority 2 feature, which means:
- All Priority 1 features must be complete
- It builds upon core Orderbook functionality

### Step 2: Reference Required DSAs
The feature requires using a "Hash Map" for "Order ID Lookup".

### Step 3: Apply RULES.md Guidelines

#### Coding Standards
```rust
// Follow naming conventions from @RULES.md ##Coding Standards
impl OrderBook {
    pub fn get_order_by_id(&self, order_id: u64) -> Option<&Order> {
        self.orders.get(&order_id)
    }
}
```

#### Error Handling
```rust
// Use Result/Option types as per @RULES.md ##Error Handling Principles
#[derive(Debug)]
pub enum OrderBookError {
    OrderNotFound,
    InvalidOrderId,
}

impl OrderBook {
    pub fn get_order_by_id(&self, order_id: u64) -> Result<&Order, OrderBookError> {
        self.orders.get(&order_id).ok_or(OrderBookError::OrderNotFound)
    }
}
```

#### Testing
```rust
// Comprehensive testing as per @RULES.md ##Testing
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_order_by_id_success() {
        let mut orderbook = OrderBook::new();
        let order = Order::new(1, OrderSide::Buy, 100.0, 10.0);
        orderbook.add_order(order.clone()).unwrap();
        
        let retrieved = orderbook.get_order_by_id(1).unwrap();
        assert_eq!(retrieved.id, 1);
    }

    #[test]
    fn test_get_order_by_id_not_found() {
        let orderbook = OrderBook::new();
        
        let result = orderbook.get_order_by_id(999);
        assert!(matches!(result, Err(OrderBookError::OrderNotFound)));
    }
}
```

#### Documentation
```rust
/// Retrieves an order by its ID
/// 
/// # Arguments
/// * `order_id` - The unique identifier of the order to retrieve
/// 
/// # Returns
/// * `Ok(&Order)` - Reference to the order if found
/// * `Err(OrderBookError::OrderNotFound)` - If no order exists with the given ID
/// 
/// This implements the Priority 2 feature from DEX-OS-V1.csv:
/// "Core Trading,Orderbook,Orderbook,Hash Map,Order ID Lookup,Medium"
/// 
/// Follows guidelines from @RULES.md ##Coding Standards and ##Error Handling
pub fn get_order_by_id(&self, order_id: u64) -> Result<&Order, OrderBookError> {
    self.orders.get(&order_id).ok_or(OrderBookError::OrderNotFound)
}
```

### Step 4: Verify Compliance
Before committing, verify that the implementation follows:
- Code organization guidelines from `@RULES.md ##Code Organization`
- Error handling principles from `@RULES.md ##Error Handling Principles`
- Testing requirements from `@RULES.md ##Testing`
- Security practices from `@RULES.md ##Security Practices`

### Step 5: Document References
In your commit message:
```
feat(orderbook): implement order ID lookup using HashMap

- Add get_order_by_id method to OrderBook
- This implements the Priority 2 feature from DEX-OS-V1.csv:
  "Core Trading,Orderbook,Orderbook,Hash Map,Order ID Lookup,Medium"
- Include comprehensive unit tests for success and error cases
- Follow guidelines from @RULES.md ##Coding Standards and ##Error Handling

References:
- @RULES.md ##Code Organization
- @RULES.md ##Coding Standards
- @RULES.md ##Error Handling
- @RULES.md ##Testing
```

## Best Practices Summary

1. **Always reference the CSV**: Include the exact CSV entry in your implementation
2. **Follow priority sequence**: Don't implement Priority 2 features before Priority 1 is complete
3. **Use specified DSAs**: Implement exactly the data structures and algorithms specified
4. **Apply all relevant rules**: Reference appropriate sections from RULES.md
5. **Document thoroughly**: Include both code comments and commit message references
6. **Test comprehensively**: Cover both success and failure cases
7. **Verify compliance**: Ensure your implementation meets all RULES.md requirements

By following this process, you'll ensure that all features in DEX-OS are implemented consistently and according to the project's architectural decisions.