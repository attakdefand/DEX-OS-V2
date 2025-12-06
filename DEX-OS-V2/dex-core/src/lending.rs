//! Lending module for DEX-OS
//!
//! This module implements lending functionality including:
//! - Compound-style interest rate models
//! - Loan accounting and tracking systems
//!
//! This implements the Priority 2 feature from DEX-OS-V1.csv:
//! "2,Core Trading,Lending,Lending,Interest Rate Model,Compound-style Algorithm,High"
//! and
//! "2,Core Trading,Lending,Lending,Accounting System,Loan Tracking,High"

use std::collections::HashMap;
use std::fmt;

/// Error types for lending operations
#[derive(Debug, Clone, PartialEq)]
pub enum LendingError {
    InsufficientCollateral,
    InsufficientLiquidity,
    InvalidAmount,
    LoanNotFound,
    MathError,
}

impl fmt::Display for LendingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LendingError::InsufficientCollateral => write!(f, "Insufficient collateral"),
            LendingError::InsufficientLiquidity => write!(f, "Insufficient liquidity"),
            LendingError::InvalidAmount => write!(f, "Invalid amount"),
            LendingError::LoanNotFound => write!(f, "Loan not found"),
            LendingError::MathError => write!(f, "Mathematical error occurred"),
        }
    }
}

impl std::error::Error for LendingError {}

/// Represents different asset types that can be lent or borrowed
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssetType {
    Token(String),
    Stablecoin(String),
    NFT(String),
}

/// Interest rate model based on Compound Finance v2
///
/// This model uses the formula:
/// - Base Rate + (Utilization Rate * Multiplier)
/// Where:
/// - Utilization Rate = Total Borrows / Total Supply
pub struct CompoundInterestRateModel {
    /// Base interest rate when utilization is 0%
    pub base_rate: f64,
    /// Multiplier for the slope of the interest rate
    pub multiplier: f64,
    /// Maximum utilization rate (usually 100%)
    pub max_utilization: f64,
    /// Kink utilization rate where the slope changes
    pub kink_utilization: f64,
    /// Multiplier after kink utilization
    pub kink_multiplier: f64,
}

impl CompoundInterestRateModel {
    /// Create a new Compound-style interest rate model
    pub fn new(
        base_rate: f64,
        multiplier: f64,
        max_utilization: f64,
        kink_utilization: f64,
        kink_multiplier: f64,
    ) -> Self {
        Self {
            base_rate,
            multiplier,
            max_utilization,
            kink_utilization,
            kink_multiplier,
        }
    }

    /// Calculate the borrow interest rate based on utilization
    ///
    /// This implements the Compound-style algorithm for interest rate calculation
    pub fn calculate_borrow_rate(
        &self,
        cash: f64,
        borrows: f64,
        reserves: f64,
    ) -> Result<f64, LendingError> {
        if cash < 0.0 || borrows < 0.0 || reserves < 0.0 {
            return Err(LendingError::InvalidAmount);
        }

        let util = if cash + borrows == 0.0 {
            0.0
        } else {
            borrows / (cash + borrows)
        };

        // Ensure utilization doesn't exceed max
        let utilization = util.min(self.max_utilization);

        let rate = if utilization <= self.kink_utilization {
            self.base_rate + utilization * self.multiplier
        } else {
            let normal_rate = self.base_rate + self.kink_utilization * self.multiplier;
            normal_rate + (utilization - self.kink_utilization) * self.kink_multiplier
        };

        Ok(rate)
    }

    /// Calculate the supply interest rate based on borrow rate and utilization
    pub fn calculate_supply_rate(
        &self,
        borrow_rate: f64,
        cash: f64,
        borrows: f64,
        reserves: f64,
        reserve_factor: f64,
    ) -> Result<f64, LendingError> {
        if cash < 0.0
            || borrows < 0.0
            || reserves < 0.0
            || reserve_factor < 0.0
            || reserve_factor > 1.0
        {
            return Err(LendingError::InvalidAmount);
        }

        let util = if cash + borrows == 0.0 {
            0.0
        } else {
            borrows / (cash + borrows)
        };

        let one_minus_reserve_factor = 1.0 - reserve_factor;
        let rate = borrow_rate * util * one_minus_reserve_factor;

        Ok(rate)
    }
}

