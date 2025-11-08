# New Features Testing Plan

This document outlines the testing plan for the newly implemented features in accordance with the testing guidelines in RULES.md.

## Implemented Features

### 1. Health Factor Calculation for Liquidation Prevention (Lending Module)
- **Module**: `dex-core/src/lending.rs`
- **Feature Reference**: "Core Trading,Lending,Lending,Health Factor Calculation,Liquidation Prevention,High"

### 2. 1,000,000 Shards for Sharding (Quantum Consensus Module)
- **Module**: `dex-core/src/quantum_consensus.rs`
- **Feature Reference**: "Core Components,Quantum Consensus (QBFT),Consensus,1,000,000 Shards,Sharding,High"

### 3. Global Finality for Finality (Quantum Consensus Module)
- **Module**: `dex-core/src/quantum_consensus.rs`
- **Feature Reference**: "Core Components,Quantum Consensus (QBFT),Consensus,Global Finality,Finality,High"

## Unit Testing

### Health Factor Calculation Tests

1. **Basic Health Factor Calculation**
   - Test with valid inputs and expected outputs
   - Verify calculation formula: (Collateral Value * Liquidation Threshold) / Loan Value
   - Test with different asset types and prices

2. **Edge Case Tests**
   - Test with zero loan value (should return infinity)
   - Test with negative prices (should return error)
   - Test with invalid liquidation thresholds (should return error)
   - Test with non-existent loans (should return error)

3. **Health Factor Update Tests**
   - Test updating health factor for existing loans
   - Test updating health factor for non-existent loans (should return error)
   - Test updating health factor for inactive loans (should return error)

4. **Liquidation Check Tests**
   - Test should_liquidate function with healthy loans (should return false)
   - Test should_liquidate function with undercollateralized loans (should return true)
   - Test with different minimum health factor thresholds

5. **Undercollateralized Loan Detection Tests**
   - Test with mix of healthy and undercollateralized loans
   - Test with empty loan portfolio
   - Test with all healthy loans
   - Test with all undercollateralized loans

### Sharding Tests

1. **Shard Initialization Tests**
   - Test initializing with valid number of shards (1-1,000,000)
   - Test initializing with zero shards (should return error)
   - Test initializing with too many shards (>1,000,000) (should return error)
   - Test initializing with no validators (should return error)

2. **Shard Management Tests**
   - Test getting existing shards
   - Test getting non-existent shards (should return None)
   - Test getting all shards
   - Test validator distribution across shards

3. **Shard Block Processing Tests**
   - Test adding blocks to existing shards
   - Test adding blocks to non-existent shards (should return error)
   - Test processing blocks with sharding support
   - Test processing blocks for non-existent shards (should return error)

### Global Finality Tests

1. **Finality Tracker Tests**
   - Test initial state (should be zero)
   - Test updating shard finalities
   - Test global finality calculation (should be minimum of all shard finalities)
   - Test updating individual shard finalities

2. **Finality Query Tests**
   - Test getting global finalized height
   - Test getting shard finalized heights
   - Test getting finalized height for non-existent shards (should return None)

3. **Integration Tests**
   - Test updating shard finalities and verifying global finality updates
   - Test processing blocks and updating finalities
   - Test consistency between shard and global finality tracking

## Integration Testing

### Module Integration Tests

1. **Lending Module Integration**
   - Test health factor calculation integration with loan accounting
   - Test liquidation prevention workflow
   - Test integration with existing loan management functions

2. **Quantum Consensus Integration**
   - Test sharding integration with consensus engine
   - Test finality tracking integration with shard processing
   - Test block processing with sharding support

3. **Cross-Module Integration**
   - Test that lending and consensus modules don't interfere with each other
   - Test that new features work with existing functionality

## Performance Testing

### Stress Tests

1. **Health Factor Calculation Performance**
   - Calculate health factors for large numbers of loans
   - Measure calculation time
   - Verify accuracy under stress

2. **Sharding Performance**
   - Initialize large numbers of shards
   - Process blocks across many shards simultaneously
   - Measure performance impact
   - Verify system stability under load

3. **Finality Tracking Performance**
   - Update finalities for large numbers of shards
   - Measure global finality calculation time
   - Verify consistency under stress

## Security Testing

### Input Validation Tests

1. **Invalid Input Tests**
   - Test with negative prices
   - Test with zero or negative loan amounts
   - Test with invalid liquidation thresholds
   - Test with excessively large numbers

2. **Boundary Condition Tests**
   - Test with minimum and maximum valid values
   - Test at shard count boundaries (0, 1, 1,000,000)
   - Test at finality height boundaries

### Error Handling Tests

1. **Error Propagation Tests**
   - Verify that errors are properly propagated
   - Test error handling in all functions
   - Verify that error messages are descriptive

2. **Recovery Tests**
   - Test system recovery from errors
   - Verify that system state remains consistent after errors
   - Test retry mechanisms

## Property-Based Testing

### Mathematical Properties

1. **Health Factor Properties**
   - Verify that health factors are always positive (except for infinity case)
   - Verify that undercollateralized loans have health factors below threshold
   - Verify that healthy loans have health factors above threshold

2. **Sharding Invariants**
   - Verify that shard IDs are unique
   - Verify that validator distribution is consistent
   - Verify that block counts are accurate

3. **Finality Invariants**
   - Verify that global finality is always ≤ individual shard finalities
   - Verify that finality values are non-decreasing
   - Verify that finality tracking is consistent

## Test Data

### Sample Test Data

1. **Loan Test Data**
   - Healthy loan: $1000 USDC loan, 0.5 ETH collateral ($2000/ETH), 80% threshold
   - Undercollateralized loan: $1000 USDC loan, 0.05 ETH collateral ($2000/ETH), 80% threshold

2. **Shard Test Data**
   - 5 shards with 3 validators each
   - 100 shards with varying validator counts
   - Maximum shard count (1,000,000) with minimal validators

3. **Finality Test Data**
   - Shards with finality heights: 100, 90, 95 (global should be 90)
   - Shards with equal finality heights
   - Empty finality tracker

## Test Environment

### Development Environment
- Rust 1.70+
- Cargo test framework
- Standard development tools

### Continuous Integration
- Automated testing on all commits
- Performance regression testing
- Security scanning

## Test Coverage Goals

### Code Coverage
- 100% function coverage
- 95% line coverage
- 90% branch coverage

### Feature Coverage
- All health factor calculation features
- All sharding features
- All finality tracking features
- All error conditions

## Test Execution

### Manual Testing
- Execute all unit tests
- Verify test output
- Check for any test failures

### Automated Testing
- Integrate with CI/CD pipeline
- Run tests on all pull requests
- Generate test reports

## Test Reporting

### Test Results
- Pass/fail status for all tests
- Performance metrics
- Code coverage reports
- Error summaries

### Test Documentation
- Test case descriptions
- Test execution procedures
- Test result interpretations
- Defect tracking

## Conclusion

This testing plan ensures comprehensive coverage of the new features, following the testing guidelines in RULES.md. The plan includes unit tests, integration tests, performance tests, security tests, and property-based tests to verify the correctness, security, and performance of the new implementations.