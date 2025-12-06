 1. Replace the demo hook with real order/AMM flows and API integration so components       
     reflect live data.                                                                      
  2. Add state management/tests (React Testing Library or Vitest) and linting (eslint,       
     prettier) to enforce consistency.                                                       
  3. Layer in wallet connectivity, form validation, and responsive tweaks for production UX.

  1. Establish a component library (Storybook or Ladle) to develop cards, tables, forms,     
     charts in isolation.                                                                    
  2. Draft a UI/UX spec covering navigation, responsive breakpoints, accessibility (WCAG AA),     and theme tokens.                                                                       
  3. Implement lint/test tooling on frontend (eslint, prettier, vitest) and wire into CI     
     alongside Rust checks.                                                                  
  4. Add security baseline: audit logging, error-handling policies, and automated cargo      
     audit/cargo deny runs before deployment. 