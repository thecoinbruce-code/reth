//! Permia Node Implementation
//!
//! This crate provides the Permia-specific node configuration,
//! integrating PermiaHash PoW consensus with the Reth node framework.

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub mod consensus;

pub use consensus::PermiaConsensusBuilder;
pub use permia_chainspec::{
    PermiaChainSpec, PERMIA_DEVNET, PERMIA_MAINNET, PERMIA_TESTNET,
    PERMIA_MAINNET_CHAIN_ID, PERMIA_TESTNET_CHAIN_ID, PERMIA_DEVNET_CHAIN_ID,
};
pub use permia_consensus::{PermiaConsensus, PermiaConsensusError, BLOCK_TIME_MS};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exports() {
        let _ = PERMIA_MAINNET_CHAIN_ID;
        let _ = BLOCK_TIME_MS;
    }
}
