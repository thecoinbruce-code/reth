//! Permia Node type configuration
//!
//! This module defines the PermiaNode type that integrates PermiaHash PoW
//! consensus with the Reth node builder framework.
//!
//! # Architecture
//!
//! PermiaNode reuses most of Ethereum's infrastructure:
//! - EVM execution (full EVM compatibility)
//! - Transaction pool
//! - P2P networking
//! - Payload building
//!
//! But replaces the consensus with PermiaHash PoW.

use crate::consensus::PermiaConsensusBuilder;
use reth_chainspec::ChainSpec;
use reth_ethereum_primitives::EthPrimitives;
use reth_node_api::NodeTypes;
use reth_node_ethereum::EthEngineTypes;
use reth_provider::EthStorage;

/// Permia node type configuration.
///
/// This defines the type parameters for a Permia node, using:
/// - **EthPrimitives**: Standard Ethereum block/transaction types
/// - **ChainSpec**: Permia chain specification
/// - **EthStorage**: Standard Ethereum storage
/// - **EthEngineTypes**: Standard Ethereum engine types
///
/// The actual consensus (PermiaHash PoW) is wired via PermiaConsensusBuilder
/// when building the node components.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct PermiaNode;

impl PermiaNode {
    /// Get the Permia consensus builder
    pub fn consensus_builder() -> PermiaConsensusBuilder {
        PermiaConsensusBuilder::default()
    }
}

impl NodeTypes for PermiaNode {
    type Primitives = EthPrimitives;
    type ChainSpec = ChainSpec;
    type Storage = EthStorage;
    type Payload = EthEngineTypes;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permia_node_types() {
        let _node = PermiaNode::default();
        let _consensus = PermiaNode::consensus_builder();
    }
}
