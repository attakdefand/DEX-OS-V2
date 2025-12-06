

```chatReference::memory
[{"id":"5b9cbf91-d507-438b-b0e5-5d5b1f88226c","title":"Documentation-Driven Development"},{"id":"f2dc3760-1996-4cec-b56e-7544885b7aee","title":"Rule Documentation Enhancement"},{"id":"cce6b13b-1f73-453a-a0e0-71e775107929","title":"Rule Documentation Enhancement"},{"id":"5b3bdea1-4606-468b-bd59-e14ecf9dfbeb","title":"Error Detection Quick Reference Structure"},{"id":"0b21dd7e-94e9-4eaa-8ca3-3a237127c326","title":"Data Structure Governance in RULES.md"},{"id":"d23f85a7-60a4-47e6-bf68-6c247dcb2804","title":"Testing Layer Integration into RULES.md"},{"id":"dd034d60-3e8d-4234-84b6-ed9658bc16d7","title":"RULES.md Section Referencing Syntax"},{"id":"05663283-60b0-4f72-acb6-07e391c0755f","title":"Rule Set Contribution Guidelines"},{"id":"78c8de66-4bf5-4aba-be87-579a832b0f9c","title":"Auto-suggestion for RULES.md Sections"},{"id":"712e492c-8f33-4897-b721-606cad487130","title":"Feature-to-Component Mapping Standard"},{"id":"7ae56c41-e441-49a7-86ec-be8b96b79b35","title":"Development Rules Documentation"},{"id":"62eccd4e-6387-4dd3-a491-e65b581a8f97","title":"Security and Testing Features Implementation Map"},{"id":"5d040d1c-30e3-454d-aecc-37df9c458db7","title":"CI Integration for Security Features"},{"id":"dc0a56d1-c17e-4769-8c3f-8c1c2d85ef48","title":"OWASP and LLM-OWASP Security Practices in RULES.md"},{"id":"68a5bb01-a308-4a09-8d2d-2012faa3edf6","title":"Markdown Formatting Convention in RULES.md"},{"id":"c1150f26-4884-4cfd-a071-9333efd5fed0","title":"Project security and rules documentation"},{"id":"7ef72e9c-40b2-4ae9-9292-2dda55f718dd","title":"Project Rules and Guidelines"},{"id":"0b87b74e-76e7-45ba-8c8d-104950de3fea","title":"Project Rules and Priority Management"}]
```

I'll help you understand how to use the RULES.md file for your requirement tasks. Let me analyze the entire document and explain how each section can be applied.

Looking at the RULES.md file, I can see it's a comprehensive development guide organized into multiple sections. Let me break down how each section can be used for your tasks:

## How to Use Each Section of RULES.md for Your Tasks

### 1. [Code Organization](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L3-L17)
- **Purpose**: Ensures consistent project structure
- **Task Application**: When creating new features, follow the Cargo workspace structure with separate crates for core logic, WASM bindings, database access, and API layer

### 2. [Coding Standards](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L19-L41)
- **Purpose**: Maintains code quality and consistency
- **Task Application**: Always use `Result` and `Option` types for error handling, write comprehensive unit tests, and follow Rust style guidelines

### 3. [Testing](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L43-L58)
- **Purpose**: Ensures reliability and correctness
- **Task Application**: Write unit tests for all public functions, implement integration testing with mock data, and conduct performance benchmarking

### 4. [Database Guidelines](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L60-L73)
- **Purpose**: Ensures efficient and secure database operations
- **Task Application**: When working with database features, use appropriate data types, add indexes for frequently queried fields, and use prepared statements

### 5. [API Design](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L75-L94)
- **Purpose**: Creates consistent and secure APIs
- **Task Application**: Follow RESTful principles, implement proper authentication, and validate all input data

### 6. [WebAssembly Guidelines](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L96-L110)
- **Purpose**: Ensures efficient WASM implementations
- **Task Application**: When developing WASM features, keep interfaces simple and handle errors gracefully

