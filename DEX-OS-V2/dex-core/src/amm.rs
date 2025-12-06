//! Automated Market Maker implementation for the DEX-OS core engine

use crate::types::{Quantity, TokenId};
use std::collections::HashMap;

/// Errors that can occur when working with the AMM
#[derive(Debug, Clone, PartialEq)]
pub enum AMMError {
    InvalidToken,
    InsufficientLiquidity,
    PriceRangeNotFound,
}

impl std::fmt::Display for AMMError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AMMError::InvalidToken => write!(f, "Invalid token"),
            AMMError::InsufficientLiquidity => write!(f, "Insufficient liquidity"),
            AMMError::PriceRangeNotFound => write!(f, "Price range not found"),
        }
    }
}

impl std::error::Error for AMMError {}

/// Tick represents a price level in concentrated liquidity AMM
/// This implements the Priority 1 feature from DEX-OS-V1.csv:
/// "Core Trading,AMM,AMM,Concentrated Liquidity,Tick-based Positioning,High"
#[derive(Debug, Clone)]
pub struct Tick {
    /// Tick index (represents a specific price level)
    pub index: i32,
    /// Liquidity amount at this tick
    pub liquidity: Quantity,
    /// Net liquidity change at this tick
    pub liquidity_net: i64,
}

/// Constant product AMM implementation (x * y = k)
#[derive(Debug, Clone)]
pub struct ConstantProductAMM {
    /// Reserves of tokens in the pool
    pub reserves: HashMap<TokenId, Quantity>,
    /// Total supply of liquidity tokens
    pub total_supply: Quantity,
    /// Fee percentage (in basis points, so 30 = 0.3%)
    pub fee: u32,
    /// Ticks for concentrated liquidity positioning
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,AMM,AMM,Concentrated Liquidity,Tick-based Positioning,High"
    pub ticks: HashMap<i32, Tick>,
    /// Current tick index
    pub current_tick: i32,
    /// Square root of the current price
    pub sqrt_price: f64,
}

impl ConstantProductAMM {
    /// Create a new AMM pool
    pub fn new(fee: u32) -> Self {
        Self {
            reserves: HashMap::new(),
            total_supply: 0,
            fee,
            ticks: HashMap::new(),
            current_tick: 0,
            sqrt_price: 1.0,
        }
    }

    /// Add liquidity to the pool with concentrated liquidity positioning
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,AMM,AMM,Concentrated Liquidity,Tick-based Positioning,High"
    pub fn add_liquidity_concentrated(
        &mut self,
        token_a: TokenId,
        token_b: TokenId,
        amount_a: Quantity,
        amount_b: Quantity,
        tick_lower: i32,
        tick_upper: i32,
    ) -> Result<Quantity, AMMError> {
        // Initialize reserves if they don't exist
        self.reserves.entry(token_a.clone()).or_insert(0);
        self.reserves.entry(token_b.clone()).or_insert(0);

        // Create ticks if they don't exist
        self.ticks.entry(tick_lower).or_insert(Tick {
            index: tick_lower,
            liquidity: 0,
            liquidity_net: 0,
        });

        self.ticks.entry(tick_upper).or_insert(Tick {
            index: tick_upper,
            liquidity: 0,
            liquidity_net: 0,
        });

        // Update tick liquidity net values
        if let Some(lower_tick) = self.ticks.get_mut(&tick_lower) {
            lower_tick.liquidity_net += amount_a as i64;
        }

        if let Some(upper_tick) = self.ticks.get_mut(&tick_upper) {
            upper_tick.liquidity_net -= amount_a as i64;
        }

        // Add liquidity to the range
        for tick_index in tick_lower..tick_upper {
            self.ticks
                .entry(tick_index)
                .and_modify(|tick| {
                    tick.liquidity += amount_a;
                })
                .or_insert(Tick {
                    index: tick_index,
                    liquidity: amount_a,
                    liquidity_net: 0,
                });
        }

        // Update reserves
        *self.reserves.get_mut(&token_a).unwrap() += amount_a;
        *self.reserves.get_mut(&token_b).unwrap() += amount_b;

        // Calculate liquidity tokens based on contribution
        let liquidity_tokens = if self.total_supply == 0 {
            // First liquidity provider
            ((amount_a as f64 * amount_b as f64).sqrt() as Quantity).max(1)
        } else {
            // Subsequent liquidity providers
            let reserve_a = self.reserves[&token_a];
            let reserve_b = self.reserves[&token_b];
            let liquidity_a = (amount_a * self.total_supply) / reserve_a;
            let liquidity_b = (amount_b * self.total_supply) / reserve_b;
            liquidity_a.min(liquidity_b)
        };

        // Update total supply
        self.total_supply += liquidity_tokens;

        Ok(liquidity_tokens)
    }

