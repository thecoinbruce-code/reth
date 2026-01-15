//! Permia PoW Block Import Implementation
//!
//! This module implements the `BlockImport` trait for Permia's PermiaHash PoW consensus.
//! It validates incoming block announcements and submits valid blocks to the Engine API.

use crate::error::PermiaGossipError;
use alloy_primitives::{B256, U256};
use permia_consensus::PermiaConsensus;
use reth_eth_wire::NewBlock;
use reth_network::import::{
    BlockImport, BlockImportError, BlockImportEvent, BlockImportOutcome, BlockValidation,
    NewBlockEvent,
};
use reth_network::message::NewBlockMessage;
use reth_network_peers::PeerId;
use reth_primitives_traits::Block as BlockTrait;
use reth_provider::BlockReaderIdExt;
use std::{
    collections::VecDeque,
    fmt::Debug,
    sync::Arc,
    task::{Context, Poll},
};
use tracing::{debug, info, trace, warn};

/// Permia PoW Block Import
///
/// Handles incoming block announcements from peers, validates PermiaHash proof-of-work,
/// and submits valid blocks to the local Engine API for import.
#[derive(Debug)]
pub struct PermiaPoWBlockImport<Provider> {
    /// PermiaHash consensus for PoW validation
    consensus: Arc<PermiaConsensus>,
    /// Provider for checking existing blocks
    provider: Provider,
    /// Pending import results
    pending_results: VecDeque<BlockImportEvent<NewBlock>>,
}

impl<Provider> PermiaPoWBlockImport<Provider>
where
    Provider: BlockReaderIdExt + Clone + Debug + 'static,
{
    /// Create a new PermiaPoWBlockImport
    pub fn new(provider: Provider) -> Self {
        let consensus = Arc::new(PermiaConsensus::new());
        Self {
            consensus,
            provider,
            pending_results: VecDeque::new(),
        }
    }

    /// Validate a block's PermiaHash proof-of-work
    fn validate_pow(&self, block: &NewBlock) -> Result<(), PermiaGossipError> {
        let header = block.block.header();
        let difficulty = header.difficulty;
        
        // Check minimum difficulty
        let min_difficulty = self.consensus.min_difficulty();
        if difficulty < min_difficulty {
            return Err(PermiaGossipError::DifficultyTooLow {
                difficulty,
                minimum: min_difficulty,
            });
        }

        // Verify the PermiaHash PoW using the header
        match self.consensus.verify_pow(header) {
            Ok(()) => {
                debug!(
                    target: "permia::gossip",
                    difficulty = %difficulty,
                    nonce = %header.nonce,
                    "PermiaHash PoW validated"
                );
                Ok(())
            }
            Err(e) => {
                warn!(
                    target: "permia::gossip",
                    error = %e,
                    "PermiaHash PoW validation failed"
                );
                Err(PermiaGossipError::InvalidPoW {
                    expected: difficulty,
                    actual: U256::ZERO,
                })
            }
        }
    }

    /// Check if block is already known
    fn is_block_known(&self, hash: B256) -> bool {
        self.provider.block_by_hash(hash).ok().flatten().is_some()
    }

    /// Process a new block announcement
    fn process_new_block(
        &mut self,
        peer_id: PeerId,
        block: NewBlockMessage<NewBlock>,
    ) -> BlockImportOutcome<NewBlock> {
        // Compute block hash from header
        let block_hash = block.block.block.header().hash_slow();
        
        // Check if already known
        if self.is_block_known(block_hash) {
            trace!(
                target: "permia::gossip",
                %block_hash,
                "Block already known, skipping"
            );
            return BlockImportOutcome {
                peer: peer_id,
                result: Err(BlockImportError::Other(Box::new(
                    PermiaGossipError::AlreadyKnown { hash: block_hash },
                ))),
            };
        }

        // Validate PermiaHash PoW
        match self.validate_pow(&block.block) {
            Ok(()) => {
                info!(
                    target: "permia::gossip",
                    %block_hash,
                    %peer_id,
                    "Valid PermiaHash block received from peer"
                );
                
                // Return valid header for relay
                BlockImportOutcome {
                    peer: peer_id,
                    result: Ok(BlockValidation::ValidHeader { block }),
                }
            }
            Err(e) => {
                warn!(
                    target: "permia::gossip",
                    %block_hash,
                    %peer_id,
                    error = %e,
                    "Invalid block received from peer"
                );
                BlockImportOutcome {
                    peer: peer_id,
                    result: Err(BlockImportError::Other(Box::new(e))),
                }
            }
        }
    }
}

impl<Provider> BlockImport<NewBlock> for PermiaPoWBlockImport<Provider>
where
    Provider: BlockReaderIdExt + Clone + Debug + Send + Sync + 'static,
{
    fn on_new_block(&mut self, peer_id: PeerId, incoming_block: NewBlockEvent<NewBlock>) {
        trace!(
            target: "permia::gossip",
            %peer_id,
            "Received new block event"
        );
        
        match incoming_block {
            NewBlockEvent::Block(block) => {
                let outcome = self.process_new_block(peer_id, block);
                self.pending_results.push_back(BlockImportEvent::Outcome(outcome));
            }
            NewBlockEvent::Hashes(hashes) => {
                // For hash announcements, we need to request the full block
                // This is handled by the network layer
                trace!(
                    target: "permia::gossip",
                    num_hashes = hashes.0.len(),
                    "Received block hash announcement"
                );
            }
        }
    }

    fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<BlockImportEvent<NewBlock>> {
        // Return any pending results
        if let Some(event) = self.pending_results.pop_front() {
            return Poll::Ready(event);
        }
        
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permia_gossip_error_display() {
        let err = PermiaGossipError::InvalidPoW {
            expected: U256::from(1000u64),
            actual: U256::from(500u64),
        };
        assert!(err.to_string().contains("Invalid PermiaHash PoW"));
    }
}