/// Represents a loan position
#[derive(Debug, Clone)]
pub struct Loan {
    /// Unique identifier for the loan
    pub id: String,
    /// Borrower's address or identifier
    pub borrower: String,
    /// Asset being borrowed
    pub asset: AssetType,
    /// Amount borrowed
    pub principal: f64,
    /// Current amount owed (principal + interest)
    pub amount_owed: f64,
    /// Collateral asset
    pub collateral_asset: AssetType,
    /// Amount of collateral
    pub collateral_amount: f64,
    /// Timestamp when loan was created
    pub created_at: u64,
    /// Timestamp when loan is due
    pub due_at: u64,
    /// Interest rate for this loan
    pub interest_rate: f64,
    /// Health factor of the loan (collateral value / loan value)
    pub health_factor: f64,
    /// Status of the loan
    pub status: LoanStatus,
}

/// Status of a loan
#[derive(Debug, Clone, PartialEq)]
pub enum LoanStatus {
    Active,
    Repaid,
    Liquidated,
    Defaulted,
}

/// Accounting system for tracking loans
pub struct LoanAccountingSystem {
    /// All loans indexed by ID
    loans: HashMap<String, Loan>,
    /// Interest rate model used for calculations
    interest_rate_model: CompoundInterestRateModel,
    /// Reserve factor for the protocol
    reserve_factor: f64,
    /// Total value of assets supplied to the protocol
    total_supply: HashMap<AssetType, f64>,
    /// Total value of assets borrowed from the protocol
    total_borrows: HashMap<AssetType, f64>,
    /// Protocol reserves
    total_reserves: HashMap<AssetType, f64>,
}

impl LoanAccountingSystem {
    /// Create a new loan accounting system
    pub fn new(interest_rate_model: CompoundInterestRateModel, reserve_factor: f64) -> Self {
        Self {
            loans: HashMap::new(),
            interest_rate_model,
            reserve_factor,
            total_supply: HashMap::new(),
            total_borrows: HashMap::new(),
            total_reserves: HashMap::new(),
        }
    }

    /// Create a new loan
    pub fn create_loan(
        &mut self,
        id: String,
        borrower: String,
        asset: AssetType,
        amount: f64,
        collateral_asset: AssetType,
        collateral_amount: f64,
        created_at: u64,
        due_at: u64,
    ) -> Result<String, LendingError> {
        if amount <= 0.0 || collateral_amount <= 0.0 {
            return Err(LendingError::InvalidAmount);
        }

        // Check if there's enough liquidity
        let available_liquidity = self.get_available_liquidity(&asset);
        if amount > available_liquidity {
            return Err(LendingError::InsufficientLiquidity);
        }

        // Calculate interest rate based on current utilization
        let cash = *self.total_supply.get(&asset).unwrap_or(&0.0);
        let borrows = *self.total_borrows.get(&asset).unwrap_or(&0.0);
        let reserves = *self.total_reserves.get(&asset).unwrap_or(&0.0);

        let interest_rate = self
            .interest_rate_model
            .calculate_borrow_rate(cash, borrows, reserves)?;

        // Create loan
        let loan = Loan {
            id: id.clone(),
            borrower,
            asset: asset.clone(),
            principal: amount,
            amount_owed: amount,
            collateral_asset,
            collateral_amount,
            created_at,
            due_at,
            interest_rate,
            health_factor: 1.5, // Default value, should be calculated based on actual prices
            status: LoanStatus::Active,
        };

        // Update accounting
        *self.total_borrows.entry(asset.clone()).or_insert(0.0) += amount;

        // Store loan
        self.loans.insert(id.clone(), loan);

        Ok(id)
    }

