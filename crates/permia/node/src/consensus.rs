//! Permia Consensus Builder
//!
//! Provides integration between Permia consensus and Reth's node builder.

use permia_consensus::PermiaConsensus;
use std::sync::Arc;

/// Builder for Permia consensus.
///
/// This builder creates a PermiaConsensus instance that uses PermiaHash PoW
/// for block validation.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct PermiaConsensusBuilder;

impl PermiaConsensusBuilder {
    /// Create a new Permia consensus builder
    pub fn new() -> Self {
        Self
    }
    
    /// Build the Permia consensus instance
    pub fn build(self) -> Arc<PermiaConsensus> {
        Arc::new(PermiaConsensus::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_build_consensus() {
        let builder = PermiaConsensusBuilder::new();
        let consensus = builder.build();
        assert!(consensus.min_difficulty() > alloy_primitives::U256::ZERO);
    }
}
