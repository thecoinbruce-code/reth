//! Storage service proofs (Proof of Spacetime)

use alloy_primitives::{Address, B256};
use serde::{Deserialize, Serialize};

/// Storage service parameters (from PROTOCOL_SPEC_v4.md)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageParams {
    /// Content identifier
    pub cid: B256,
    /// Content size in bytes
    pub size_bytes: u64,
    /// Storage duration in seconds
    pub duration_seconds: u64,
    /// Replication factor (minimum 3)
    pub replication: u8,
}

impl StorageParams {
    /// Create new storage params
    pub fn new(cid: B256, size_bytes: u64, duration_seconds: u64, replication: u8) -> Self {
        Self {
            cid,
            size_bytes,
            duration_seconds,
            replication: replication.max(3), // Minimum 3
        }
    }

    /// Calculate storage cost in USD cents per month (simplified)
    pub fn monthly_cost_cents(&self) -> u64 {
        // $0.001 per GB per month base rate
        let gb = (self.size_bytes as f64) / (1024.0 * 1024.0 * 1024.0);
        let months = (self.duration_seconds as f64) / (30.0 * 24.0 * 3600.0);
        let base_cost = gb * months * 0.1; // 0.1 cents per GB-month
        let replicated_cost = base_cost * (self.replication as f64);
        replicated_cost.ceil() as u64
    }
}

/// Storage proof (Proof of Spacetime)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProof {
    /// Miner providing storage
    pub miner: Address,
    /// Content being stored
    pub cid: B256,
    /// Size being stored
    pub size_bytes: u64,
    /// Merkle root of stored data
    pub merkle_root: B256,
    /// Proof of random access (challenge-response)
    pub challenge_index: u64,
    /// Response to challenge
    pub challenge_response: B256,
    /// Merkle proof for challenge
    pub merkle_proof: Vec<B256>,
    /// Epoch when proof was generated
    pub epoch: u64,
}

impl StorageProof {
    /// Verify the storage proof
    pub fn verify(&self) -> bool {
        // TODO: Implement full Merkle proof verification
        // For now, basic validation
        !self.merkle_proof.is_empty() && self.size_bytes > 0
    }

    /// Calculate service score contribution
    pub fn service_score(&self) -> u64 {
        // Score based on size stored (1 point per GB)
        let gb = self.size_bytes / (1024 * 1024 * 1024);
        gb.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_params() {
        let params = StorageParams::new(
            B256::ZERO,
            1024 * 1024 * 1024, // 1 GB
            30 * 24 * 3600,     // 30 days
            3,
        );

        assert_eq!(params.replication, 3);
        assert!(params.monthly_cost_cents() > 0);
    }

    #[test]
    fn test_storage_proof() {
        let proof = StorageProof {
            miner: Address::ZERO,
            cid: B256::repeat_byte(1),
            size_bytes: 1024 * 1024 * 1024,
            merkle_root: B256::repeat_byte(2),
            challenge_index: 42,
            challenge_response: B256::repeat_byte(3),
            merkle_proof: vec![B256::repeat_byte(4)],
            epoch: 100,
        };

        assert!(proof.verify());
        assert_eq!(proof.service_score(), 1);
    }
}
