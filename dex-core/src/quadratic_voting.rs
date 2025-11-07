//! Quadratic Voting implementation for the DEX-OS core engine
//!
//! This module implements the Priority 3 feature from DEX-OS-V2.csv:
//! - Core Trading,Governance,Governance,Quadratic Voting,Decision Making,Medium
//!
//! It provides functionality for quadratic voting in governance decisions.

use crate::types::TraderId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Quadratic Voting system
#[derive(Debug, Clone)]
pub struct QuadraticVoting {
    /// Vote credits available to each voter
    credits: HashMap<TraderId, u64>,
    /// Votes cast in current proposal
    votes: HashMap<TraderId, QuadraticVote>,
    /// Total votes for option A
    votes_for_a: u64,
    /// Total votes for option B
    votes_against_b: u64,
}

/// A quadratic vote
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuadraticVote {
    /// Number of votes cast
    pub votes: u64,
    /// Cost in credits (votes^2)
    pub cost: u64,
}

impl QuadraticVoting {
    /// Create a new quadratic voting system
    pub fn new() -> Self {
        Self {
            credits: HashMap::new(),
            votes: HashMap::new(),
            votes_for_a: 0,
            votes_against_b: 0,
        }
    }

    /// Allocate vote credits to a voter
    pub fn allocate_credits(&mut self, voter: TraderId, credits: u64) {
        self.credits.insert(voter, credits);
    }

    /// Cast a quadratic vote
    pub fn cast_vote(
        &mut self,
        voter: TraderId,
        votes: u64,
        support: bool,
    ) -> Result<(), QuadraticVotingError> {
        // Check if voter has sufficient credits
        let available_credits = *self.credits.get(&voter).unwrap_or(&0);
        let cost = votes * votes;

        if cost > available_credits {
            return Err(QuadraticVotingError::InsufficientCredits);
        }

        // Check if voter has already voted
        if self.votes.contains_key(&voter) {
            return Err(QuadraticVotingError::AlreadyVoted);
        }

        // Deduct credits
        let remaining_credits = available_credits - cost;
        self.credits.insert(voter.clone(), remaining_credits);

        // Record vote
        let vote = QuadraticVote { votes, cost };
        self.votes.insert(voter.clone(), vote);

        // Add to totals
        if support {
            self.votes_for_a += votes;
        } else {
            self.votes_against_b += votes;
        }

        Ok(())
    }

    /// Get voting results
    pub fn get_results(&self) -> QuadraticVotingResults {
        QuadraticVotingResults {
            votes_for: self.votes_for_a,
            votes_against: self.votes_against_b,
            total_voters: self.votes.len(),
            total_credits_spent: self.votes.values().map(|v| v.cost).sum(),
        }
    }

    /// Get voter's remaining credits
    pub fn get_remaining_credits(&self, voter: &TraderId) -> u64 {
        *self.credits.get(voter).unwrap_or(&0)
    }

    /// Get voter's cast vote
    pub fn get_vote(&self, voter: &TraderId) -> Option<&QuadraticVote> {
        self.votes.get(voter)
    }

    /// Reset voting for a new proposal
    pub fn reset(&mut self) {
        self.votes.clear();
        self.votes_for_a = 0;
        self.votes_against_b = 0;
    }
}

impl Default for QuadraticVoting {
    fn default() -> Self {
        Self::new()
    }
}

/// Results of a quadratic voting session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuadraticVotingResults {
    /// Total votes for option A
    pub votes_for: u64,
    /// Total votes against option B
    pub votes_against: u64,
    /// Number of voters who participated
    pub total_voters: usize,
    /// Total credits spent by all voters
    pub total_credits_spent: u64,
}

