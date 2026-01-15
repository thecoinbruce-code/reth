//! Service proof types

use alloy_primitives::{Address, B256, Bytes};
use serde::{Deserialize, Serialize};

use crate::{ServiceError, ServiceType};

/// Service proof type identifier (from PROTOCOL_SPEC_v4.md)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ServiceProofType {
    /// Proof of Spacetime for storage
    StoragePoST = 0x01,
    /// Delivery receipts for CDN
    CdnDelivery = 0x02,
    /// Execution proof for compute
    ComputeExecution = 0x03,
}

/// Service proof data (type-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceProofData {
    /// Storage proof data
    Storage {
        /// Content identifier
        cid: B256,
        /// Merkle proof of storage
        merkle_proof: Vec<B256>,
        /// Challenge response
        challenge_response: B256,
    },
    /// CDN proof data
    Cdn {
        /// Content identifier
        cid: B256,
        /// Bandwidth served (bytes)
        bandwidth_bytes: u64,
        /// Client receipts (hashes)
        client_receipts: Vec<B256>,
    },
    /// Compute proof data
    Compute {
        /// WASM binary CID
        wasm_cid: B256,
        /// Input hash
        input_hash: B256,
        /// Output hash
        output_hash: B256,
        /// Cycles consumed
        cycles: u64,
    },
}

/// A service proof from a miner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceProof {
    /// Type of service proof
    pub proof_type: ServiceProofType,
    /// Miner who generated the proof
    pub miner: Address,
    /// Proof epoch (1 epoch = 1 hour)
    pub epoch: u64,
    /// Type-specific proof data
    pub data: ServiceProofData,
    /// Miner signature
    pub signature: Vec<u8>,
}

impl ServiceProof {
    /// Create a new storage proof
    pub fn new_storage(
        miner: Address,
        epoch: u64,
        cid: B256,
        merkle_proof: Vec<B256>,
        challenge_response: B256,
    ) -> Self {
        Self {
            proof_type: ServiceProofType::StoragePoST,
            miner,
            epoch,
            data: ServiceProofData::Storage {
                cid,
                merkle_proof,
                challenge_response,
            },
            signature: Vec::new(),
        }
    }

    /// Create a new CDN proof
    pub fn new_cdn(
        miner: Address,
        epoch: u64,
        cid: B256,
        bandwidth_bytes: u64,
        client_receipts: Vec<B256>,
    ) -> Self {
        Self {
            proof_type: ServiceProofType::CdnDelivery,
            miner,
            epoch,
            data: ServiceProofData::Cdn {
                cid,
                bandwidth_bytes,
                client_receipts,
            },
            signature: Vec::new(),
        }
    }

    /// Create a new compute proof
    pub fn new_compute(
        miner: Address,
        epoch: u64,
        wasm_cid: B256,
        input_hash: B256,
        output_hash: B256,
        cycles: u64,
    ) -> Self {
        Self {
            proof_type: ServiceProofType::ComputeExecution,
            miner,
            epoch,
            data: ServiceProofData::Compute {
                wasm_cid,
                input_hash,
                output_hash,
                cycles,
            },
            signature: Vec::new(),
        }
    }

    /// Get the service type
    pub fn service_type(&self) -> ServiceType {
        match self.proof_type {
            ServiceProofType::StoragePoST => ServiceType::Storage,
            ServiceProofType::CdnDelivery => ServiceType::Cdn,
            ServiceProofType::ComputeExecution => ServiceType::Compute,
        }
    }

    /// Verify the proof (basic validation)
    pub fn verify(&self, current_epoch: u64) -> Result<(), ServiceError> {
        // Check epoch is not too old (max 24 epochs = 24 hours)
        if self.epoch + 24 < current_epoch {
            return Err(ServiceError::ProofExpired(self.epoch, current_epoch));
        }

        // TODO: Implement full verification for each proof type
        // - Storage: verify merkle proof against chain state
        // - CDN: verify client receipt signatures
        // - Compute: verify execution trace

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_proof() {
        let proof = ServiceProof::new_storage(
            Address::ZERO,
            100,
            B256::repeat_byte(1),
            vec![B256::repeat_byte(2)],
            B256::repeat_byte(3),
        );

        assert_eq!(proof.service_type(), ServiceType::Storage);
        assert!(proof.verify(100).is_ok());
        assert!(proof.verify(200).is_err()); // Expired
    }

    #[test]
    fn test_cdn_proof() {
        let proof = ServiceProof::new_cdn(
            Address::ZERO,
            100,
            B256::repeat_byte(1),
            1_000_000,
            vec![B256::repeat_byte(2)],
        );

        assert_eq!(proof.service_type(), ServiceType::Cdn);
    }

    #[test]
    fn test_compute_proof() {
        let proof = ServiceProof::new_compute(
            Address::ZERO,
            100,
            B256::repeat_byte(1),
            B256::repeat_byte(2),
            B256::repeat_byte(3),
            1_000_000,
        );

        assert_eq!(proof.service_type(), ServiceType::Compute);
    }
}