    /// Repay a loan
    pub fn repay_loan(&mut self, loan_id: &str, amount: f64) -> Result<(), LendingError> {
        let loan = self
            .loans
            .get_mut(loan_id)
            .ok_or(LendingError::LoanNotFound)?;

        if amount <= 0.0 {
            return Err(LendingError::InvalidAmount);
        }

        if loan.status != LoanStatus::Active {
            return Err(LendingError::LoanNotFound);
        }

        // Update amount owed
        loan.amount_owed -= amount;

        // If fully repaid, update status
        if loan.amount_owed <= 0.0 {
            loan.status = LoanStatus::Repaid;
            *self.total_borrows.entry(loan.asset.clone()).or_insert(0.0) -= loan.principal;
        }

        Ok(())
    }

    /// Liquidate a loan that has fallen below health threshold
    pub fn liquidate_loan(&mut self, loan_id: &str) -> Result<(), LendingError> {
        let loan = self
            .loans
            .get_mut(loan_id)
            .ok_or(LendingError::LoanNotFound)?;

        if loan.status != LoanStatus::Active {
            return Err(LendingError::LoanNotFound);
        }

        // Mark as liquidated
        loan.status = LoanStatus::Liquidated;
        *self.total_borrows.entry(loan.asset.clone()).or_insert(0.0) -= loan.principal;

        Ok(())
    }

    /// Get a loan by ID
    pub fn get_loan(&self, loan_id: &str) -> Option<&Loan> {
        self.loans.get(loan_id)
    }

    /// Get all loans for a borrower
    pub fn get_loans_for_borrower(&self, borrower: &str) -> Vec<&Loan> {
        self.loans
            .values()
            .filter(|loan| loan.borrower == borrower)
            .collect()
    }

    /// Calculate interest accrued for a loan
    pub fn calculate_interest(
        &self,
        loan_id: &str,
        current_time: u64,
    ) -> Result<f64, LendingError> {
        let loan = self.loans.get(loan_id).ok_or(LendingError::LoanNotFound)?;

        if loan.status != LoanStatus::Active {
            return Ok(0.0);
        }

        // Simple interest calculation: principal * rate * time
        // In a real implementation, this would use compounding
        let time_elapsed = (current_time - loan.created_at) as f64 / 31536000.0; // Convert to years
        let interest = loan.principal * loan.interest_rate * time_elapsed;

        Ok(interest)
    }

    /// Update loan with accrued interest
    pub fn accrue_interest(
        &mut self,
        loan_id: &str,
        current_time: u64,
    ) -> Result<(), LendingError> {
        let interest = self.calculate_interest(loan_id, current_time)?;
        let loan = self
            .loans
            .get_mut(loan_id)
            .ok_or(LendingError::LoanNotFound)?;
        loan.amount_owed = loan.principal + interest;
        Ok(())
    }

    /// Get available liquidity for an asset
    pub fn get_available_liquidity(&self, asset: &AssetType) -> f64 {
        let supply = *self.total_supply.get(asset).unwrap_or(&0.0);
        let borrows = *self.total_borrows.get(asset).unwrap_or(&0.0);
        supply - borrows
    }

    /// Get utilization rate for an asset
    pub fn get_utilization_rate(&self, asset: &AssetType) -> f64 {
        let cash = *self.total_supply.get(asset).unwrap_or(&0.0);
        let borrows = *self.total_borrows.get(asset).unwrap_or(&0.0);

        if cash + borrows == 0.0 {
            0.0
        } else {
            borrows / (cash + borrows)
        }
    }

    /// Get borrow rate for an asset
    pub fn get_borrow_rate(&self, asset: &AssetType) -> Result<f64, LendingError> {
        let cash = *self.total_supply.get(asset).unwrap_or(&0.0);
        let borrows = *self.total_borrows.get(asset).unwrap_or(&0.0);
        let reserves = *self.total_reserves.get(asset).unwrap_or(&0.0);

        self.interest_rate_model
            .calculate_borrow_rate(cash, borrows, reserves)
    }

    /// Get supply rate for an asset
    pub fn get_supply_rate(&self, asset: &AssetType) -> Result<f64, LendingError> {
        let borrow_rate = self.get_borrow_rate(asset)?;
        let cash = *self.total_supply.get(asset).unwrap_or(&0.0);
        let borrows = *self.total_borrows.get(asset).unwrap_or(&0.0);
        let reserves = *self.total_reserves.get(asset).unwrap_or(&0.0);

        self.interest_rate_model.calculate_supply_rate(
            borrow_rate,
            cash,
            borrows,
            reserves,
            self.reserve_factor,
        )
    }

