//! Permia Consensus Builder
//!
//! Provides integration between Permia consensus and Reth's node builder.

use permia_consensus::{PermiaConsensus, PermiaPoWConsensus};
use reth_chainspec::ChainSpec;
use std::sync::Arc;

/// Builder for Permia consensus.
///
/// This builder creates a PermiaPoWConsensus instance that uses PermiaHash PoW
/// for block validation.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct PermiaConsensusBuilder;

impl PermiaConsensusBuilder {
    /// Create a new Permia consensus builder
    pub fn new() -> Self {
        Self
    }
    
    /// Build the standalone Permia consensus instance
    pub fn build_standalone(self) -> Arc<PermiaConsensus> {
        Arc::new(PermiaConsensus::new())
    }
    
    /// Build the Permia PoW consensus with chain spec
    pub fn build_with_chain_spec(self, chain_spec: Arc<ChainSpec>) -> Arc<PermiaPoWConsensus> {
        Arc::new(PermiaPoWConsensus::new(chain_spec))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reth_chainspec::PERMIA_DEV;
    
    #[test]
    fn test_build_consensus() {
        let builder = PermiaConsensusBuilder::new();
        let consensus = builder.build_standalone();
        assert!(consensus.min_difficulty() > alloy_primitives::U256::ZERO);
    }
    
    #[test]
    fn test_build_with_chain_spec() {
        let builder = PermiaConsensusBuilder::new();
        let consensus = builder.build_with_chain_spec(PERMIA_DEV.clone());
        assert_eq!(consensus.chain_spec().chain.id(), 42071);
    }
}