    /// Remove liquidity from the pool with concentrated liquidity positioning
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,AMM,AMM,Concentrated Liquidity,Tick-based Positioning,High"
    pub fn remove_liquidity_concentrated(
        &mut self,
        token_a: TokenId,
        token_b: TokenId,
        liquidity_tokens: Quantity,
        tick_lower: i32,
        tick_upper: i32,
    ) -> Result<(Quantity, Quantity), AMMError> {
        if liquidity_tokens > self.total_supply {
            return Err(AMMError::InsufficientLiquidity);
        }

        let reserve_a = *self.reserves.get(&token_a).unwrap_or(&0);
        let reserve_b = *self.reserves.get(&token_b).unwrap_or(&0);

        let amount_a = (liquidity_tokens * reserve_a) / self.total_supply;
        let amount_b = (liquidity_tokens * reserve_b) / self.total_supply;

        // Update tick liquidity net values
        if let Some(lower_tick) = self.ticks.get_mut(&tick_lower) {
            lower_tick.liquidity_net -= amount_a as i64;
        }

        if let Some(upper_tick) = self.ticks.get_mut(&tick_upper) {
            upper_tick.liquidity_net += amount_a as i64;
        }

        // Remove liquidity from the range
        for tick_index in tick_lower..tick_upper {
            if let Some(tick) = self.ticks.get_mut(&tick_index) {
                tick.liquidity = tick.liquidity.saturating_sub(amount_a);
            }
        }

        // Update reserves
        *self.reserves.get_mut(&token_a).unwrap() -= amount_a;
        *self.reserves.get_mut(&token_b).unwrap() -= amount_b;

        // Update total supply
        self.total_supply -= liquidity_tokens;

        Ok((amount_a, amount_b))
    }

    /// Get liquidity at a specific tick
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,AMM,AMM,Concentrated Liquidity,Tick-based Positioning,High"
    pub fn get_liquidity_at_tick(&self, tick_index: i32) -> Quantity {
        self.ticks
            .get(&tick_index)
            .map(|tick| tick.liquidity)
            .unwrap_or(0)
    }

    /// Get all ticks with liquidity
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Trading,AMM,AMM,Concentrated Liquidity,Tick-based Positioning,High"
    pub fn get_active_ticks(&self) -> Vec<&Tick> {
        self.ticks
            .values()
            .filter(|tick| tick.liquidity > 0)
            .collect()
    }

    /// Add liquidity to the pool
    pub fn add_liquidity(
        &mut self,
        token_a: TokenId,
        amount_a: Quantity,
        token_b: TokenId,
        amount_b: Quantity,
    ) -> Result<Quantity, AMMError> {
        // Initialize reserves if they don't exist
        self.reserves.entry(token_a.clone()).or_insert(0);
        self.reserves.entry(token_b.clone()).or_insert(0);

        let reserve_a = self.reserves[&token_a];
        let reserve_b = self.reserves[&token_b];

        let liquidity_tokens = if self.total_supply == 0 {
            // First liquidity provider
            // Calculate initial liquidity tokens as geometric mean
            ((amount_a as f64 * amount_b as f64).sqrt() as Quantity).max(1)
        } else {
            // Subsequent liquidity providers
            // Calculate liquidity tokens based on proportional contribution
            let liquidity_a = (amount_a * self.total_supply) / reserve_a;
            let liquidity_b = (amount_b * self.total_supply) / reserve_b;
            liquidity_a.min(liquidity_b)
        };

        // Update reserves
        *self.reserves.get_mut(&token_a).unwrap() += amount_a;
        *self.reserves.get_mut(&token_b).unwrap() += amount_b;

        // Update total supply
        self.total_supply += liquidity_tokens;

        Ok(liquidity_tokens)
    }

    /// Remove liquidity from the pool
    pub fn remove_liquidity(
        &mut self,
        token_a: TokenId,
        token_b: TokenId,
        liquidity_tokens: Quantity,
    ) -> Result<(Quantity, Quantity), AMMError> {
        if liquidity_tokens > self.total_supply {
            return Err(AMMError::InsufficientLiquidity);
        }

        let reserve_a = *self.reserves.get(&token_a).unwrap_or(&0);
        let reserve_b = *self.reserves.get(&token_b).unwrap_or(&0);

        let amount_a = (liquidity_tokens * reserve_a) / self.total_supply;
        let amount_b = (liquidity_tokens * reserve_b) / self.total_supply;

        // Update reserves
        *self.reserves.get_mut(&token_a).unwrap() -= amount_a;
        *self.reserves.get_mut(&token_b).unwrap() -= amount_b;

        // Update total supply
        self.total_supply -= liquidity_tokens;

        Ok((amount_a, amount_b))
    }