    /// Supply assets to the protocol
    pub fn supply_assets(&mut self, asset: AssetType, amount: f64) -> Result<(), LendingError> {
        if amount <= 0.0 {
            return Err(LendingError::InvalidAmount);
        }

        *self.total_supply.entry(asset).or_insert(0.0) += amount;
        Ok(())
    }

    /// Withdraw supplied assets from the protocol
    pub fn withdraw_assets(&mut self, asset: AssetType, amount: f64) -> Result<(), LendingError> {
        if amount <= 0.0 {
            return Err(LendingError::InvalidAmount);
        }

        let available = self.get_available_liquidity(&asset);
        if amount > available {
            return Err(LendingError::InsufficientLiquidity);
        }

        *self.total_supply.entry(asset).or_insert(0.0) -= amount;
        Ok(())
    }

    /// Get total supply for an asset
    pub fn get_total_supply(&self, asset: &AssetType) -> f64 {
        *self.total_supply.get(asset).unwrap_or(&0.0)
    }

    /// Get total borrows for an asset
    pub fn get_total_borrows(&self, asset: &AssetType) -> f64 {
        *self.total_borrows.get(asset).unwrap_or(&0.0)
    }

    /// Get total reserves for an asset
    pub fn get_total_reserves(&self, asset: &AssetType) -> f64 {
        *self.total_reserves.get(asset).unwrap_or(&0.0)
    }

    /// Calculate the health factor for a loan
    ///
    /// Health factor = (Collateral Value * Liquidation Threshold) / Loan Value
    /// This implements the Priority 2 feature from DEX-OS-V1.csv:
    /// "2,Core Trading,Lending,Lending,Health Factor Calculation,Liquidation Prevention,High"
    pub fn calculate_health_factor(
        &self,
        loan_id: &str,
        collateral_price: f64,
        loan_asset_price: f64,
        liquidation_threshold: f64,
    ) -> Result<f64, LendingError> {
        let loan = self.loans.get(loan_id).ok_or(LendingError::LoanNotFound)?;

        if collateral_price <= 0.0
            || loan_asset_price <= 0.0
            || liquidation_threshold <= 0.0
            || liquidation_threshold > 1.0
        {
            return Err(LendingError::InvalidAmount);
        }

        if loan.status != LoanStatus::Active {
            return Err(LendingError::LoanNotFound);
        }

        let collateral_value = loan.collateral_amount * collateral_price;
        let loan_value = loan.amount_owed * loan_asset_price;

        if loan_value == 0.0 {
            return Ok(f64::INFINITY);
        }

        let health_factor = (collateral_value * liquidation_threshold) / loan_value;
        Ok(health_factor)
    }

    /// Update the health factor for a loan
    ///
    /// This function updates the health factor stored in the loan object
    pub fn update_health_factor(
        &mut self,
        loan_id: &str,
        collateral_price: f64,
        loan_asset_price: f64,
        liquidation_threshold: f64,
    ) -> Result<(), LendingError> {
        let health_factor = self.calculate_health_factor(
            loan_id,
            collateral_price,
            loan_asset_price,
            liquidation_threshold,
        )?;
        let loan = self
            .loans
            .get_mut(loan_id)
            .ok_or(LendingError::LoanNotFound)?;
        loan.health_factor = health_factor;
        Ok(())
    }

    /// Check if a loan should be liquidated based on health factor
    ///
    /// Returns true if the health factor is below the liquidation threshold
    pub fn should_liquidate(
        &self,
        loan_id: &str,
        collateral_price: f64,
        loan_asset_price: f64,
        liquidation_threshold: f64,
        min_health_factor: f64,
    ) -> Result<bool, LendingError> {
        let health_factor = self.calculate_health_factor(
            loan_id,
            collateral_price,
            loan_asset_price,
            liquidation_threshold,
        )?;
        Ok(health_factor < min_health_factor)
    }

