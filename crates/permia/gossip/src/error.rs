//! Permia gossip error types

use alloy_primitives::{B256, U256};
use thiserror::Error;

/// Errors that can occur during Permia block gossip
#[derive(Debug, Error)]
pub enum PermiaGossipError {
    /// Invalid PermiaHash proof-of-work
    #[error("Invalid PermiaHash PoW: expected difficulty {expected}, got {actual}")]
    InvalidPoW {
        /// Expected minimum difficulty
        expected: U256,
        /// Actual computed difficulty
        actual: U256,
    },

    /// Block hash mismatch
    #[error("Block hash mismatch: expected {expected}, computed {computed}")]
    HashMismatch {
        /// Expected hash from announcement
        expected: B256,
        /// Computed hash from block data
        computed: B256,
    },

    /// Parent block not found
    #[error("Parent block not found: {parent_hash}")]
    ParentNotFound {
        /// Hash of missing parent
        parent_hash: B256,
    },

    /// Difficulty too low
    #[error("Difficulty too low: {difficulty} < minimum {minimum}")]
    DifficultyTooLow {
        /// Block's difficulty
        difficulty: U256,
        /// Minimum required difficulty
        minimum: U256,
    },

    /// Block already known
    #[error("Block already known: {hash}")]
    AlreadyKnown {
        /// Hash of known block
        hash: B256,
    },

    /// Engine API error
    #[error("Engine API error: {0}")]
    EngineApi(String),

    /// Provider error
    #[error("Provider error: {0}")]
    Provider(String),

    /// Consensus error
    #[error("Consensus error: {0}")]
    Consensus(#[from] reth_consensus::ConsensusError),
}
