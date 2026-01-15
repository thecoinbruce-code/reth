//! Permia Chain Specifications
//!
//! Defines the chain parameters for Permia networks:
//! - Mainnet (chain ID: 42069)
//! - Testnet (chain ID: 42070)
//! - Devnet (chain ID: 42071)

use alloy_genesis::{ChainConfig, Genesis};
use alloy_primitives::{address, b256, Address, B256, U256};
use once_cell::sync::Lazy;
use std::collections::BTreeMap;

/// Permia mainnet chain ID
pub const PERMIA_MAINNET_CHAIN_ID: u64 = 42069;

/// Permia testnet chain ID  
pub const PERMIA_TESTNET_CHAIN_ID: u64 = 42070;

/// Permia devnet chain ID
pub const PERMIA_DEVNET_CHAIN_ID: u64 = 42071;

/// Target block time in milliseconds
pub const BLOCK_TIME_MS: u64 = 400;

/// Maximum block gas limit
pub const MAX_BLOCK_GAS: u64 = 60_000_000;

/// Treasury address
pub const TREASURY_ADDRESS: Address = address!("0000000000000000000000000000000000000001");

/// PermiaSwap POL address
pub const PERMIASWAP_POL_ADDRESS: Address = address!("0000000000000000000000000000000000000002");

/// Permia mainnet genesis hash
pub static PERMIA_MAINNET_GENESIS_HASH: Lazy<B256> = Lazy::new(|| {
    b256!("0000000000000000000000000000000000000000000000000000000000000000")
});

/// Permia mainnet chain spec
pub static PERMIA_MAINNET: Lazy<PermiaChainSpec> = Lazy::new(|| {
    PermiaChainSpec {
        chain_id: PERMIA_MAINNET_CHAIN_ID,
        name: "permia-mainnet".to_string(),
        genesis: permia_mainnet_genesis(),
        block_time_ms: BLOCK_TIME_MS,
        max_block_gas: MAX_BLOCK_GAS,
    }
});

/// Permia testnet chain spec
pub static PERMIA_TESTNET: Lazy<PermiaChainSpec> = Lazy::new(|| {
    PermiaChainSpec {
        chain_id: PERMIA_TESTNET_CHAIN_ID,
        name: "permia-testnet".to_string(),
        genesis: permia_testnet_genesis(),
        block_time_ms: BLOCK_TIME_MS,
        max_block_gas: MAX_BLOCK_GAS,
    }
});

/// Permia devnet chain spec (for local development)
pub static PERMIA_DEVNET: Lazy<PermiaChainSpec> = Lazy::new(|| {
    PermiaChainSpec {
        chain_id: PERMIA_DEVNET_CHAIN_ID,
        name: "permia-dev".to_string(),
        genesis: permia_devnet_genesis(),
        block_time_ms: BLOCK_TIME_MS,
        max_block_gas: MAX_BLOCK_GAS,
    }
});

/// Permia chain specification
#[derive(Debug, Clone)]
pub struct PermiaChainSpec {
    /// Chain ID
    pub chain_id: u64,
    /// Chain name
    pub name: String,
    /// Genesis configuration
    pub genesis: Genesis,
    /// Target block time in milliseconds
    pub block_time_ms: u64,
    /// Maximum block gas
    pub max_block_gas: u64,
}

impl PermiaChainSpec {
    /// Get chain spec by name
    pub fn from_name(name: &str) -> Option<&'static PermiaChainSpec> {
        match name {
            "permia-mainnet" | "mainnet" => Some(&PERMIA_MAINNET),
            "permia-testnet" | "testnet" => Some(&PERMIA_TESTNET),
            "permia-dev" | "dev" => Some(&PERMIA_DEVNET),
            _ => None,
        }
    }
    
    /// Get chain spec by chain ID
    pub fn from_chain_id(chain_id: u64) -> Option<&'static PermiaChainSpec> {
        match chain_id {
            PERMIA_MAINNET_CHAIN_ID => Some(&PERMIA_MAINNET),
            PERMIA_TESTNET_CHAIN_ID => Some(&PERMIA_TESTNET),
            PERMIA_DEVNET_CHAIN_ID => Some(&PERMIA_DEVNET),
            _ => None,
        }
    }
}

