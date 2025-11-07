# How to Interact with RULES.md

This document provides a comprehensive guide on how to effectively use and interact with the RULES.md file in the DEX-OS project.

## Understanding RULES.md Structure

The RULES.md file is organized into logical sections that correspond to different aspects of software development:

1. **Code Organization** - Project structure and module organization
2. **Coding Standards** - Rust conventions, error handling, documentation
3. **Testing** - Unit, integration, and performance testing guidelines
4. **Database Guidelines** - Schema design and query optimization
5. **API Design** - RESTful principles and security considerations
6. **WebAssembly Guidelines** - WASM interface design and performance
7. **Git Workflow** - Commit messages and branching strategy
8. **Performance Considerations** - Memory management and concurrency
9. **Security Practices** - Input validation, data protection, authentication
10. **OWASP and LLM-OWASP Security Practices** - Web and AI security compliance
11. **Monitoring and Observability** - Logging and metrics
12. **Deployment** - Configuration and containerization
13. **Dependencies** - Version management and selection criteria
14. **Web3 Security, Testing, and Protection** - Blockchain-specific guidelines
15. **Feature Implementation Priority** - Priority-based development approach

## Methods to Reference RULES.md

### 1. Section-Based Referencing
Reference specific sections when working on related tasks:
- `@RULES.md ##Coding Standards` - For code implementation tasks
- `@RULES.md ##Security Practices` - For security-related work
- `@RULES.md ##Testing` - For testing requirements
- `@RULES.md ##API Design` - For API development

### 2. Rule Application Process
When working on any task, follow this process:

1. **Identify Task Type**
   - What kind of functionality are you implementing?
   - Which components does it affect?

2. **Find Relevant Sections**
   - Scan RULES.md for sections related to your task
   - Look for specific guidelines that apply

3. **Apply Guidelines**
   - Follow the coding standards
   - Implement required security measures
   - Add appropriate tests
   - Document your code

4. **Verify Compliance**
   - Check that your implementation follows the rules
   - Ensure all relevant guidelines are applied

### 3. Priority-Based Development
The RULES.md includes a "Feature Implementation Priority" section that references DEX-OS-V1.csv:

1. Check the priority level of your feature in DEX-OS-V1.csv
2. Follow the development sequence guidelines
3. Reference the algorithm selection guidance
4. Ensure higher priority features are complete before lower priority ones

## Practical Examples

### Example 1: Implementing a New Orderbook Feature
```
References:
- @RULES.md ##Code Organization
- @RULES.md ##Coding Standards
- @RULES.md ##Testing
- @RULES.md ##Feature Implementation Priority
```

### Example 2: Adding API Security
```
References:
- @RULES.md ##API Design
- @RULES.md ##Security Practices
- @RULES.md ##OWASP and LLM-OWASP Security Practices
```

### Example 3: Database Schema Changes
```
References:
- @RULES.md ##Database Guidelines
- @RULES.md ##Security Practices
```

## Best Practices for RULES.md Interaction

### 1. Always Reference Applicable Rules
When implementing any feature, explicitly reference the relevant RULES.md sections in:
- Code comments
- Commit messages
- Pull request descriptions

### 2. Keep RULES.md Updated
The RULES.md file is a living document:
- Add new rules when implementing significant features
- Maintain consistent formatting
- Place rules in appropriate sections
- Submit changes via pull request

### 3. Use RULES.md for Code Reviews
During code reviews, verify that:
- Applicable rules from RULES.md have been followed
- New code complies with established standards
- Security and testing requirements are met

## Quick Reference Table

| Task Type | RULES.md Sections to Check |
|-----------|---------------------------|
| New feature implementation | Feature Implementation Priority, Code Organization, Coding Standards |
| Security enhancement | Security Practices, OWASP/LLM-OWASP, Web3 Security |
| Performance optimization | Performance Considerations, Database Guidelines |
| API development | API Design, WebAssembly Guidelines |
| Database work | Database Guidelines, Security |
| Testing | Testing section throughout |
| Documentation | Documentation section under Coding Standards |
| Web3/Blockchain | Web3 Security, Testing, Protection |

## Contributing to RULES.md

When adding new rules to RULES.md:

1. **Maintain Consistent Formatting**
   - Use the same bullet-point structure
   - Keep language clear and actionable
   - Place rules in appropriate existing sections

2. **Ensure Specificity**
   - Make each rule actionable
   - Provide context when needed
   - Avoid vague or ambiguous language

3. **Follow Contribution Guidelines**
   - Submit changes via pull request
   - Get review from team members
   - Update related documentation if needed

This approach ensures that RULES.md remains a useful, up-to-date reference for all development activities in the DEX-OS project.