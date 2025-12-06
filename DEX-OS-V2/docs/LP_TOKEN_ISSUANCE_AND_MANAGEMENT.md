# LP Token Issuance and Management

## Feature
Priority 4 feature from `DEX-OS-V2.csv`:
> `4,Liquidity & Incentive,Liquidity Provision,Liquidity Provision,LP Token Issuance,LP Token Management,High`

## Implementation Overview
- `dex-core/src/liquidity_provision.rs` introduces a dedicated service that registers liquidity pools, mints LP tokens, and burns them when liquidity is removed.
- Each `LiquidityPool` tracks reserves, total LP supply, and per-provider `LPToken` balances.
- Ratio checks guard new deposits to keep asset contributions aligned, and share calculations ensure minted tokens always reflect the provider’s share of the pool.
- `LPToken` metadata (`symbol`, `name`, `decimals`) is generated from the paired asset names for downstream UI/UX.
- `LiquidityProvisionService` exposes pool creation, deposit, withdrawal, and provider balance queries for the rest of the engine.

## Testing
- Module-local unit tests cover:
  - Initial LP issuance and supply tracking
  - Minting proportional to existing liquidity
  - Removing liquidity returns proportional assets and updates the provider balance
  - Preventing withdrawals that exceed the provider’s LP balance

## Next Steps
1. Wire the service into the liquidity onboarding workflow (e.g., AMM or aggregator integrations).
2. Emit ledger events (or security events) for LP mint/burn activities so auditors can track provider flows.