    /// Swap tokens in the pool
    pub fn swap(
        &mut self,
        from_token: TokenId,
        to_token: TokenId,
        amount_in: Quantity,
    ) -> Result<Quantity, AMMError> {
        let reserve_in = *self
            .reserves
            .get(&from_token)
            .ok_or(AMMError::InvalidToken)?;
        let reserve_out = *self.reserves.get(&to_token).ok_or(AMMError::InvalidToken)?;

        if reserve_in == 0 || reserve_out == 0 {
            return Err(AMMError::InsufficientLiquidity);
        }

        // Calculate amount out with fee
        let amount_in_with_fee = amount_in * (10000 - self.fee) as u64;
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = (reserve_in * 10000) + amount_in_with_fee;
        let amount_out = numerator / denominator;

        if amount_out >= reserve_out {
            return Err(AMMError::InsufficientLiquidity);
        }

        // Update reserves
        *self.reserves.get_mut(&from_token).unwrap() += amount_in;
        *self.reserves.get_mut(&to_token).unwrap() -= amount_out;

        Ok(amount_out)
    }

    /// Get the price of one token in terms of another
    pub fn get_price(&self, from_token: &TokenId, to_token: &TokenId) -> Result<f64, AMMError> {
        let reserve_in = *self
            .reserves
            .get(from_token)
            .ok_or(AMMError::InvalidToken)?;
        let reserve_out = *self.reserves.get(to_token).ok_or(AMMError::InvalidToken)?;

        if reserve_in == 0 {
            return Err(AMMError::InsufficientLiquidity);
        }

        Ok(reserve_out as f64 / reserve_in as f64)
    }

    /// Find the optimal price within a given range using binary search
    /// This implements the Priority 2 feature from DEX-OS-V1.csv:
    /// "Core Trading,AMM,AMM,Binary Search,Price Range Checks,Medium"
    ///
    /// Uses binary search to find a price within the specified range that meets
    /// the desired criteria for trading.
    ///
    /// # Arguments
    /// * `from_token` - The token being sold
    /// * `to_token` - The token being bought
    /// * `min_price` - Minimum acceptable price (in to_token/from_token)
    /// * `max_price` - Maximum acceptable price (in to_token/from_token)
    /// * `tolerance` - Tolerance for price matching (as a ratio)
    ///
    /// # Returns
    /// * `Ok(f64)` - The optimal price within the range
    /// * `Err(AMMError)` - If no suitable price is found or on error
    pub fn find_price_in_range(
        &self,
        from_token: &TokenId,
        to_token: &TokenId,
        min_price: f64,
        max_price: f64,
        tolerance: f64,
    ) -> Result<f64, AMMError> {
        // Validate inputs
        if min_price <= 0.0 || max_price <= 0.0 || tolerance <= 0.0 {
            return Err(AMMError::PriceRangeNotFound);
        }

        if min_price > max_price {
            return Err(AMMError::PriceRangeNotFound);
        }

        let reserve_in = *self
            .reserves
            .get(from_token)
            .ok_or(AMMError::InvalidToken)?;
        let reserve_out = *self.reserves.get(to_token).ok_or(AMMError::InvalidToken)?;

        if reserve_in == 0 || reserve_out == 0 {
            return Err(AMMError::InsufficientLiquidity);
        }

        // Get current price
        let current_price = reserve_out as f64 / reserve_in as f64;

        // Check if current price is within range
        if current_price >= min_price && current_price <= max_price {
            return Ok(current_price);
        }

        // If current price is outside the range, use binary search to find
        // a trade amount that would result in a price within range
        let target_price = (min_price + max_price) / 2.0;

        // Binary search for the optimal trade amount
        let result = self.binary_search_for_price(
            reserve_in,
            reserve_out,
            target_price,
            tolerance,
            0.0,
            (reserve_in as f64).min(1_000_000.0), // Upper bound for search
        )?;

        Ok(result)
    }

    /// Helper function that performs binary search to find a trade amount
    /// that results in a desired price
    fn binary_search_for_price(
        &self,
        reserve_in: Quantity,
        reserve_out: Quantity,
        target_price: f64,
        tolerance: f64,
        low: f64,
        high: f64,
    ) -> Result<f64, AMMError> {
        const MAX_ITERATIONS: u32 = 100;
        let mut iterations = 0;
        let mut low = low;
        let mut high = high;

        while iterations < MAX_ITERATIONS && (high - low) > 1e-10 {
            let mid = (low + high) / 2.0;

            // Calculate resulting price after trading 'mid' amount of input token
            // For constant product formula: x * y = k
            // After trade: (x + mid) * (y - out) = k
            // So: out = y - k / (x + mid) = y - (x * y) / (x + mid)
            let k = (reserve_in as f64) * (reserve_out as f64);
            let new_reserve_in = (reserve_in as f64) + mid;

            if new_reserve_in <= 0.0 {
                low = mid;
                iterations += 1;
                continue;
            }

            let new_reserve_out = k / new_reserve_in;
            let resulting_price = new_reserve_out / new_reserve_in;

            let price_diff = (resulting_price - target_price).abs();

            if price_diff <= tolerance {
                return Ok(resulting_price);
            } else if resulting_price > target_price {
                low = mid;
            } else {
                high = mid;
            }

            iterations += 1;
        }

        // If we couldn't find an exact match, return the best approximation
        let mid = (low + high) / 2.0;
        let k = (reserve_in as f64) * (reserve_out as f64);
        let new_reserve_in = (reserve_in as f64) + mid;

        if new_reserve_in <= 0.0 {
            return Err(AMMError::PriceRangeNotFound);
        }

        let new_reserve_out = k / new_reserve_in;
        let resulting_price = new_reserve_out / new_reserve_in;

        // Check if this is within an acceptable range
        let price_diff = (resulting_price - target_price).abs();
        if price_diff <= tolerance * 10.0 {
            Ok(resulting_price)
        } else {
            Err(AMMError::PriceRangeNotFound)
        }
    }

