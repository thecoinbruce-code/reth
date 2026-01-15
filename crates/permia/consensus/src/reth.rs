//! Reth Consensus Integration for Permia
//!
//! Implements the Reth Consensus traits for PermiaHash PoW.

use crate::{difficulty::DifficultyCalculator, pow, PermiaConsensusError};
use alloy_consensus::Header;
use alloy_primitives::U256;
use reth_chainspec::ChainSpec;
use reth_consensus::{Consensus, ConsensusError, FullConsensus, HeaderValidator};
use reth_consensus_common::validation::{
    validate_against_parent_gas_limit, validate_against_parent_hash_number,
    validate_against_parent_timestamp, validate_block_pre_execution, validate_body_against_header,
    validate_header_extra_data, validate_header_gas,
};
use reth_primitives_traits::{Block, BlockHeader, NodePrimitives, RecoveredBlock, SealedBlock, SealedHeader};
use reth_execution_types::BlockExecutionResult;
use std::{error::Error, fmt::Debug, sync::Arc};

/// Custom error for Permia consensus
#[derive(Debug, Clone)]
struct PermiaError(String);

impl std::fmt::Display for PermiaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for PermiaError {}

fn custom_error(msg: impl Into<String>) -> ConsensusError {
    ConsensusError::Custom(Arc::new(PermiaError(msg.into())))
}

/// Maximum allowed extra data size in bytes
const MAX_EXTRA_DATA_SIZE: usize = 32;

/// Permia Proof-of-Work Consensus
///
/// Validates blocks using PermiaHash and difficulty adjustment.
#[derive(Debug, Clone)]
pub struct PermiaPoWConsensus {
    /// Chain specification
    chain_spec: Arc<ChainSpec>,
    /// Difficulty calculator
    difficulty_calc: DifficultyCalculator,
    /// Maximum extra data size
    max_extra_data_size: usize,
}

impl PermiaPoWConsensus {
    /// Create a new instance
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self {
            chain_spec,
            difficulty_calc: DifficultyCalculator::new(),
            max_extra_data_size: MAX_EXTRA_DATA_SIZE,
        }
    }

    /// Get the chain spec
    pub fn chain_spec(&self) -> &Arc<ChainSpec> {
        &self.chain_spec
    }

    /// Validate PoW for a header
    fn validate_pow(&self, header: &Header) -> Result<(), ConsensusError> {
        pow::verify_pow(header).map_err(|e| match e {
            PermiaConsensusError::InvalidProofOfWork => {
                custom_error("Invalid PermiaHash proof of work")
            }
            _ => custom_error(format!("{}", e)),
        })
    }

    /// Validate difficulty
    fn validate_difficulty(
        &self,
        header: &Header,
        parent: &Header,
    ) -> Result<(), ConsensusError> {
        let expected = self.difficulty_calc.calculate(parent, header.timestamp);
        
        // Allow some tolerance for difficulty
        let min_allowed = expected * U256::from(95u64) / U256::from(100u64);
        let max_allowed = expected * U256::from(105u64) / U256::from(100u64);
        
        if header.difficulty < min_allowed || header.difficulty > max_allowed {
            return Err(custom_error(format!(
                "Invalid difficulty: expected ~{}, got {}",
                expected, header.difficulty
            )));
        }
        
        Ok(())
    }
}

impl<H> HeaderValidator<H> for PermiaPoWConsensus
where
    H: BlockHeader + AsRef<Header>,
{
    fn validate_header(&self, header: &SealedHeader<H>) -> Result<(), ConsensusError> {
        let h = header.header();
        
        // Validate extra data size
        validate_header_extra_data(h, self.max_extra_data_size)?;
        
        // Validate gas
        validate_header_gas(h)?;
        
        // Validate PoW
        self.validate_pow(h.as_ref())?;
        
        // Validate difficulty is non-zero
        if h.difficulty().is_zero() {
            return Err(custom_error("Difficulty cannot be zero in PoW"));
        }
        
        Ok(())
    }

    fn validate_header_against_parent(
        &self,
        header: &SealedHeader<H>,
        parent: &SealedHeader<H>,
    ) -> Result<(), ConsensusError> {
        // Standard validations
        validate_against_parent_hash_number(header.header(), parent)?;
        validate_against_parent_timestamp(header.header(), parent.header())?;
        validate_against_parent_gas_limit(header, parent, &*self.chain_spec)?;
        
        // Validate difficulty adjustment
        self.validate_difficulty(header.header().as_ref(), parent.header().as_ref())?;
        
        Ok(())
    }
}

impl<B> Consensus<B> for PermiaPoWConsensus
where
    B: Block,
    B::Header: AsRef<Header>,
{
    fn validate_body_against_header(
        &self,
        body: &B::Body,
        header: &SealedHeader<B::Header>,
    ) -> Result<(), ConsensusError> {
        validate_body_against_header(body, header.header())
    }

    fn validate_block_pre_execution(&self, block: &SealedBlock<B>) -> Result<(), ConsensusError> {
        validate_block_pre_execution(block, &*self.chain_spec)
    }
}

impl<N> FullConsensus<N> for PermiaPoWConsensus
where
    N: NodePrimitives,
    N::BlockHeader: AsRef<Header>,
{
    fn validate_block_post_execution(
        &self,
        _block: &RecoveredBlock<N::Block>,
        _result: &BlockExecutionResult<N::Receipt>,
    ) -> Result<(), ConsensusError> {
        // For PoW, we don't have additional post-execution validation
        // The PoW validation happens in header validation
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reth_chainspec::PERMIA_DEV;

    #[test]
    fn test_consensus_creation() {
        let consensus = PermiaPoWConsensus::new(PERMIA_DEV.clone());
        assert_eq!(consensus.chain_spec().chain.id(), 42071);
    }
}
