//! Permia Consensus Builder
//!
//! Provides integration between Permia consensus and Reth's node builder.

use permia_consensus::{PermiaConsensus, PermiaPoWConsensus};
use reth_chainspec::ChainSpec;
use reth_node_builder::{components::ConsensusBuilder, BuilderContext, FullNodeTypes};
use reth_node_api::NodeTypes;
use reth_ethereum_primitives::EthPrimitives;
use std::sync::Arc;

/// Builder for Permia consensus.
///
/// This builder creates a PermiaPoWConsensus instance that uses PermiaHash PoW
/// for block validation, integrated with Reth's node builder.
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
}

impl<Node> ConsensusBuilder<Node> for PermiaConsensusBuilder
where
    Node: FullNodeTypes<
        Types: NodeTypes<ChainSpec = ChainSpec, Primitives = EthPrimitives>,
    >,
{
    type Consensus = Arc<PermiaPoWConsensus>;

    async fn build_consensus(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::Consensus> {
        Ok(Arc::new(PermiaPoWConsensus::new(ctx.chain_spec())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_build_consensus() {
        let builder = PermiaConsensusBuilder::new();
        let consensus = builder.build_standalone();
        assert!(consensus.min_difficulty() > alloy_primitives::U256::ZERO);
    }
}
