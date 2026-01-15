//! Permia chain specifications
//!
//! Chain IDs:
//! - Mainnet: 42069
//! - Testnet: 42070  
//! - Devnet: 42071

use crate::{make_genesis_header, BaseFeeParams, BaseFeeParamsKind, ChainSpec};
use alloc::sync::Arc;
use alloy_chains::Chain;
use reth_ethereum_forks::DEV_HARDFORKS;
use reth_primitives_traits::{sync::LazyLock, SealedHeader};

/// Permia mainnet chain ID
pub const PERMIA_MAINNET_CHAIN_ID: u64 = 42069;

/// Permia testnet chain ID
pub const PERMIA_TESTNET_CHAIN_ID: u64 = 42070;

/// Permia devnet chain ID
pub const PERMIA_DEVNET_CHAIN_ID: u64 = 42071;

/// Target block time in milliseconds
pub const PERMIA_BLOCK_TIME_MS: u64 = 400;

/// Permia devnet specification
pub static PERMIA_DEV: LazyLock<Arc<ChainSpec>> = LazyLock::new(|| {
    let genesis = serde_json::from_str(include_str!("../res/genesis/permia-dev.json"))
        .expect("Can't deserialize Permia dev genesis json");
    let hardforks = DEV_HARDFORKS.clone();
    ChainSpec {
        chain: Chain::from_id(PERMIA_DEVNET_CHAIN_ID),
        genesis_header: SealedHeader::seal_slow(make_genesis_header(&genesis, &hardforks)),
        genesis,
        paris_block_and_final_difficulty: None, // Permia uses PoW, not PoS
        hardforks,
        base_fee_params: BaseFeeParamsKind::Constant(BaseFeeParams::ethereum()),
        deposit_contract: None,
        ..Default::default()
    }
    .into()
});

/// Permia testnet specification
pub static PERMIA_TESTNET: LazyLock<Arc<ChainSpec>> = LazyLock::new(|| {
    let genesis = serde_json::from_str(include_str!("../res/genesis/permia-testnet.json"))
        .expect("Can't deserialize Permia testnet genesis json");
    let hardforks = DEV_HARDFORKS.clone();
    ChainSpec {
        chain: Chain::from_id(PERMIA_TESTNET_CHAIN_ID),
        genesis_header: SealedHeader::seal_slow(make_genesis_header(&genesis, &hardforks)),
        genesis,
        paris_block_and_final_difficulty: None,
        hardforks,
        base_fee_params: BaseFeeParamsKind::Constant(BaseFeeParams::ethereum()),
        deposit_contract: None,
        ..Default::default()
    }
    .into()
});

/// Permia mainnet specification
pub static PERMIA_MAINNET: LazyLock<Arc<ChainSpec>> = LazyLock::new(|| {
    let genesis = serde_json::from_str(include_str!("../res/genesis/permia-mainnet.json"))
        .expect("Can't deserialize Permia mainnet genesis json");
    let hardforks = DEV_HARDFORKS.clone();
    ChainSpec {
        chain: Chain::from_id(PERMIA_MAINNET_CHAIN_ID),
        genesis_header: SealedHeader::seal_slow(make_genesis_header(&genesis, &hardforks)),
        genesis,
        paris_block_and_final_difficulty: None,
        hardforks,
        base_fee_params: BaseFeeParamsKind::Constant(BaseFeeParams::ethereum()),
        deposit_contract: None,
        ..Default::default()
    }
    .into()
});

/// Get Permia chain spec by chain ID
pub fn permia_chain_spec(chain_id: u64) -> Option<Arc<ChainSpec>> {
    match chain_id {
        PERMIA_MAINNET_CHAIN_ID => Some(PERMIA_MAINNET.clone()),
        PERMIA_TESTNET_CHAIN_ID => Some(PERMIA_TESTNET.clone()),
        PERMIA_DEVNET_CHAIN_ID => Some(PERMIA_DEV.clone()),
        _ => None,
    }
}

/// Get Permia chain spec by name
pub fn permia_chain_spec_by_name(name: &str) -> Option<Arc<ChainSpec>> {
    match name.to_lowercase().as_str() {
        "permia" | "permia-mainnet" | "mainnet" => Some(PERMIA_MAINNET.clone()),
        "permia-testnet" | "testnet" => Some(PERMIA_TESTNET.clone()),
        "permia-dev" | "dev" => Some(PERMIA_DEV.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permia_dev_chain_id() {
        assert_eq!(PERMIA_DEV.chain.id(), PERMIA_DEVNET_CHAIN_ID);
    }

    #[test]
    fn test_permia_testnet_chain_id() {
        assert_eq!(PERMIA_TESTNET.chain.id(), PERMIA_TESTNET_CHAIN_ID);
    }

    #[test]
    fn test_permia_mainnet_chain_id() {
        assert_eq!(PERMIA_MAINNET.chain.id(), PERMIA_MAINNET_CHAIN_ID);
    }

    #[test]
    fn test_chain_spec_lookup() {
        assert!(permia_chain_spec(42069).is_some());
        assert!(permia_chain_spec(42070).is_some());
        assert!(permia_chain_spec(42071).is_some());
        assert!(permia_chain_spec(1).is_none());
    }
}
