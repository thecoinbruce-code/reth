//! Block template for mining
//!
//! A block template contains all the information needed to mine a new block,
//! except for the nonce and mix_hash which are found through PoW.

use alloy_consensus::Header;
use alloy_primitives::{Address, B256, Bytes, U256};
use permia_consensus::pow::compute_seal_hash;

/// Block template for mining
///
/// Contains all block data except nonce/mix_hash which are found by mining.
#[derive(Debug, Clone)]
pub struct BlockTemplate {
    /// Parent block hash
    pub parent_hash: B256,
    /// Block number
    pub number: u64,
    /// Timestamp (milliseconds since epoch)
    pub timestamp: u64,
    /// Miner address (coinbase)
    pub beneficiary: Address,
    /// State root after transactions
    pub state_root: B256,
    /// Transactions root
    pub transactions_root: B256,
    /// Receipts root
    pub receipts_root: B256,
    /// Difficulty target
    pub difficulty: U256,
    /// Gas limit
    pub gas_limit: u64,
    /// Gas used
    pub gas_used: u64,
    /// Extra data (max 32 bytes)
    pub extra_data: Bytes,
    /// Base fee per gas (EIP-1559)
    pub base_fee_per_gas: Option<u64>,
}

impl BlockTemplate {
    /// Create a new block template
    pub fn new(
        parent_hash: B256,
        number: u64,
        timestamp: u64,
        beneficiary: Address,
        difficulty: U256,
    ) -> Self {
        Self {
            parent_hash,
            number,
            timestamp,
            beneficiary,
            state_root: B256::ZERO,
            transactions_root: B256::ZERO,
            receipts_root: B256::ZERO,
            difficulty,
            gas_limit: 60_000_000, // 60M gas limit per spec
            gas_used: 0,
            extra_data: Bytes::from_static(b"permia"),
            base_fee_per_gas: Some(1_000_000_000), // 1 gwei
        }
    }

    /// Convert template to a header (without nonce/mix_hash)
    pub fn to_header(&self) -> Header {
        Header {
            parent_hash: self.parent_hash,
            ommers_hash: B256::ZERO,
            beneficiary: self.beneficiary,
            state_root: self.state_root,
            transactions_root: self.transactions_root,
            receipts_root: self.receipts_root,
            logs_bloom: alloy_primitives::Bloom::ZERO,
            difficulty: self.difficulty,
            number: self.number,
            gas_limit: self.gas_limit,
            gas_used: self.gas_used,
            timestamp: self.timestamp,
            extra_data: self.extra_data.clone(),
            mix_hash: B256::ZERO,
            nonce: alloy_primitives::FixedBytes::ZERO,
            base_fee_per_gas: self.base_fee_per_gas,
            withdrawals_root: None,
            blob_gas_used: None,
            excess_blob_gas: None,
            parent_beacon_block_root: None,
            requests_hash: None,
        }
    }

    /// Compute the seal hash for this template
    pub fn seal_hash(&self) -> B256 {
        compute_seal_hash(&self.to_header())
    }

    /// Get the target value from difficulty
    pub fn target(&self) -> U256 {
        permia_consensus::pow::difficulty_to_target(self.difficulty)
    }
}

/// Builder for creating block templates
#[derive(Debug, Default)]
pub struct BlockTemplateBuilder {
    template: Option<BlockTemplate>,
}

impl BlockTemplateBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self { template: None }
    }

    /// Set the parent block
    pub fn parent(mut self, hash: B256, number: u64) -> Self {
        let template = self.template.get_or_insert_with(|| {
            BlockTemplate::new(hash, number + 1, 0, Address::ZERO, U256::from(1u64))
        });
        template.parent_hash = hash;
        template.number = number + 1;
        self
    }

    /// Set the miner address
    pub fn beneficiary(mut self, address: Address) -> Self {
        if let Some(ref mut t) = self.template {
            t.beneficiary = address;
        }
        self
    }

    /// Set the timestamp
    pub fn timestamp(mut self, ts: u64) -> Self {
        if let Some(ref mut t) = self.template {
            t.timestamp = ts;
        }
        self
    }

    /// Set the difficulty
    pub fn difficulty(mut self, diff: U256) -> Self {
        if let Some(ref mut t) = self.template {
            t.difficulty = diff;
        }
        self
    }

    /// Set the state root
    pub fn state_root(mut self, root: B256) -> Self {
        if let Some(ref mut t) = self.template {
            t.state_root = root;
        }
        self
    }

    /// Set extra data
    pub fn extra_data(mut self, data: Bytes) -> Self {
        if let Some(ref mut t) = self.template {
            t.extra_data = data;
        }
        self
    }

    /// Build the template
    pub fn build(self) -> Option<BlockTemplate> {
        self.template
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_template() {
        let template = BlockTemplate::new(
            B256::ZERO,
            1,
            1000,
            Address::ZERO,
            U256::from(1_000_000u64),
        );

        assert_eq!(template.number, 1);
        assert_eq!(template.gas_limit, 60_000_000);
        
        let header = template.to_header();
        assert_eq!(header.number, 1);
    }

    #[test]
    fn test_template_builder() {
        let template = BlockTemplateBuilder::new()
            .parent(B256::ZERO, 0)
            .beneficiary(Address::ZERO)
            .timestamp(1000)
            .difficulty(U256::from(1_000_000u64))
            .build()
            .unwrap();

        assert_eq!(template.number, 1);
        assert_eq!(template.timestamp, 1000);
    }
}