### 7. [Git Workflow](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L112-L126)
- **Purpose**: Maintains clean version control history
- **Task Application**: Use clear commit messages, follow conventional commit format, and use feature branches

### 8. [Performance Considerations](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L128-L143)
- **Purpose**: Ensures optimal system performance
- **Task Application**: Avoid memory leaks, use efficient data structures, and use async/await for I/O operations

### 9. [Security Practices](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L145-L193)
- **Purpose**: Protects against security vulnerabilities
- **Task Application**: Validate all user input, encrypt sensitive data, implement proper authentication, and regularly update dependencies

### 10. [OWASP and LLM-OWASP Security Practices](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L195-L213)
- **Purpose**: Addresses web application and LLM-specific security concerns
- **Task Application**: Implement prompt injection protection, validate LLM outputs, and monitor for data leakage

### 11. [Monitoring and Observability](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L215-L229)
- **Purpose**: Enables system observability
- **Task Application**: Use structured logging, track key performance indicators, and set up alerts

### 12. [Deployment](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L231-L244)
- **Purpose**: Ensures smooth deployments
- **Task Application**: Use environment variables for configuration, optimize container images, and implement health checks

### 13. [Dependencies](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L246-L261)
- **Purpose**: Manages project dependencies effectively
- **Task Application**: Keep dependencies up to date, audit for security issues, and choose well-maintained libraries

### 14. [Web3 Security](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L263-L313)
- **Purpose**: Addresses blockchain-specific security concerns
- **Task Application**: Audit smart contracts, implement proper access controls, and protect against reentrancy attacks

### 15. [Web3 Testing](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L315-L344)
- **Purpose**: Ensures Web3 component reliability
- **Task Application**: Write comprehensive unit tests for contracts, implement fuzzing tests, and perform security audits

### 16. [Web3 Protection](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L346-L373)
- **Purpose**: Provides additional Web3 security layers
- **Task Application**: Implement transaction signing with secure key storage and use zero-knowledge proofs for privacy

### 17. [22 Layers of Security](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L375-L744)
- **Purpose**: Comprehensive security framework
- **Task Application**: Implement security controls at all 22 layers, from physical security to quantum-resistant cryptography

