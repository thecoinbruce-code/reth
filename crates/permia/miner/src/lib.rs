//! Permia CPU Miner
//!
//! This crate provides CPU mining functionality for the Permia network
//! using the PermiaHash proof-of-work algorithm.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      PERMIA MINER                               │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │   ┌─────────────────────────────────────────────────────────┐   │
//! │   │  Block Template                                          │   │
//! │   │  • Parent hash, state root, transactions                 │   │
//! │   │  • Difficulty target, timestamp                          │   │
//! │   └─────────────────────────────────────────────────────────┘   │
//! │                              │                                  │
//! │   ┌─────────────────────────────────────────────────────────┐   │
//! │   │  Nonce Search (parallel threads)                         │   │
//! │   │  • Each thread searches different nonce range            │   │
//! │   │  • First valid solution wins                             │   │
//! │   └─────────────────────────────────────────────────────────┘   │
//! │                              │                                  │
//! │   ┌─────────────────────────────────────────────────────────┐   │
//! │   │  Block Submission                                        │   │
//! │   │  • Seal block with nonce + mix_hash                      │   │
//! │   │  • Broadcast to network                                  │   │
//! │   └─────────────────────────────────────────────────────────┘   │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub mod worker;
pub mod template;

pub use worker::{MiningWorker, MiningResult, MiningConfig};
pub use template::BlockTemplate;

use alloy_primitives::U256;
use thiserror::Error;

/// Mining errors
#[derive(Debug, Error)]
pub enum MiningError {
    /// No solution found within nonce range
    #[error("No solution found in nonce range {start}..{end}")]
    NoSolution { start: u64, end: u64 },
    
    /// Mining was cancelled
    #[error("Mining cancelled")]
    Cancelled,
    
    /// Invalid block template
    #[error("Invalid block template: {0}")]
    InvalidTemplate(String),
    
    /// Consensus error
    #[error("Consensus error: {0}")]
    Consensus(#[from] permia_consensus::PermiaConsensusError),
}

/// Mining statistics
#[derive(Debug, Clone, Default)]
pub struct MiningStats {
    /// Total hashes computed
    pub hashes: u64,
    /// Hashes per second
    pub hashrate: f64,
    /// Blocks found
    pub blocks_found: u64,
    /// Current difficulty
    pub difficulty: U256,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_error() {
        let err = MiningError::NoSolution { start: 0, end: 1000 };
        assert!(err.to_string().contains("No solution"));
    }
}