    /// Get loans that are below the minimum health factor
    ///
    /// Returns a vector of loan IDs that need to be liquidated
    pub fn get_undercollateralized_loans(
        &self,
        collateral_prices: &HashMap<AssetType, f64>,
        loan_asset_prices: &HashMap<AssetType, f64>,
        liquidation_threshold: f64,
        min_health_factor: f64,
    ) -> Vec<String> {
        let mut undercollateralized_loans = Vec::new();

        for (loan_id, loan) in &self.loans {
            if loan.status != LoanStatus::Active {
                continue;
            }

            let collateral_price = match collateral_prices.get(&loan.collateral_asset) {
                Some(price) => *price,
                None => continue,
            };

            let loan_asset_price = match loan_asset_prices.get(&loan.asset) {
                Some(price) => *price,
                None => continue,
            };

            if let Ok(should_liquidate) = self.should_liquidate(
                loan_id,
                collateral_price,
                loan_asset_price,
                liquidation_threshold,
                min_health_factor,
            ) {
                if should_liquidate {
                    undercollateralized_loans.push(loan_id.clone());
                }
            }
        }

        undercollateralized_loans
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compound_interest_rate_model() {
        // Create a model similar to Compound's USDC market
        let model = CompoundInterestRateModel::new(
            0.02, // 2% base rate
            0.1,  // 10% multiplier before kink
            1.0,  // 100% max utilization
            0.8,  // 80% kink
            0.5,  // 50% multiplier after kink
        );

        // Test at 0% utilization
        let rate = model.calculate_borrow_rate(1000.0, 0.0, 0.0).unwrap();
        assert_eq!(rate, 0.02); // Should equal base rate

        // Test at 50% utilization (before kink)
        let rate = model.calculate_borrow_rate(500.0, 500.0, 0.0).unwrap();
        assert_eq!(rate, 0.07); // 2% + (50% * 10%) = 7%

        // Test at 90% utilization (after kink)
        let rate = model.calculate_borrow_rate(100.0, 900.0, 0.0).unwrap();
        let expected = 0.02 + (0.8 * 0.1) + (0.1 * 0.5); // 2% + 8% + 5% = 15%
        assert_eq!(rate, expected);
    }

    #[test]
    fn test_supply_rate_calculation() {
        let model = CompoundInterestRateModel::new(0.02, 0.1, 1.0, 0.8, 0.5);

        let borrow_rate = 0.07; // 7%
        let supply_rate = model
            .calculate_supply_rate(borrow_rate, 500.0, 500.0, 0.0, 0.1)
            .unwrap();
        let expected = 0.07 * 0.5 * 0.9; // 7% * 50% utilization * 90% (1 - 10% reserve factor)
        assert_eq!(supply_rate, expected);
    }

    #[test]
    fn test_loan_accounting_system() {
        let interest_model = CompoundInterestRateModel::new(0.02, 0.1, 1.0, 0.8, 0.5);

        let mut accounting = LoanAccountingSystem::new(interest_model, 0.1);

        // Supply some assets first
        accounting
            .supply_assets(AssetType::Token("USDC".to_string()), 1000.0)
            .unwrap();

        // Create a loan
        let loan_id = accounting
            .create_loan(
                "loan1".to_string(),
                "borrower1".to_string(),
                AssetType::Token("USDC".to_string()),
                100.0,
                AssetType::Token("ETH".to_string()),
                0.5,
                1000000,
                10086400, // 100 days later
            )
            .unwrap();

        assert_eq!(loan_id, "loan1");

        // Check loan exists
        let loan = accounting.get_loan("loan1").unwrap();
        assert_eq!(loan.borrower, "borrower1");
        assert_eq!(loan.principal, 100.0);
        assert_eq!(loan.status, LoanStatus::Active);

        // Check accounting
        assert_eq!(
            accounting.get_total_supply(&AssetType::Token("USDC".to_string())),
            1000.0
        );
        assert_eq!(
            accounting.get_total_borrows(&AssetType::Token("USDC".to_string())),
            100.0
        );
        assert_eq!(
            accounting.get_available_liquidity(&AssetType::Token("USDC".to_string())),
            900.0
        );
    }

    #[test]
    fn test_repay_loan() {
        let interest_model = CompoundInterestRateModel::new(0.02, 0.1, 1.0, 0.8, 0.5);

        let mut accounting = LoanAccountingSystem::new(interest_model, 0.1);

        // Supply assets
        accounting
            .supply_assets(AssetType::Token("USDC".to_string()), 1000.0)
            .unwrap();

        // Create a loan
        accounting
            .create_loan(
                "loan1".to_string(),
                "borrower1".to_string(),
                AssetType::Token("USDC".to_string()),
                100.0,
                AssetType::Token("ETH".to_string()),
                0.5,
                1000000,
                10086400,
            )
            .unwrap();

        // Repay part of the loan
        accounting.repay_loan("loan1", 50.0).unwrap();

        let loan = accounting.get_loan("loan1").unwrap();
        assert_eq!(loan.amount_owed, 50.0);
        assert_eq!(loan.status, LoanStatus::Active);

        // Repay remaining amount
        accounting.repay_loan("loan1", 50.0).unwrap();

        let loan = accounting.get_loan("loan1").unwrap();
        assert_eq!(loan.amount_owed, 0.0);
        assert_eq!(loan.status, LoanStatus::Repaid);
    }

    #[test]
    fn test_liquidate_loan() {
        let interest_model = CompoundInterestRateModel::new(0.02, 0.1, 1.0, 0.8, 0.5);

        let mut accounting = LoanAccountingSystem::new(interest_model, 0.1);

        // Supply assets
        accounting
            .supply_assets(AssetType::Token("USDC".to_string()), 1000.0)
            .unwrap();

        // Create a loan
        accounting
            .create_loan(
                "loan1".to_string(),
                "borrower1".to_string(),
                AssetType::Token("USDC".to_string()),
                100.0,
                AssetType::Token("ETH".to_string()),
                0.5,
                1000000,
                10086400,
            )
            .unwrap();

        // Liquidate the loan
        accounting.liquidate_loan("loan1").unwrap();

        let loan = accounting.get_loan("loan1").unwrap();
        assert_eq!(loan.status, LoanStatus::Liquidated);
    }

    #[test]
    fn test_error_cases() {
        let interest_model = CompoundInterestRateModel::new(0.02, 0.1, 1.0, 0.8, 0.5);

        let mut accounting = LoanAccountingSystem::new(interest_model, 0.1);

        // Try to create loan without sufficient liquidity
        let result = accounting.create_loan(
            "loan1".to_string(),
            "borrower1".to_string(),
            AssetType::Token("USDC".to_string()),
            100.0,
            AssetType::Token("ETH".to_string()),
            0.5,
            1000000,
            10086400,
        );

        assert_eq!(result, Err(LendingError::InsufficientLiquidity));

        // Try to repay non-existent loan
        let result = accounting.repay_loan("nonexistent", 50.0);
        assert_eq!(result, Err(LendingError::LoanNotFound));
    }

    #[test]
    fn test_health_factor_calculation() {
        let interest_model = CompoundInterestRateModel::new(0.02, 0.1, 1.0, 0.8, 0.5);

        let mut accounting = LoanAccountingSystem::new(interest_model, 0.1);

        // Supply assets
        accounting
            .supply_assets(AssetType::Token("USDC".to_string()), 1000.0)
            .unwrap();

        // Create a loan
        accounting
            .create_loan(
                "loan1".to_string(),
                "borrower1".to_string(),
                AssetType::Token("USDC".to_string()),
                100.0,
                AssetType::Token("ETH".to_string()),
                0.5,
                1000000,
                10086400,
            )
            .unwrap();

        // Calculate health factor
        let health_factor = accounting
            .calculate_health_factor(
                "loan1", 2000.0, // ETH price: $2000
                1.0,    // USDC price: $1
                0.8,    // 80% liquidation threshold
            )
            .unwrap();

        // Expected: (0.5 * 2000 * 0.8) / (100 * 1) = 800 / 100 = 8.0
        assert_eq!(health_factor, 8.0);
    }

    #[test]
    fn test_health_factor_update() {
        let interest_model = CompoundInterestRateModel::new(0.02, 0.1, 1.0, 0.8, 0.5);

        let mut accounting = LoanAccountingSystem::new(interest_model, 0.1);

        // Supply assets
        accounting
            .supply_assets(AssetType::Token("USDC".to_string()), 1000.0)
            .unwrap();

        // Create a loan
        accounting
            .create_loan(
                "loan1".to_string(),
                "borrower1".to_string(),
                AssetType::Token("USDC".to_string()),
                100.0,
                AssetType::Token("ETH".to_string()),
                0.5,
                1000000,
                10086400,
            )
            .unwrap();

        // Update health factor
        accounting
            .update_health_factor(
                "loan1", 2000.0, // ETH price: $2000
                1.0,    // USDC price: $1
                0.8,    // 80% liquidation threshold
            )
            .unwrap();

        let loan = accounting.get_loan("loan1").unwrap();
        assert_eq!(loan.health_factor, 8.0);
    }

    #[test]
    fn test_should_liquidate() {
        let interest_model = CompoundInterestRateModel::new(0.02, 0.1, 1.0, 0.8, 0.5);

        let mut accounting = LoanAccountingSystem::new(interest_model, 0.1);

        // Supply assets
        accounting
            .supply_assets(AssetType::Token("USDC".to_string()), 1000.0)
            .unwrap();

        // Create a loan
        accounting
            .create_loan(
                "loan1".to_string(),
                "borrower1".to_string(),
                AssetType::Token("USDC".to_string()),
                100.0,
                AssetType::Token("ETH".to_string()),
                0.05, // Only 0.05 ETH as collateral
                1000000,
                10086400,
            )
            .unwrap();

        // Check if should liquidate (health factor should be < 1.0)
        let should_liquidate = accounting
            .should_liquidate(
                "loan1", 2000.0, // ETH price: $2000
                1.0,    // USDC price: $1
                0.8,    // 80% liquidation threshold
                1.0,    // Minimum health factor
            )
            .unwrap();

        // Expected health factor: (0.05 * 2000 * 0.8) / (100 * 1) = 80 / 100 = 0.8
        // Since 0.8 < 1.0, should liquidate
        assert!(should_liquidate);
    }

    #[test]
    fn test_get_undercollateralized_loans() {
        let interest_model = CompoundInterestRateModel::new(0.02, 0.1, 1.0, 0.8, 0.5);

        let mut accounting = LoanAccountingSystem::new(interest_model, 0.1);

        // Supply assets
        accounting
            .supply_assets(AssetType::Token("USDC".to_string()), 1000.0)
            .unwrap();
        accounting
            .supply_assets(AssetType::Token("DAI".to_string()), 1000.0)
            .unwrap();

        // Create a healthy loan
        accounting
            .create_loan(
                "loan1".to_string(),
                "borrower1".to_string(),
                AssetType::Token("USDC".to_string()),
                100.0,
                AssetType::Token("ETH".to_string()),
                0.5,
                1000000,
                10086400,
            )
            .unwrap();

        // Create an undercollateralized loan
        accounting
            .create_loan(
                "loan2".to_string(),
                "borrower2".to_string(),
                AssetType::Token("DAI".to_string()),
                500.0,
                AssetType::Token("ETH".to_string()),
                0.1,
                1000000,
                10086400,
            )
            .unwrap();

        let mut collateral_prices = HashMap::new();
        collateral_prices.insert(AssetType::Token("ETH".to_string()), 2000.0);

        let mut loan_asset_prices = HashMap::new();
        loan_asset_prices.insert(AssetType::Token("USDC".to_string()), 1.0);
        loan_asset_prices.insert(AssetType::Token("DAI".to_string()), 1.0);

        let undercollateralized = accounting.get_undercollateralized_loans(
            &collateral_prices,
            &loan_asset_prices,
            0.8, // 80% liquidation threshold
            1.0, // Minimum health factor
        );

        // Only loan2 should be undercollateralized
        assert_eq!(undercollateralized.len(), 1);
        assert_eq!(undercollateralized[0], "loan2");
    }
}
