//! Permia Node Implementation
//!
//! This crate provides the Permia-specific node configuration,
//! integrating PermiaHash PoW consensus with the Reth node framework.

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub mod consensus;

pub use consensus::PermiaConsensusBuilder;
pub use permia_consensus::{PermiaConsensus, PermiaConsensusError, PermiaPoWConsensus, BLOCK_TIME_MS};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exports() {
        let _ = BLOCK_TIME_MS;
    }
}
