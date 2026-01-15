//! Permia Genesis Tool
//!
//! This crate provides tools for creating and initializing the Permia blockchain
//! genesis state.
//!
//! # Genesis Configuration (from PROTOCOL_SPEC_v4.md)
//!
//! ```text
//! Genesis Block:
//! ├── Chain ID: 42069 (mainnet), 42070 (testnet), 42071 (devnet)
//! ├── Block Time Target: 400ms
//! ├── Initial Difficulty: 2^20 = 1,048,576
//! ├── Block Reward: 10 MIA
//! ├── Initial Supply: 0 (all mined)
//! └── Pre-allocated Accounts:
//!     ├── Foundation: 10% of year 1 mining (vested)
//!     ├── Team: 5% (4-year vest)
//!     └── Community: 5% (ecosystem grants)
//! ```

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub mod config;
pub mod builder;

pub use config::{GenesisConfig, NetworkType, Allocation};
pub use builder::GenesisBuilder;

use alloy_primitives::{Address, B256, U256};
use thiserror::Error;

/// Genesis creation errors
#[derive(Debug, Error)]
pub enum GenesisError {
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// JSON serialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Chain constants
pub mod constants {
    use super::U256;

    /// Mainnet chain ID
    pub const MAINNET_CHAIN_ID: u64 = 42069;
    
    /// Testnet chain ID
    pub const TESTNET_CHAIN_ID: u64 = 42070;
    
    /// Devnet chain ID
    pub const DEVNET_CHAIN_ID: u64 = 42071;
    
    /// Target block time in milliseconds
    pub const TARGET_BLOCK_TIME_MS: u64 = 400;
    
    /// Initial mining difficulty (2^20)
    pub const INITIAL_DIFFICULTY: u64 = 1_048_576;
    
    /// Base block reward in wei (10 MIA = 10 * 10^18)
    pub const BASE_BLOCK_REWARD: u128 = 10_000_000_000_000_000_000;
    
    /// Blocks per day at 400ms
    pub const BLOCKS_PER_DAY: u64 = 216_000; // 86400 * 1000 / 400
    
    /// Blocks per year
    pub const BLOCKS_PER_YEAR: u64 = 78_840_000; // 365 * BLOCKS_PER_DAY
    
    /// Foundation allocation (10% of year 1 mining)
    pub fn foundation_allocation() -> U256 {
        // 10 MIA * blocks_per_year * 0.10
        let year1_mining = U256::from(BASE_BLOCK_REWARD) * U256::from(BLOCKS_PER_YEAR);
        year1_mining / U256::from(10)
    }
    
    /// Team allocation (5% of year 1 mining)
    pub fn team_allocation() -> U256 {
        let year1_mining = U256::from(BASE_BLOCK_REWARD) * U256::from(BLOCKS_PER_YEAR);
        year1_mining / U256::from(20)
    }
    
    /// Community allocation (5% of year 1 mining)
    pub fn community_allocation() -> U256 {
        let year1_mining = U256::from(BASE_BLOCK_REWARD) * U256::from(BLOCKS_PER_YEAR);
        year1_mining / U256::from(20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use constants::*;

    #[test]
    fn test_chain_ids() {
        assert_eq!(MAINNET_CHAIN_ID, 42069);
        assert_eq!(TESTNET_CHAIN_ID, 42070);
        assert_eq!(DEVNET_CHAIN_ID, 42071);
    }

    #[test]
    fn test_allocations() {
        let foundation = foundation_allocation();
        let team = team_allocation();
        let community = community_allocation();
        
        // Foundation should be 2x team/community
        assert_eq!(foundation, team * U256::from(2));
        assert_eq!(team, community);
        
        // Total should be 20% of year 1 mining
        let total = foundation + team + community;
        let year1 = U256::from(BASE_BLOCK_REWARD) * U256::from(BLOCKS_PER_YEAR);
        assert_eq!(total, year1 / U256::from(5));
    }
}