    /// Check if a given price is within acceptable slippage range
    /// This is another implementation of price range checking using binary search concepts
    ///
    /// # Arguments
    /// * `from_token` - The token being sold
    /// * `to_token` - The token being bought
    /// * `proposed_price` - The proposed trade price
    /// * `max_slippage` - Maximum allowed slippage (as a ratio, e.g., 0.005 for 0.5%)
    ///
    /// # Returns
    /// * `Ok(bool)` - True if price is within slippage range, false otherwise
    /// * `Err(AMMError)` - On error
    pub fn is_price_within_slippage(
        &self,
        from_token: &TokenId,
        to_token: &TokenId,
        proposed_price: f64,
        max_slippage: f64,
    ) -> Result<bool, AMMError> {
        let current_price = self.get_price(from_token, to_token)?;

        // Calculate price impact
        let price_impact = ((current_price - proposed_price) / current_price).abs();

        Ok(price_impact <= max_slippage)
    }
}

/// Liquidity pool that manages paired token reserves for a specific market.
/// Implements Priority 4 feature:
/// - `4,Core Trading,AMM,AMM,Liquidity Pools (paired token reserves),Token Reserve Management,High`
#[derive(Debug, Clone)]
pub struct LiquidityPool {
    base_token: TokenId,
    quote_token: TokenId,
    reserve_base: Quantity,
    reserve_quote: Quantity,
    total_liquidity: Quantity,
    fee_bps: u32,
}

impl LiquidityPool {
    /// Create a new empty liquidity pool for the token pair.
    pub fn new(base_token: TokenId, quote_token: TokenId, fee_bps: u32) -> Self {
        Self {
            base_token,
            quote_token,
            reserve_base: 0,
            reserve_quote: 0,
            total_liquidity: 0,
            fee_bps,
        }
    }

    /// Current reserves (base, quote).
    pub fn reserves(&self) -> (Quantity, Quantity) {
        (self.reserve_base, self.reserve_quote)
    }

    /// Add balanced liquidity to the pool and mint LP shares.
    pub fn add_liquidity(
        &mut self,
        amount_base: Quantity,
        amount_quote: Quantity,
    ) -> Result<Quantity, AMMError> {
        if amount_base == 0 || amount_quote == 0 {
            return Err(AMMError::InsufficientLiquidity);
        }

        let minted = if self.total_liquidity == 0 {
            // Initial provider sets price; mint geometric mean of deposits.
            ((amount_base as f64 * amount_quote as f64).sqrt() as Quantity).max(1)
        } else {
            let liquidity_from_base = (amount_base * self.total_liquidity) / self.reserve_base;
            let liquidity_from_quote = (amount_quote * self.total_liquidity) / self.reserve_quote;
            liquidity_from_base.min(liquidity_from_quote)
        };

        if minted == 0 {
            return Err(AMMError::InsufficientLiquidity);
        }

        self.reserve_base = self.reserve_base.saturating_add(amount_base);
        self.reserve_quote = self.reserve_quote.saturating_add(amount_quote);
        self.total_liquidity = self.total_liquidity.saturating_add(minted);

        Ok(minted)
    }

    /// Remove liquidity and return underlying tokens.
    pub fn remove_liquidity(
        &mut self,
        lp_tokens: Quantity,
    ) -> Result<(Quantity, Quantity), AMMError> {
        if lp_tokens == 0 || lp_tokens > self.total_liquidity {
            return Err(AMMError::InsufficientLiquidity);
        }

        let amount_base = (lp_tokens * self.reserve_base) / self.total_liquidity;
        let amount_quote = (lp_tokens * self.reserve_quote) / self.total_liquidity;

        self.reserve_base = self.reserve_base.saturating_sub(amount_base);
        self.reserve_quote = self.reserve_quote.saturating_sub(amount_quote);
        self.total_liquidity = self.total_liquidity.saturating_sub(lp_tokens);

        Ok((amount_base, amount_quote))
    }

