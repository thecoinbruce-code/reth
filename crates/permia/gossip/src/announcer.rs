//! Permia Block Announcer
//!
//! This module provides the block announcement service that broadcasts
//! newly mined blocks to peers via the P2P network.

use alloy_primitives::U128;
use reth_chain_state::{CanonStateNotification, CanonStateSubscriptions};
use reth_eth_wire::{NetworkPrimitives, NewBlock};
use reth_ethereum_primitives::EthPrimitives;
use reth_network::NetworkHandle;
use reth_primitives_traits::RecoveredBlock;
use std::future::Future;
use tokio_stream::StreamExt;
use tracing::{debug, info};

/// Permia Block Announcer
///
/// Listens for new canonical blocks and announces them to peers.
/// This enables P2P block propagation for PoW consensus.
pub struct PermiaBlockAnnouncer<N: NetworkPrimitives> {
    /// Network handle for announcing blocks
    network: NetworkHandle<N>,
}

impl<N> PermiaBlockAnnouncer<N>
where
    N: reth_eth_wire::NetworkPrimitives<NewBlockPayload = NewBlock>,
{
    /// Create a new block announcer
    pub fn new(network: NetworkHandle<N>) -> Self {
        Self { network }
    }

    /// Run the block announcer, listening for new blocks and announcing them
    pub async fn run<P>(self, provider: P)
    where
        P: CanonStateSubscriptions<Primitives = EthPrimitives>,
    {
        info!(target: "permia::announcer", "Block announcer started");
        
        let mut stream = provider.canonical_state_stream();
        
        while let Some(notification) = stream.next().await {
            match notification {
                CanonStateNotification::Commit { new } => {
                    // Announce all new blocks - blocks() returns (number, block) tuples
                    for (_number, block) in new.blocks() {
                        self.announce_block(block);
                    }
                }
                CanonStateNotification::Reorg { new, old } => {
                    debug!(
                        target: "permia::announcer",
                        reverted_blocks = old.len(),
                        new_blocks = new.len(),
                        "Chain reorg detected"
                    );
                    // Announce new blocks after reorg
                    for (_number, block) in new.blocks() {
                        self.announce_block(block);
                    }
                }
            }
        }
        
        info!(target: "permia::announcer", "Block announcer stopped");
    }

    /// Announce a single block to peers
    fn announce_block(&self, block: &RecoveredBlock<<EthPrimitives as reth_primitives_traits::NodePrimitives>::Block>) {
        let header = block.header();
        let hash = block.hash();
        let number = header.number;
        let difficulty = header.difficulty;
        
        // Create NewBlock message with total difficulty
        // For PoW, TD is cumulative difficulty up to this block
        let new_block = NewBlock {
            block: block.clone().into_block(),
            td: U128::from(difficulty),
        };
        
        info!(
            target: "permia::announcer",
            block_number = %number,
            block_hash = %hash,
            difficulty = %difficulty,
            "Announcing block to peers"
        );
        
        self.network.announce_block(new_block, hash);
    }
}

/// Spawn the block announcer as a background task
pub fn spawn_block_announcer<N, P>(
    network: NetworkHandle<N>,
    provider: P,
) -> impl Future<Output = ()>
where
    N: reth_eth_wire::NetworkPrimitives<NewBlockPayload = NewBlock> + 'static,
    P: CanonStateSubscriptions<Primitives = EthPrimitives> + 'static,
{
    let announcer = PermiaBlockAnnouncer::new(network);
    async move {
        announcer.run(provider).await;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exists() {
        // Module compiles
    }
}
