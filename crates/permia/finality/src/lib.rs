//! Permia BFT Finality Layer
//!
//! This crate implements the BFT (Byzantine Fault Tolerant) finality mechanism
//! for the Permia network, providing fast block confirmation.
//!
//! # Architecture (from PROTOCOL_SPEC_v4.md)
//!
//! ```text
//! Block Production Flow:
//!
//! 1. Miners compete via PoW to propose block
//! 2. Winner broadcasts block to network
//! 3. Validators vote on block validity
//! 4. Block is FINAL when:
//!    - 67+ validators (2/3+) have signed, OR
//!    - 3 subsequent blocks built on it
//! ```
//!
//! # Validator Set
//!
//! - Top 100 miners by stake + service score
//! - Updated every epoch (3,600 blocks = 24 minutes at 400ms)
//! - Minimum stake: 10,000 MIA

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub mod validator;
pub mod vote;
pub mod finality;

pub use validator::{Validator, ValidatorSet, ValidatorSetUpdate};
pub use vote::{Vote, VoteMessage, VoteAggregator};
pub use finality::{FinalityTracker, FinalityStatus};

use alloy_primitives::{Address, B256, U256};
use thiserror::Error;

/// Finality configuration constants
pub mod config {
    /// Number of validators in the active set
    pub const VALIDATOR_SET_SIZE: usize = 100;
    
    /// Blocks per epoch for validator set updates
    pub const EPOCH_LENGTH: u64 = 3600; // ~24 minutes at 400ms blocks
    
    /// Minimum stake required to be a validator (in wei)
    /// 10,000 MIA = 10_000 * 10^18 wei
    pub const MIN_STAKE: u128 = 10_000_000_000_000_000_000_000; // 10,000 MIA
    
    /// Threshold for BFT finality (2/3 + 1)
    pub const FINALITY_THRESHOLD: usize = 67;
    
    /// Blocks required for implicit finality
    pub const IMPLICIT_FINALITY_DEPTH: u64 = 3;
}

/// Finality errors
#[derive(Debug, Error)]
pub enum FinalityError {
    /// Invalid signature on vote
    #[error("Invalid vote signature")]
    InvalidSignature,
    
    /// Validator not in active set
    #[error("Validator {0} not in active set")]
    NotValidator(Address),
    
    /// Duplicate vote from validator
    #[error("Duplicate vote from {0} for block {1}")]
    DuplicateVote(Address, B256),
    
    /// Block not found
    #[error("Block {0} not found")]
    BlockNotFound(B256),
    
    /// Invalid block for voting
    #[error("Cannot vote on block: {0}")]
    InvalidBlock(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_values() {
        assert_eq!(config::VALIDATOR_SET_SIZE, 100);
        assert_eq!(config::FINALITY_THRESHOLD, 67);
        assert_eq!(config::IMPLICIT_FINALITY_DEPTH, 3);
    }
}