    /// Swap from one token to the other, applying pool fee.
    pub fn swap(&mut self, from_token: &TokenId, amount_in: Quantity) -> Result<Quantity, AMMError> {
        if amount_in == 0 {
            return Err(AMMError::InvalidToken);
        }

        let fee_multiplier = 10_000u128.saturating_sub(self.fee_bps as u128);

        if from_token == &self.base_token {
            if self.reserve_base == 0 || self.reserve_quote == 0 {
                return Err(AMMError::InsufficientLiquidity);
            }
            let amount_in_with_fee = (amount_in as u128 * fee_multiplier) / 10_000u128;
            let numerator = amount_in_with_fee * self.reserve_quote as u128;
            let denominator = (self.reserve_base as u128 * 10_000u128) + amount_in_with_fee;
            let amount_out = (numerator / denominator) as Quantity;
            if amount_out == 0 || amount_out >= self.reserve_quote {
                return Err(AMMError::InsufficientLiquidity);
            }

            self.reserve_base = self.reserve_base.saturating_add(amount_in);
            self.reserve_quote = self.reserve_quote.saturating_sub(amount_out);
            Ok(amount_out)
        } else if from_token == &self.quote_token {
            if self.reserve_base == 0 || self.reserve_quote == 0 {
                return Err(AMMError::InsufficientLiquidity);
            }
            let amount_in_with_fee = (amount_in as u128 * fee_multiplier) / 10_000u128;
            let numerator = amount_in_with_fee * self.reserve_base as u128;
            let denominator = (self.reserve_quote as u128 * 10_000u128) + amount_in_with_fee;
            let amount_out = (numerator / denominator) as Quantity;
            if amount_out == 0 || amount_out >= self.reserve_base {
                return Err(AMMError::InsufficientLiquidity);
            }

            self.reserve_quote = self.reserve_quote.saturating_add(amount_in);
            self.reserve_base = self.reserve_base.saturating_sub(amount_out);
            Ok(amount_out)
        } else {
            Err(AMMError::InvalidToken)
        }
    }

    /// Percentage ownership of the pool for a given LP token amount.
    pub fn share_of_pool(&self, lp_tokens: Quantity) -> Option<f64> {
        if self.total_liquidity == 0 || lp_tokens > self.total_liquidity {
            return None;
        }
        Some(lp_tokens as f64 / self.total_liquidity as f64)
    }

    /// Get the tokens that define this pool.
    pub fn tokens(&self) -> (&TokenId, &TokenId) {
        (&self.base_token, &self.quote_token)
    }

    /// Quote a swap without mutating pool reserves.
    pub fn quote_swap(&self, from_token: &TokenId, amount_in: Quantity) -> Result<Quantity, AMMError> {
        if amount_in == 0 {
            return Err(AMMError::InvalidToken);
        }
        let fee_multiplier = 10_000u128.saturating_sub(self.fee_bps as u128);

        if from_token == &self.base_token {
            if self.reserve_base == 0 || self.reserve_quote == 0 {
                return Err(AMMError::InsufficientLiquidity);
            }
            let amount_in_with_fee = (amount_in as u128 * fee_multiplier) / 10_000u128;
            let numerator = amount_in_with_fee * self.reserve_quote as u128;
            let denominator = (self.reserve_base as u128 * 10_000u128) + amount_in_with_fee;
            let amount_out = (numerator / denominator) as Quantity;
            if amount_out == 0 || amount_out >= self.reserve_quote {
                return Err(AMMError::InsufficientLiquidity);
            }
            Ok(amount_out)
        } else if from_token == &self.quote_token {
            if self.reserve_base == 0 || self.reserve_quote == 0 {
                return Err(AMMError::InsufficientLiquidity);
            }
            let amount_in_with_fee = (amount_in as u128 * fee_multiplier) / 10_000u128;
            let numerator = amount_in_with_fee * self.reserve_base as u128;
            let denominator = (self.reserve_quote as u128 * 10_000u128) + amount_in_with_fee;
            let amount_out = (numerator / denominator) as Quantity;
            if amount_out == 0 || amount_out >= self.reserve_base {
                return Err(AMMError::InsufficientLiquidity);
            }
            Ok(amount_out)
        } else {
            Err(AMMError::InvalidToken)
        }
    }
}

/// Router contract that executes multi-hop swaps across registered AMM pools.
/// Implements Priority 4 feature:
/// - `4,Core Trading,AMM,AMM,Router Contract for Execution,Multi-hop Routing,High`
#[derive(Debug, Default, Clone)]
pub struct RouterContract {
    pools: HashMap<(TokenId, TokenId), LiquidityPool>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RouterError {
    InvalidPath,
    ZeroInput,
    PoolNotFound { from: TokenId, to: TokenId },
    SlippageExceeded { min_expected: Quantity, actual: Quantity },
    SwapFailed(AMMError),
}

impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterError::InvalidPath => write!(f, "invalid routing path"),
            RouterError::ZeroInput => write!(f, "input amount must be greater than zero"),
            RouterError::PoolNotFound { from, to } => {
                write!(f, "pool not found for hop {} -> {}", from, to)
            }
            RouterError::SlippageExceeded { min_expected, actual } => write!(
                f,
                "slippage exceeded: expected at least {}, got {}",
                min_expected, actual
            ),
            RouterError::SwapFailed(err) => write!(f, "swap failed: {}", err),
        }
    }
}

impl std::error::Error for RouterError {}

impl From<AMMError> for RouterError {
    fn from(err: AMMError) -> Self {
        RouterError::SwapFailed(err)
    }
}