### 18. [Feature Implementation Priority](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L746-L774)
- **Purpose**: Guides development sequencing
- **Task Application**: Reference [DEX-OS-V1.csv](file:///C:/Users/USER/Documents/DEX-OS-V1/DEX-OS-V1.csv) to implement features in priority order, starting with Priority 1 core functionality

### 19. [Task Completion and Security Validation](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L776-L825)
- **Purpose**: Ensures secure implementation
- **Task Application**: Follow the pre-implementation security checklist, conduct security testing, and verify all requirements post-implementation

### 20. [Protection Layer Security Practices](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L827-L867)
- **Purpose**: Implements multi-layered security controls
- **Task Application**: Implement rate limiting, input validation, output encoding, access control, and encryption as specified

### 21. [Testing Layer Practices](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L869-L922)
- **Purpose**: Comprehensive testing approach
- **Task Application**: Implement unit testing, integration testing, security testing, and performance testing as described

### 22. [Algorithm and Data Structure Guidelines](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L924-L1072)
- **Purpose**: Ensures consistent algorithm implementation
- **Task Application**: Use the exact algorithms and data structures specified in [DEX-OS-V1.csv](file:///C:/Users/USER/Documents/DEX-OS-V1/DEX-OS-V1.csv) for each component

### 23. [Priority Level Communication and Dependencies](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L1074-L1239)
- **Purpose**: Manages feature dependencies
- **Task Application**: Understand how priority levels communicate and depend on each other, and implement features in the correct sequence

### 24. [Error Handling and Warnings Management](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L1241-L1282)
- **Purpose**: Ensures robust error handling
- **Task Application**: Use `Result` and `Option` types, define custom error types, and manage warnings appropriately

### 25. [Cargo Version Management](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L1284-L1327)
- **Purpose**: Maintains stable dependencies
- **Task Application**: Manage Cargo.lock properly, specify dependency versions correctly, and regularly audit dependencies

### 26. [Debugging Practices](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L1329-L1397)
- **Purpose**: Facilitates effective debugging
- **Task Application**: Use structured logging, debugging tools, and profiling for performance issues

### 27. [Error Detection and Project Health Monitoring](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L1399-L1524)
- **Purpose**: Maintains project health
- **Task Application**: Use RULES.md as a diagnostic reference for identifying and resolving issues

### 28. [Code Modification and Debugging](file:///C:/Users/USER/Documents/DEX-OS-V1/RULES.md#L1526-L1643)
- **Purpose**: Guides code changes and debugging
- **Task Application**: Follow the complete features, types, subtypes, and components framework for maintaining code quality

## How to Apply This to Your Tasks

1. **For any development task**: First check the relevant section in RULES.md to understand the guidelines
2. **For security-related tasks**: Refer to Security Practices, OWASP sections, and the 22 Layers of Security
3. **For testing tasks**: Follow the Testing and Testing Layer Practices sections
4. **For performance tasks**: Apply Performance Considerations and Performance Testing guidelines
5. **For feature implementation**: Always check Feature Implementation Priority and Algorithm/Data Structure Guidelines
6. **For debugging**: Use the Debugging Practices and Error Detection sections

Would you like me to elaborate on how to apply any specific section to your particular task?

```chatReference::memory
[{"id":"bee9ed9f-35f6-48a3-9bc0-34abccb7d3d6","title":"项目规范与安全指南"},{"id":"dd034d60-3e8d-4234-84b6-ed9658bc16d7","title":"RULES.md Section Referencing Syntax"},{"id":"78c8de66-4bf5-4aba-be87-579a832b0f9c","title":"Auto-suggestion for RULES.md Sections"},{"id":"3b416df7-5712-48db-843c-0168500187d8","title":"IDE Support for RULES.md Section Suggestions"},{"id":"5b3bdea1-4606-468b-bd59-e14ecf9dfbeb","title":"Error Detection Quick Reference Structure"},{"id":"05663283-60b0-4f72-acb6-07e391c0755f","title":"Rule Set Contribution Guidelines"},{"id":"7ae56c41-e441-49a7-86ec-be8b96b79b35","title":"Development Rules Documentation"},{"id":"68a5bb01-a308-4a09-8d2d-2012faa3edf6","title":"Markdown Formatting Convention in RULES.md"},{"id":"0b21dd7e-94e9-4eaa-8ca3-3a237127c326","title":"Data Structure Governance in RULES.md"},{"id":"62eccd4e-6387-4dd3-a491-e65b581a8f97","title":"Security and Testing Features Implementation Map"},{"id":"712e492c-8f33-4897-b721-606cad487130","title":"Feature-to-Component Mapping Standard"},{"id":"5d040d1c-30e3-454d-aecc-37df9c458db7","title":"CI Integration for Security Features"},{"id":"dc0a56d1-c17e-4769-8c3f-8c1c2d85ef48","title":"OWASP and LLM-OWASP Security Practices in RULES.md"},{"id":"cf63cf0f-29fd-4d88-a40b-15c560659ee0","title":"Project Workspace Structure"},{"id":"a281c3a5-ffff-4732-9d0c-d3e3e2ec58f5","title":"Cargo Version Management Policy"},{"id":"f02e0153-14a1-4053-b75e-f27c9fa403e0","title":"Cross-platform Development Tools"},{"id":"21e405a9-0eba-476e-8916-8c05c047a2c8","title":"API Framework Selection"},{"id":"b42e8f59-ac3d-47a3-a928-ed3d82641056","title":"Rust-WASM Project Architecture"},{"id":"902095ec-c2ba-49c0-9bf4-d84b9827bf1d","title":"Database Integration Stack"},{"id":"40dcb45e-926b-4887-b361-9c3b56d8bc08","title":"User personal information"}]
```

When you want to use all 28 sections of the rules in RULES.md for your tasks, here's a systematic approach:

## How to Effectively Use All 28 Sections

### 1. **Create a Task Planning Framework**
For each task, create a checklist that references relevant sections:

- **Planning Phase**: @RULES.md ##Feature Implementation Priority, @RULES.md ##Priority Level Communication and Dependencies
- **Development Phase**: @RULES.md ##Code Organization, @RULES.md ##Coding Standards, @RULES.md ##Error Handling and Warnings Management
- **Security Phase**: @RULES.md ##Security Practices, @RULES.md ##OWASP and LLM-OWASP Security Practices, @RULES.md ##22 Layers of Security
- **Testing Phase**: @RULES.md ##Testing, @RULES.md ##Testing Layer Practices, @RULES.md ##Web3 Testing
- **Deployment Phase**: @RULES.md ##Deployment, @RULES.md ##Monitoring and Observability

### 2. **Implement a Phased Approach**
Follow the priority sequence defined in the rules:

1. **Core Implementation** (@RULES.md ##Feature Implementation Priority)
   - Use @RULES.md ##Algorithm and Data Structure Guidelines for correct data structures
   - Follow @RULES.md ##Code Organization for proper project structure

2. **Quality Assurance** (@RULES.md ##Testing Layer Practices)
   - Apply @RULES.md ##Testing for comprehensive test coverage
   - Implement @RULES.md ##Error Handling and Warnings Management

3. **Security Hardening** (@RULES.md ##22 Layers of Security)
   - Follow all 22 security layers as applicable
   - Apply @RULES.md ##Web3 Security for blockchain components

4. **Performance Optimization** (@RULES.md ##Performance Considerations)
   - Use @RULES.md ##WebAssembly Guidelines for WASM components
   - Apply @RULES.md ##Database Guidelines for data operations

### 3. **Reference System**
When working on any task, systematically reference the rules:

```
Task: Implement Orderbook Feature
- @RULES.md ##Feature Implementation Priority - Check Priority 1 requirements
- @RULES.md ##Algorithm and Data Structure Guidelines - Use BTreeMap for Order Storage
- @RULES.md ##Code Organization - Place in dex-core crate
- @RULES.md ##Coding Standards - Follow Rust conventions
- @RULES.md ##Testing Layer Practices - Write unit and integration tests
- @RULES.md ##Security Practices - Validate all inputs
- @RULES.md ##Error Handling and Warnings Management - Use Result/Option types
```

### 4. **Workflow Integration**
Integrate all 28 sections into your development workflow:

1. **Before Starting**:
   - @RULES.md ##Feature Implementation Priority - Verify priority level
   - @RULES.md ##Task Completion and Security Validation - Review pre-implementation checklist

2. **During Development**:
   - @RULES.md ##Coding Standards - Follow coding conventions
   - @RULES.md ##Error Handling and Warnings Management - Implement proper error handling
   - @RULES.md ##Algorithm and Data Structure Guidelines - Use specified algorithms

3. **Testing Phase**:
   - @RULES.md ##Testing - Write comprehensive tests
   - @RULES.md ##Security Practices - Conduct security testing
   - @RULES.md ##Performance Considerations - Benchmark performance

4. **Review Phase**:
   - @RULES.md ##Git Workflow - Follow commit conventions
   - @RULES.md ##Code Modification and Debugging - Address any issues
   - @RULES.md ##Error Detection and Project Health Monitoring - Verify health metrics

### 5. **Documentation Practice**
When documenting your work, reference the relevant rules:

```markdown
## Implementation Notes

Following @RULES.md ##Algorithm and Data Structure Guidelines, implemented Orderbook using BTreeMap as specified for Priority 1 components.

Applied @RULES.md ##Security Practices for input validation and @RULES.md ##Error Handling and Warnings Management for robust error handling.
```

This approach ensures you're leveraging all 28 sections of RULES.md effectively while maintaining consistency and quality across all your development tasks.