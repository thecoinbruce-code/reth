//! Permia Network Configuration
//!
//! This module provides the network configuration for Permia nodes,
//! integrating PermiaPoWBlockImport for P2P block validation.

use permia_gossip::PermiaPoWBlockImport;
use reth_chainspec::Hardforks;
use reth_eth_wire::EthNetworkPrimitives;
use reth_ethereum_primitives::EthPrimitives;
use reth_network::{NetworkConfigBuilder, NetworkHandle, NetworkManager, PeersInfo};
use reth_network::primitives::BasicNetworkPrimitives;
use reth_node_api::PrimitivesTy;
use reth_node_builder::{
    components::NetworkBuilder,
    node::{FullNodeTypes, NodeTypes},
    BuilderContext,
};
use reth_provider::BlockReaderIdExt;
use reth_transaction_pool::{PoolPooledTx, PoolTransaction, TransactionPool};
use reth_tracing::tracing::info;
use std::fmt::Debug;

/// Permia Network Builder with PermiaHash PoW block validation
///
/// This network builder sets up the P2P network to use `PermiaPoWBlockImport`
/// for validating incoming block announcements using PermiaHash proof-of-work.
#[derive(Debug, Default, Clone, Copy)]
pub struct PermiaNetworkBuilder;

impl<Node, Pool> NetworkBuilder<Node, Pool> for PermiaNetworkBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec: Hardforks, Primitives = EthPrimitives>>,
    Node::Provider: BlockReaderIdExt + Clone + Debug + Send + Sync + 'static,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = reth_node_api::TxTy<Node::Types>>>
        + Unpin
        + 'static,
{
    type Network =
        NetworkHandle<BasicNetworkPrimitives<PrimitivesTy<Node::Types>, PoolPooledTx<Pool>>>;

    async fn build_network(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> eyre::Result<Self::Network> {
        // Get the network config builder
        let network_config_builder = ctx.network_config_builder()?;
        
        // Set up PermiaPoWBlockImport for P2P block validation
        let provider = ctx.provider().clone();
        let block_import = Box::new(PermiaPoWBlockImport::new(provider));
        let network_config_builder = network_config_builder.block_import(block_import);
        
        // Build the network config
        let network_config = ctx.build_network_config(network_config_builder);
        
        // Start the network
        let network = NetworkManager::builder(network_config).await?;
        let handle = ctx.start_network(network, pool);
        
        info!(
            target: "permia::network",
            enode = %handle.local_node_record(),
            "Permia P2P network initialized with PermiaHash PoW validation"
        );
        
        Ok(handle)
    }
}

/// Configure the network for Permia PoW block gossip (helper function)
pub fn configure_permia_network<Provider>(
    builder: NetworkConfigBuilder<EthNetworkPrimitives>,
    provider: Provider,
) -> NetworkConfigBuilder<EthNetworkPrimitives>
where
    Provider: BlockReaderIdExt + Clone + Debug + Send + Sync + 'static,
{
    let block_import = Box::new(PermiaPoWBlockImport::new(provider));
    builder.block_import(block_import)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exists() {
        // Module compiles
    }
}