impl RouterContract {
    /// Create an empty router.
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
        }
    }

    /// Register or replace a pool keyed by its token pair (order-insensitive).
    pub fn register_pool(&mut self, pool: LiquidityPool) {
        let (a, b) = pool.tokens();
        self.pools.insert(key_for_pair(a, b), pool);
    }

    /// Execute a multi-hop swap along the provided token path.
    pub fn execute_path(
        &mut self,
        path: &[TokenId],
        amount_in: Quantity,
    ) -> Result<Quantity, RouterError> {
        self.execute_path_with_min_output(path, amount_in, 0)
    }

    /// Execute a multi-hop swap enforcing a minimum output (slippage guard).
    pub fn execute_path_with_min_output(
        &mut self,
        path: &[TokenId],
        amount_in: Quantity,
        min_amount_out: Quantity,
    ) -> Result<Quantity, RouterError> {
        if path.len() < 2 {
            return Err(RouterError::InvalidPath);
        }
        if amount_in == 0 {
            return Err(RouterError::ZeroInput);
        }

        // Stage swaps on a cloned pool set so we can abort without mutating on failure.
        let mut staged_pools = self.pools.clone();
        let mut current_amount = amount_in;

        for hop in path.windows(2) {
            let from = &hop[0];
            let to = &hop[1];
            let key = key_for_pair(from, to);
            let pool = staged_pools
                .get_mut(&key)
                .ok_or_else(|| RouterError::PoolNotFound {
                    from: from.clone(),
                    to: to.clone(),
                })?;

            current_amount = pool.swap(from, current_amount)?;
        }

        if current_amount < min_amount_out {
            return Err(RouterError::SlippageExceeded {
                min_expected: min_amount_out,
                actual: current_amount,
            });
        }

        // Commit staged state.
        self.pools = staged_pools;
        Ok(current_amount)
    }

    /// Returns pool reserves for inspection (base, quote) if registered.
    pub fn pool_reserves(&self, token_a: &TokenId, token_b: &TokenId) -> Option<(Quantity, Quantity)> {
        self.pools
            .get(&key_for_pair(token_a, token_b))
            .map(|p| p.reserves())
    }
}

