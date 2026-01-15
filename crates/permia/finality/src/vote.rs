//! Vote messages and aggregation for BFT finality

use alloy_primitives::{Address, B256};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::{FinalityError, ValidatorSet};

/// A vote for a block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vote {
    /// The block being voted on
    pub block_hash: B256,
    /// Block number
    pub block_number: u64,
    /// Validator who cast the vote
    pub validator: Address,
    /// ECDSA signature (v, r, s concatenated)
    pub signature: Vec<u8>,
}

impl Vote {
    /// Create a new vote (without signature - for testing)
    pub fn new_unsigned(block_hash: B256, block_number: u64, validator: Address) -> Self {
        Self {
            block_hash,
            block_number,
            validator,
            signature: vec![0u8; 65],
        }
    }

    /// Get the message that should be signed
    pub fn signing_message(&self) -> B256 {
        use alloy_primitives::keccak256;
        
        let mut data = Vec::with_capacity(72);
        data.extend_from_slice(b"PERMIA_VOTE:");
        data.extend_from_slice(self.block_hash.as_slice());
        data.extend_from_slice(&self.block_number.to_be_bytes());
        
        keccak256(&data)
    }

    /// Verify the vote signature
    pub fn verify(&self) -> Result<(), FinalityError> {
        // TODO: Implement ECDSA signature verification
        // For now, accept all votes (will be implemented with proper crypto)
        Ok(())
    }
}

/// Message containing a vote for network propagation
#[derive(Debug, Clone)]
pub struct VoteMessage {
    /// The vote
    pub vote: Vote,
    /// Timestamp when vote was cast
    pub timestamp: u64,
}

impl VoteMessage {
    /// Create a new vote message
    pub fn new(vote: Vote) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        Self { vote, timestamp }
    }
}

/// Aggregates votes for blocks
#[derive(Debug, Default)]
pub struct VoteAggregator {
    /// Votes per block hash
    votes: HashMap<B256, HashMap<Address, Vote>>,
    /// Blocks that have reached finality
    finalized: HashSet<B256>,
}

impl VoteAggregator {
    /// Create a new vote aggregator
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a vote, returns true if this vote contributed to finality
    pub fn add_vote(
        &mut self,
        vote: Vote,
        validator_set: &ValidatorSet,
    ) -> Result<bool, FinalityError> {
        // Verify validator is in active set
        if !validator_set.is_validator(&vote.validator) {
            return Err(FinalityError::NotValidator(vote.validator));
        }

        // Verify signature
        vote.verify()?;

        let block_hash = vote.block_hash;
        let validator = vote.validator;

        // Check for duplicate
        let block_votes = self.votes.entry(block_hash).or_default();
        if block_votes.contains_key(&validator) {
            return Err(FinalityError::DuplicateVote(validator, block_hash));
        }

        // Add vote
        block_votes.insert(validator, vote);

        // Check if we've reached finality threshold
        let vote_count = block_votes.len();
        let threshold = validator_set.finality_threshold();

        if vote_count >= threshold && !self.finalized.contains(&block_hash) {
            self.finalized.insert(block_hash);
            return Ok(true);
        }

        Ok(false)
    }

    /// Get the number of votes for a block
    pub fn vote_count(&self, block_hash: &B256) -> usize {
        self.votes.get(block_hash).map(|v| v.len()).unwrap_or(0)
    }

    /// Check if a block has been finalized
    pub fn is_finalized(&self, block_hash: &B256) -> bool {
        self.finalized.contains(block_hash)
    }

    /// Get all votes for a block
    pub fn get_votes(&self, block_hash: &B256) -> Vec<&Vote> {
        self.votes
            .get(block_hash)
            .map(|v| v.values().collect())
            .unwrap_or_default()
    }

    /// Get all voters for a block
    pub fn get_voters(&self, block_hash: &B256) -> Vec<Address> {
        self.votes
            .get(block_hash)
            .map(|v| v.keys().copied().collect())
            .unwrap_or_default()
    }

    /// Clean up votes for blocks older than the given number
    pub fn prune_before(&mut self, block_number: u64) {
        self.votes.retain(|_, votes| {
            votes.values().any(|v| v.block_number >= block_number)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Validator;
    use alloy_primitives::U256;

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
    fn test_vote_creation() {
        let vote = Vote::new_unsigned(
            B256::repeat_byte(1),
            100,
            Address::repeat_byte(1),
        );
        
        assert_eq!(vote.block_number, 100);
    }

    #[test]
    fn test_vote_aggregation() {
        let validator_set = create_test_validator_set(100);
        let mut aggregator = VoteAggregator::new();
        
        let block_hash = B256::repeat_byte(1);
        
        // Add 66 votes (not enough for finality)
        for i in 0..66u8 {
            let vote = Vote::new_unsigned(block_hash, 100, Address::repeat_byte(i));
            let result = aggregator.add_vote(vote, &validator_set);
            assert!(result.is_ok());
            assert!(!result.unwrap()); // Not finalized yet
        }
        
        assert_eq!(aggregator.vote_count(&block_hash), 66);
        assert!(!aggregator.is_finalized(&block_hash));
        
        // Add 67th vote (triggers finality)
        let vote = Vote::new_unsigned(block_hash, 100, Address::repeat_byte(66));
        let result = aggregator.add_vote(vote, &validator_set).unwrap();
        assert!(result); // Finalized!
        assert!(aggregator.is_finalized(&block_hash));
    }

    #[test]
    fn test_duplicate_vote_rejected() {
        let validator_set = create_test_validator_set(10);
        let mut aggregator = VoteAggregator::new();
        
        let block_hash = B256::repeat_byte(1);
        let vote = Vote::new_unsigned(block_hash, 100, Address::repeat_byte(1));
        
        // First vote succeeds
        assert!(aggregator.add_vote(vote.clone(), &validator_set).is_ok());
        
        // Duplicate vote fails
        let result = aggregator.add_vote(vote, &validator_set);
        assert!(matches!(result, Err(FinalityError::DuplicateVote(_, _))));
    }

    #[test]
    fn test_non_validator_rejected() {
        let validator_set = create_test_validator_set(10);
        let mut aggregator = VoteAggregator::new();
        
        // Address 100 is not a validator (only 0-9 are)
        let vote = Vote::new_unsigned(
            B256::repeat_byte(1),
            100,
            Address::repeat_byte(100),
        );
        
        let result = aggregator.add_vote(vote, &validator_set);
        assert!(matches!(result, Err(FinalityError::NotValidator(_))));
    }
}
