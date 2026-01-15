//! Block finality tracking

use alloy_primitives::B256;
use std::collections::HashMap;

use crate::{config, ValidatorSet, VoteAggregator};

/// Status of a block's finality
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalityStatus {
    /// Block is not yet final
    Pending {
        /// Number of validator votes received
        votes: usize,
        /// Votes needed for BFT finality
        threshold: usize,
    },
    /// Block is final via BFT (2/3+ votes)
    FinalizedBft {
        /// Number of votes that finalized the block
        votes: usize,
    },
    /// Block is final via depth (3+ confirmations)
    FinalizedDepth {
        /// Current depth (confirmations)
        depth: u64,
    },
}

impl FinalityStatus {
    /// Check if the block is final (by any method)
    pub fn is_final(&self) -> bool {
        matches!(self, FinalityStatus::FinalizedBft { .. } | FinalityStatus::FinalizedDepth { .. })
    }
}

/// Tracks finality for blocks
#[derive(Debug)]
pub struct FinalityTracker {
    /// Vote aggregator
    votes: VoteAggregator,
    /// Block depths (hash -> depth from chain head)
    depths: HashMap<B256, u64>,
    /// Chain of block hashes (most recent first)
    chain: Vec<B256>,
    /// Maximum chain length to track
    max_chain_length: usize,
}

impl Default for FinalityTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl FinalityTracker {
    /// Create a new finality tracker
    pub fn new() -> Self {
        Self {
            votes: VoteAggregator::new(),
            depths: HashMap::new(),
            chain: Vec::new(),
            max_chain_length: 1000,
        }
    }

    /// Add a new block to the chain
    pub fn add_block(&mut self, block_hash: B256) {
        // Add to front of chain (most recent)
        self.chain.insert(0, block_hash);
        
        // Update depths
        for (i, hash) in self.chain.iter().enumerate() {
            self.depths.insert(*hash, i as u64);
        }
        
        // Prune old entries
        if self.chain.len() > self.max_chain_length {
            let removed: Vec<_> = self.chain.drain(self.max_chain_length..).collect();
            for hash in removed {
                self.depths.remove(&hash);
            }
        }
    }

    /// Get the depth (confirmations) of a block
    pub fn depth(&self, block_hash: &B256) -> Option<u64> {
        self.depths.get(block_hash).copied()
    }

    /// Get the finality status of a block
    pub fn status(&self, block_hash: &B256, validator_set: &ValidatorSet) -> FinalityStatus {
        // Check BFT finality first
        if self.votes.is_finalized(block_hash) {
            return FinalityStatus::FinalizedBft {
                votes: self.votes.vote_count(block_hash),
            };
        }

        // Check depth finality
        if let Some(depth) = self.depth(block_hash) {
            if depth >= config::IMPLICIT_FINALITY_DEPTH {
                return FinalityStatus::FinalizedDepth { depth };
            }
        }

        // Still pending
        FinalityStatus::Pending {
            votes: self.votes.vote_count(block_hash),
            threshold: validator_set.finality_threshold(),
        }
    }

    /// Check if a block is final
    pub fn is_final(&self, block_hash: &B256, validator_set: &ValidatorSet) -> bool {
        self.status(block_hash, validator_set).is_final()
    }

    /// Get mutable access to the vote aggregator
    pub fn votes_mut(&mut self) -> &mut VoteAggregator {
        &mut self.votes
    }

    /// Get reference to the vote aggregator
    pub fn votes(&self) -> &VoteAggregator {
        &self.votes
    }

    /// Get the latest finalized block
    pub fn latest_finalized(&self, validator_set: &ValidatorSet) -> Option<B256> {
        // First check for BFT finalized blocks
        for hash in &self.chain {
            if self.votes.is_finalized(hash) {
                return Some(*hash);
            }
        }

        // Then check for depth finalized
        for hash in &self.chain {
            if let Some(depth) = self.depth(hash) {
                if depth >= config::IMPLICIT_FINALITY_DEPTH {
                    return Some(*hash);
                }
            }
        }

        None
    }

    /// Prune data for blocks older than the given depth
    pub fn prune(&mut self, keep_depth: u64) {
        let cutoff_block = self.chain.len().saturating_sub(keep_depth as usize);
        
        if cutoff_block > 0 {
            let removed: Vec<_> = self.chain.drain(cutoff_block..).collect();
            for hash in &removed {
                self.depths.remove(hash);
            }
            
            // Also prune votes
            if let Some(oldest) = self.chain.last() {
                if let Some(&block_num) = self.depths.get(oldest) {
                    self.votes.prune_before(block_num.saturating_sub(10));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Validator, Vote};
    use alloy_primitives::{Address, U256};

    fn create_test_validator_set(count: usize) -> ValidatorSet {
        let validators: Vec<_> = (0..count)
            .map(|i| Validator::new(
                Address::repeat_byte(i as u8),
                U256::from(100u64),
                10,
            ))
            .collect();
        
        ValidatorSet::from_validators(validators, 1, 0)
    }

    #[test]
    fn test_depth_finality() {
        let validator_set = create_test_validator_set(100);
        let mut tracker = FinalityTracker::new();
        
        // Add 4 blocks
        let blocks: Vec<_> = (0..4).map(|i| B256::repeat_byte(i)).collect();
        for block in &blocks {
            tracker.add_block(*block);
        }
        
        // Block 0 (oldest) should be at depth 3
        assert_eq!(tracker.depth(&blocks[0]), Some(3));
        
        // Block 0 should be final (depth >= 3)
        assert!(tracker.is_final(&blocks[0], &validator_set));
        
        // Block 3 (newest) should not be final
        assert!(!tracker.is_final(&blocks[3], &validator_set));
    }

    #[test]
    fn test_bft_finality() {
        let validator_set = create_test_validator_set(100);
        let mut tracker = FinalityTracker::new();
        
        let block_hash = B256::repeat_byte(1);
        tracker.add_block(block_hash);
        
        // Not final yet (no votes, no depth)
        assert!(!tracker.is_final(&block_hash, &validator_set));
        
        // Add 67 votes (threshold)
        for i in 0..67u8 {
            let vote = Vote::new_unsigned(block_hash, 100, Address::repeat_byte(i));
            tracker.votes_mut().add_vote(vote, &validator_set).unwrap();
        }
        
        // Now should be final via BFT
        let status = tracker.status(&block_hash, &validator_set);
        assert!(matches!(status, FinalityStatus::FinalizedBft { votes: 67 }));
    }

    #[test]
    fn test_finality_status() {
        let validator_set = create_test_validator_set(100);
        let mut tracker = FinalityTracker::new();
        
        let block_hash = B256::repeat_byte(1);
        tracker.add_block(block_hash);
        
        // Initially pending
        let status = tracker.status(&block_hash, &validator_set);
        assert!(matches!(status, FinalityStatus::Pending { votes: 0, threshold: 67 }));
        
        // Add some votes
        for i in 0..30u8 {
            let vote = Vote::new_unsigned(block_hash, 100, Address::repeat_byte(i));
            tracker.votes_mut().add_vote(vote, &validator_set).unwrap();
        }
        
        let status = tracker.status(&block_hash, &validator_set);
        assert!(matches!(status, FinalityStatus::Pending { votes: 30, .. }));
    }
}