fn key_for_pair(a: &TokenId, b: &TokenId) -> (TokenId, TokenId) {
    if a <= b {
        (a.clone(), b.clone())
    } else {
        (b.clone(), a.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amm_creation() {
        let amm = ConstantProductAMM::new(30); // 0.3% fee
        assert_eq!(amm.total_supply, 0);
        assert_eq!(amm.fee, 30);
        assert!(amm.reserves.is_empty());
        assert!(amm.ticks.is_empty());
        assert_eq!(amm.current_tick, 0);
        assert_eq!(amm.sqrt_price, 1.0);
    }

    #[test]
    fn test_add_liquidity() {
        let mut amm = ConstantProductAMM::new(30);
        let token_a = "BTC".to_string();
        let token_b = "USD".to_string();

        let liquidity_tokens = amm
            .add_liquidity(
                token_a.clone(),
                1000,
                token_b.clone(),
                50000000, // 50,000,000 USD (assuming 1 BTC = 50,000 USD)
            )
            .unwrap();

        assert!(liquidity_tokens > 0);
        assert_eq!(amm.total_supply, liquidity_tokens);
        assert_eq!(*amm.reserves.get(&token_a).unwrap(), 1000);
        assert_eq!(*amm.reserves.get(&token_b).unwrap(), 50000000);
    }

    #[test]
    fn test_add_liquidity_concentrated() {
        let mut amm = ConstantProductAMM::new(30);
        let token_a = "BTC".to_string();
        let token_b = "USD".to_string();

        let liquidity_tokens = amm
            .add_liquidity_concentrated(
                token_a.clone(),
                token_b.clone(),
                1000,
                50000000, // 50,000,000 USD
                -100,     // Lower tick
                100,      // Upper tick
            )
            .unwrap();

        assert!(liquidity_tokens > 0);
        assert_eq!(amm.total_supply, liquidity_tokens);
        assert_eq!(*amm.reserves.get(&token_a).unwrap(), 1000);
        assert_eq!(*amm.reserves.get(&token_b).unwrap(), 50000000);

        // Check that ticks were created
        assert!(amm.ticks.contains_key(&-100));
        assert!(amm.ticks.contains_key(&100));

        // Check liquidity at ticks
        for i in -99..100 {
            assert_eq!(amm.get_liquidity_at_tick(i), 1000);
        }
    }

    #[test]
    fn test_remove_liquidity_concentrated() {
        let mut amm = ConstantProductAMM::new(30);
        let token_a = "BTC".to_string();
        let token_b = "USD".to_string();

        // Add liquidity first
        let liquidity_tokens = amm
            .add_liquidity_concentrated(token_a.clone(), token_b.clone(), 1000, 50000000, -100, 100)
            .unwrap();

        // Remove some liquidity
        let removed_liquidity = liquidity_tokens / 2;
        let (amount_a, amount_b) = amm
            .remove_liquidity_concentrated(
                token_a.clone(),
                token_b.clone(),
                removed_liquidity,
                -100,
                100,
            )
            .unwrap();

        assert!(amount_a > 0);
        assert!(amount_b > 0);
        assert_eq!(amm.total_supply, liquidity_tokens - removed_liquidity);
    }

    #[test]
    fn test_get_liquidity_at_tick() {
        let mut amm = ConstantProductAMM::new(30);
        let token_a = "BTC".to_string();
        let token_b = "USD".to_string();

        // Add liquidity to specific ticks
        amm.add_liquidity_concentrated(token_a, token_b, 1000, 50000000, -50, 50)
            .unwrap();

        // Check liquidity at different ticks
        assert_eq!(amm.get_liquidity_at_tick(-50), 1000);
        assert_eq!(amm.get_liquidity_at_tick(0), 1000);
        assert_eq!(amm.get_liquidity_at_tick(49), 1000);
        assert_eq!(amm.get_liquidity_at_tick(50), 0); // Outside range
        assert_eq!(amm.get_liquidity_at_tick(-51), 0); // Outside range
    }

    #[test]
    fn test_get_active_ticks() {
        let mut amm = ConstantProductAMM::new(30);
        let token_a = "BTC".to_string();
        let token_b = "USD".to_string();

        // Add liquidity to specific ticks
        amm.add_liquidity_concentrated(token_a, token_b, 1000, 50000000, -10, 10)
            .unwrap();

        // Get active ticks
        let active_ticks = amm.get_active_ticks();
        assert_eq!(active_ticks.len(), 20); // Ticks -10 to 9 (10 is exclusive)

        // All active ticks should have liquidity
        for tick in active_ticks {
            assert!(tick.liquidity > 0);
        }
    }

    #[test]
    fn test_find_price_in_range() {
        let mut amm = ConstantProductAMM::new(30);
        let token_a = "DAI".to_string();
        let token_b = "USDC".to_string();

        // Add initial liquidity
        amm.add_liquidity(
            token_a.clone(),
            1000000, // 1,000,000 DAI
            token_b.clone(),
            1000000, // 1,000,000 USDC
        )
        .unwrap();

        // Test finding price in range
        let price = amm
            .find_price_in_range(&token_a, &token_b, 0.9, 1.1, 0.001)
            .unwrap();

        // Price should be close to 1.0 (since we have equal reserves)
        assert!(price > 0.9 && price < 1.1);
    }

    #[test]
    fn test_find_price_in_range_not_found() {
        let mut amm = ConstantProductAMM::new(30);
        let token_a = "DAI".to_string();
        let token_b = "USDC".to_string();

        // Add initial liquidity
        amm.add_liquidity(
            token_a.clone(),
            1000000, // 1,000,000 DAI
            token_b.clone(),
            1000000, // 1,000,000 USDC
        )
        .unwrap();

        // Test with impossible range
        let result = amm.find_price_in_range(&token_a, &token_b, 2.0, 3.0, 0.001);

        // Should return an error since we can't achieve such a high price
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AMMError::PriceRangeNotFound);
    }

    #[test]
    fn test_is_price_within_slippage() {
        let mut amm = ConstantProductAMM::new(30);
        let token_a = "DAI".to_string();
        let token_b = "USDC".to_string();

        // Add initial liquidity
        amm.add_liquidity(
            token_a.clone(),
            1000000, // 1,000,000 DAI
            token_b.clone(),
            1000000, // 1,000,000 USDC
        )
        .unwrap();

        // Test price within slippage
        let within_slippage = amm
            .is_price_within_slippage(
                &token_a, &token_b, 1.0,  // Current price is 1.0
                0.01, // 1% slippage allowed
            )
            .unwrap();

        assert!(within_slippage);

        // Test price outside slippage
        let within_slippage = amm
            .is_price_within_slippage(
                &token_a, &token_b, 1.5,  // Much higher than current price
                0.01, // 1% slippage allowed
            )
            .unwrap();

        assert!(!within_slippage);
    }

    #[test]
    fn liquidity_pool_adds_and_tracks_reserves() {
        let mut pool = LiquidityPool::new("ETH".into(), "USDC".into(), 30);
        let minted = pool.add_liquidity(10_000, 20_000_000).unwrap();
        assert!(minted > 0);
        let (base, quote) = pool.reserves();
        assert_eq!(base, 10_000);
        assert_eq!(quote, 20_000_000);
        assert_eq!(pool.total_liquidity, minted);
    }

    #[test]
    fn liquidity_pool_mints_proportional_shares_for_later_providers() {
        let mut pool = LiquidityPool::new("ETH".into(), "USDC".into(), 30);
        let first = pool.add_liquidity(1_000, 3_000_000).unwrap();
        let second = pool.add_liquidity(500, 1_500_000).unwrap();
        assert!(second > 0);
        // Second provider should receive half the shares of the first because deposits are proportional.
        assert_eq!(second, first / 2);
        let (base, quote) = pool.reserves();
        assert_eq!(base, 1_500);
        assert_eq!(quote, 4_500_000);
    }

    #[test]
    fn liquidity_pool_removes_liquidity_and_returns_tokens() {
        let mut pool = LiquidityPool::new("ETH".into(), "USDC".into(), 30);
        let minted = pool.add_liquidity(2_000, 6_000_000).unwrap();
        let half = minted / 2;
        let (base_out, quote_out) = pool.remove_liquidity(half).unwrap();
        assert!(base_out > 0 && quote_out > 0);
        let (base_reserve, quote_reserve) = pool.reserves();
        assert_eq!(base_reserve, 2_000 - base_out);
        assert_eq!(quote_reserve, 6_000_000 - quote_out);
        assert_eq!(pool.total_liquidity, minted - half);
    }

    #[test]
    fn liquidity_pool_swaps_base_for_quote_and_updates_reserves() {
        let mut pool = LiquidityPool::new("ETH".into(), "USDC".into(), 30);
        pool.add_liquidity(10_000, 20_000_000).unwrap();
        let quote_out = pool.swap(&"ETH".into(), 1_000).unwrap();
        assert!(quote_out > 0);
        let (base_reserve, quote_reserve) = pool.reserves();
        // Base reserve increases by input, quote reserve decreases by output.
        assert_eq!(base_reserve, 11_000);
        assert_eq!(quote_reserve, 20_000_000 - quote_out);
    }

    #[test]
    fn liquidity_pool_rejects_unknown_token_swaps() {
        let mut pool = LiquidityPool::new("ETH".into(), "USDC".into(), 30);
        pool.add_liquidity(1_000, 2_000_000).unwrap();
        let err = pool.swap(&"BTC".into(), 100).unwrap_err();
        assert_eq!(err, AMMError::InvalidToken);
    }

    #[test]
    fn router_executes_multi_hop_swap() {
        let mut router = RouterContract::new();

        let mut pool_eth_usdc = LiquidityPool::new("ETH".into(), "USDC".into(), 30);
        pool_eth_usdc.add_liquidity(10_000, 20_000_000).unwrap();
        router.register_pool(pool_eth_usdc);

        let mut pool_usdc_dai = LiquidityPool::new("USDC".into(), "DAI".into(), 10);
        pool_usdc_dai.add_liquidity(20_000_000, 20_000_000).unwrap();
        router.register_pool(pool_usdc_dai);

        let path = vec!["ETH".to_string(), "USDC".to_string(), "DAI".to_string()];
        let amount_out = router.execute_path(&path, 500).unwrap();

        assert!(amount_out > 0, "multi-hop output should be positive");

        // Verify intermediate pool reserves were updated.
        let (eth_reserve, usdc_reserve) = router.pool_reserves(&"ETH".into(), &"USDC".into()).unwrap();
        assert_eq!(eth_reserve, 10_500);
        assert!(usdc_reserve < 20_000_000);
    }

    #[test]
    fn router_errors_when_hop_pool_missing() {
        let mut router = RouterContract::new();
        let mut pool_eth_usdc = LiquidityPool::new("ETH".into(), "USDC".into(), 30);
        pool_eth_usdc.add_liquidity(1_000, 2_000_000).unwrap();
        router.register_pool(pool_eth_usdc);

        let path = vec!["ETH".to_string(), "USDC".to_string(), "DAI".to_string()];
        let err = router.execute_path(&path, 100).unwrap_err();
        matches!(err, RouterError::PoolNotFound { .. });
    }

    #[test]
    fn router_enforces_min_output_and_does_not_commit_on_failure() {
        let mut router = RouterContract::new();

        let mut pool_eth_usdc = LiquidityPool::new("ETH".into(), "USDC".into(), 30);
        pool_eth_usdc.add_liquidity(5_000, 10_000_000).unwrap();
        router.register_pool(pool_eth_usdc);

        let mut pool_usdc_dai = LiquidityPool::new("USDC".into(), "DAI".into(), 10);
        pool_usdc_dai.add_liquidity(10_000_000, 10_000_000).unwrap();
        router.register_pool(pool_usdc_dai);

        let path = vec!["ETH".to_string(), "USDC".to_string(), "DAI".to_string()];

        // Set an intentionally high min output to trigger slippage error.
        let err = router
            .execute_path_with_min_output(&path, 100, 1_000_000)
            .unwrap_err();
        matches!(err, RouterError::SlippageExceeded { .. });

        // State should be unchanged after the failed attempt.
        let (eth_reserve, usdc_reserve) = router.pool_reserves(&"ETH".into(), &"USDC".into()).unwrap();
        assert_eq!(eth_reserve, 5_000);
        assert_eq!(usdc_reserve, 10_000_000);

        // Now execute with a realistic minimum.
        let ok_out = router
            .execute_path_with_min_output(&path, 100, 1)
            .expect("swap should succeed with lenient min");
        assert!(ok_out > 0);
    }
}