/// Create mainnet genesis
fn permia_mainnet_genesis() -> Genesis {
    let mut alloc = BTreeMap::new();
    
    // Treasury allocation (10% of supply = 100M MIA)
    alloc.insert(
        TREASURY_ADDRESS,
        alloy_genesis::GenesisAccount {
            balance: U256::from(100_000_000u64) * U256::from(10u64).pow(U256::from(18u64)),
            ..Default::default()
        },
    );
    
    // PermiaSwap POL allocation (5% = 50M MIA)
    alloc.insert(
        PERMIASWAP_POL_ADDRESS,
        alloy_genesis::GenesisAccount {
            balance: U256::from(50_000_000u64) * U256::from(10u64).pow(U256::from(18u64)),
            ..Default::default()
        },
    );
    
    Genesis {
        config: ChainConfig {
            chain_id: PERMIA_MAINNET_CHAIN_ID,
            homestead_block: Some(0),
            eip150_block: Some(0),
            eip155_block: Some(0),
            eip158_block: Some(0),
            byzantium_block: Some(0),
            constantinople_block: Some(0),
            petersburg_block: Some(0),
            istanbul_block: Some(0),
            berlin_block: Some(0),
            london_block: Some(0),
            ..Default::default()
        },
        nonce: 0x42069,
        timestamp: 0,
        gas_limit: MAX_BLOCK_GAS,
        difficulty: U256::from(1u64 << 20),
        alloc,
        ..Default::default()
    }
}

/// Create testnet genesis
fn permia_testnet_genesis() -> Genesis {
    let mut alloc = BTreeMap::new();
    
    // Faucet allocation for testnet
    let faucet = address!("0000000000000000000000000000000000001000");
    alloc.insert(
        faucet,
        alloy_genesis::GenesisAccount {
            balance: U256::from(1_000_000_000u64) * U256::from(10u64).pow(U256::from(18u64)),
            ..Default::default()
        },
    );
    
    Genesis {
        config: ChainConfig {
            chain_id: PERMIA_TESTNET_CHAIN_ID,
            homestead_block: Some(0),
            eip150_block: Some(0),
            eip155_block: Some(0),
            eip158_block: Some(0),
            byzantium_block: Some(0),
            constantinople_block: Some(0),
            petersburg_block: Some(0),
            istanbul_block: Some(0),
            berlin_block: Some(0),
            london_block: Some(0),
            ..Default::default()
        },
        nonce: 0x42070,
        timestamp: 0,
        gas_limit: MAX_BLOCK_GAS,
        difficulty: U256::from(1u64 << 16), // Lower difficulty for testnet
        alloc,
        ..Default::default()
    }
}

/// Create devnet genesis (for local development)
fn permia_devnet_genesis() -> Genesis {
    let mut alloc = BTreeMap::new();
    
    // Dev accounts with plenty of funds
    for i in 1..=10 {
        let addr = Address::from_word(B256::from(U256::from(i)));
        alloc.insert(
            addr,
            alloy_genesis::GenesisAccount {
                balance: U256::from(1_000_000u64) * U256::from(10u64).pow(U256::from(18u64)),
                ..Default::default()
            },
        );
    }
    
    Genesis {
        config: ChainConfig {
            chain_id: PERMIA_DEVNET_CHAIN_ID,
            homestead_block: Some(0),
            eip150_block: Some(0),
            eip155_block: Some(0),
            eip158_block: Some(0),
            byzantium_block: Some(0),
            constantinople_block: Some(0),
            petersburg_block: Some(0),
            istanbul_block: Some(0),
            berlin_block: Some(0),
            london_block: Some(0),
            ..Default::default()
        },
        nonce: 0x42071,
        timestamp: 0,
        gas_limit: MAX_BLOCK_GAS,
        difficulty: U256::from(1u64 << 10), // Very low difficulty for dev
        alloc,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mainnet_chain_spec() {
        assert_eq!(PERMIA_MAINNET.chain_id, PERMIA_MAINNET_CHAIN_ID);
        assert_eq!(PERMIA_MAINNET.name, "permia-mainnet");
    }
    
    #[test]
    fn test_chain_spec_lookup() {
        assert!(PermiaChainSpec::from_name("mainnet").is_some());
        assert!(PermiaChainSpec::from_chain_id(42069).is_some());
    }
}
