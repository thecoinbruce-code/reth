//! Permia Network Configuration
//!
//! This module provides the network configuration for Permia nodes,
//! integrating PermiaPoWBlockImport for P2P block validation.

use permia_gossip::PermiaPoWBlockImport;
use reth_eth_wire::EthNetworkPrimitives;
use reth_network::NetworkConfigBuilder;
use reth_provider::BlockReaderIdExt;
use std::fmt::Debug;

/// Configure the network for Permia PoW block gossip
///
/// This sets up the network to use `PermiaPoWBlockImport` for validating
/// incoming block announcements using PermiaHash proof-of-work.
///
/// # Example
///
/// ```ignore
/// use permia_node::network::configure_permia_network;
///
/// let network_config = NetworkConfigBuilder::new(secret_key)
///     .apply(|builder| configure_permia_network(builder, provider));
/// ```
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