/// Errors that can occur during quadratic voting
#[derive(Debug, Error)]
pub enum QuadraticVotingError {
    #[error("Insufficient vote credits")]
    InsufficientCredits,
    #[error("Voter has already cast a vote")]
    AlreadyVoted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadratic_voting_creation() {
        let qv = QuadraticVoting::new();
        assert!(qv.credits.is_empty());
        assert!(qv.votes.is_empty());
        assert_eq!(qv.votes_for_a, 0);
        assert_eq!(qv.votes_against_b, 0);
    }

    #[test]
    fn test_allocate_credits() {
        let mut qv = QuadraticVoting::new();
        let voter = "voter1".to_string();

        qv.allocate_credits(voter.clone(), 100);
        assert_eq!(qv.get_remaining_credits(&voter), 100);
    }

    #[test]
    fn test_cast_vote() {
        let mut qv = QuadraticVoting::new();
        let voter = "voter1".to_string();

        // Allocate credits
        qv.allocate_credits(voter.clone(), 100);

        // Cast vote
        assert!(qv.cast_vote(voter.clone(), 5, true).is_ok());

        // Check results
        assert_eq!(qv.votes_for_a, 5);
        assert_eq!(qv.votes_against_b, 0);
        assert_eq!(qv.get_remaining_credits(&voter), 75); // 100 - (5*5)

        // Check stored vote
        let vote = qv.get_vote(&voter).unwrap();
        assert_eq!(vote.votes, 5);
        assert_eq!(vote.cost, 25);
    }

    #[test]
    fn test_cast_vote_insufficient_credits() {
        let mut qv = QuadraticVoting::new();
        let voter = "voter1".to_string();

        // Allocate insufficient credits
        qv.allocate_credits(voter.clone(), 10);

        // Try to cast vote that costs more than available credits
        let result = qv.cast_vote(voter.clone(), 5, true); // Cost: 25 credits
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            QuadraticVotingError::InsufficientCredits
        ));
    }

    #[test]
    fn test_cast_vote_already_voted() {
        let mut qv = QuadraticVoting::new();
        let voter = "voter1".to_string();

        // Allocate credits
        qv.allocate_credits(voter.clone(), 100);

        // Cast first vote
        assert!(qv.cast_vote(voter.clone(), 3, true).is_ok());

        // Try to cast second vote
        let result = qv.cast_vote(voter.clone(), 2, false);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            QuadraticVotingError::AlreadyVoted
        ));
    }

    #[test]
    fn test_get_results() {
        let mut qv = QuadraticVoting::new();

        // Allocate credits to voters
        qv.allocate_credits("voter1".to_string(), 100);
        qv.allocate_credits("voter2".to_string(), 100);
        qv.allocate_credits("voter3".to_string(), 100);

        // Cast votes
        assert!(qv.cast_vote("voter1".to_string(), 4, true).is_ok()); // Cost: 16
        assert!(qv.cast_vote("voter2".to_string(), 3, true).is_ok()); // Cost: 9
        assert!(qv.cast_vote("voter3".to_string(), 2, false).is_ok()); // Cost: 4

        let results = qv.get_results();
        assert_eq!(results.votes_for, 7); // 4 + 3
        assert_eq!(results.votes_against, 2);
        assert_eq!(results.total_voters, 3);
        assert_eq!(results.total_credits_spent, 29); // 16 + 9 + 4
    }

    #[test]
    fn test_reset() {
        let mut qv = QuadraticVoting::new();

        // Allocate credits and cast votes
        qv.allocate_credits("voter1".to_string(), 100);
        assert!(qv.cast_vote("voter1".to_string(), 5, true).is_ok());
        assert_eq!(qv.votes_for_a, 5);
        assert_eq!(qv.votes.len(), 1);

        // Reset for new proposal
        qv.reset();
        assert_eq!(qv.votes_for_a, 0);
        assert_eq!(qv.votes_against_b, 0);
        assert!(qv.votes.is_empty());
        // Credits should remain
        assert_eq!(qv.get_remaining_credits(&"voter1".to_string()), 75);
    }
}
