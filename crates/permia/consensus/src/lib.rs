//! Permia Consensus Implementation
//!
//! This crate provides the consensus mechanism for Permia:
//! - PermiaHash: Memory-hard ASIC-resistant Proof-of-Work
//! - BFT Finality: Fast finality through validator voting

pub mod pow;
pub mod difficulty;
pub mod reth;

pub use reth::PermiaPoWConsensus;

use alloy_consensus::Header;
use alloy_primitives::U256;
use std::sync::Arc;

/// Permia chain ID
pub const PERMIA_CHAIN_ID: u64 = 42069;

/// Target block time in milliseconds
pub const BLOCK_TIME_MS: u64 = 400;

/// Permia consensus implementation
#[derive(Debug, Clone)]
pub struct PermiaConsensus {
    /// Difficulty calculator
    difficulty_calc: Arc<difficulty::DifficultyCalculator>,
}

impl PermiaConsensus {
    /// Create new Permia consensus instance
    pub fn new() -> Self {
        Self {
            difficulty_calc: Arc::new(difficulty::DifficultyCalculator::new()),
        }
    }
    
    /// Verify PermiaHash proof of work
    pub fn verify_pow(&self, header: &Header) -> Result<(), PermiaConsensusError> {
        pow::verify_pow(header).map_err(|_| PermiaConsensusError::InvalidProofOfWork)
    }
    
    /// Calculate next block difficulty
    pub fn calculate_difficulty(&self, parent: &Header, timestamp: u64) -> U256 {
        self.difficulty_calc.calculate(parent, timestamp)
    }
    
    /// Get minimum difficulty
    pub fn min_difficulty(&self) -> U256 {
        self.difficulty_calc.min_difficulty()
    }
}

impl Default for PermiaConsensus {
    fn default() -> Self {
        Self::new()
    }
}

/// Permia-specific consensus errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum PermiaConsensusError {
    #[error("invalid proof of work")]
    InvalidProofOfWork,
    #[error("invalid difficulty")]
    InvalidDifficulty,
    #[error("timestamp too old")]
    TimestampTooOld,
    #[error("block number mismatch")]
    BlockNumberMismatch,
    #[error("extra data too large")]
    ExtraDataTooLarge,
    #[error("gas used exceeds limit")]
    GasUsedExceedsLimit,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_consensus_creation() {
        let consensus = PermiaConsensus::new();
        assert!(consensus.min_difficulty() > U256::ZERO);
    }
}
