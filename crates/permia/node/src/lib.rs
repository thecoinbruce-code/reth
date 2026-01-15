//! Permia Node Implementation
//!
//! This crate provides the Permia-specific node configuration,
//! integrating PermiaHash PoW consensus with the Reth node framework.
//!
//! # Usage
//!
//! ```ignore
//! use permia_node::PermiaNode;
//! use reth_node_builder::NodeBuilder;
//!
//! let handle = NodeBuilder::new(config)
//!     .node(PermiaNode::default())
//!     .launch()
//!     .await?;
//! ```

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub mod consensus;
pub mod network;
pub mod node;

pub use consensus::PermiaConsensusBuilder;
pub use network::{configure_permia_network, PermiaNetworkBuilder};
pub use node::PermiaNode;
pub use permia_consensus::{PermiaConsensus, PermiaConsensusError, PermiaPoWConsensus, BLOCK_TIME_MS};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exports() {
        let _ = BLOCK_TIME_MS;
        let _ = PermiaNode::default();
    }
}
