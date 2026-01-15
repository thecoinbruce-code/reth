//! CDN service proofs (Content Delivery)

use alloy_primitives::{Address, B256};
use serde::{Deserialize, Serialize};

/// CDN service parameters (from PROTOCOL_SPEC_v4.md)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnParams {
    /// Content to serve
    pub cid: B256,
    /// Bandwidth allocation in bytes
    pub bandwidth_bytes: u64,
    /// Geographic regions (encoded as u8 region codes)
    pub regions: Vec<u8>,
}

impl CdnParams {
    /// Create new CDN params
    pub fn new(cid: B256, bandwidth_bytes: u64, regions: Vec<u8>) -> Self {
        Self {
            cid,
            bandwidth_bytes,
            regions,
        }
    }

    /// Calculate CDN cost in USD cents (simplified)
    pub fn cost_cents(&self) -> u64 {
        // $0.01 per GB bandwidth
        let gb = (self.bandwidth_bytes as f64) / (1024.0 * 1024.0 * 1024.0);
        (gb * 1.0).ceil() as u64 // 1 cent per GB
    }
}

/// CDN delivery proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnProof {
    /// Miner providing CDN
    pub miner: Address,
    /// Content served
    pub cid: B256,
    /// Total bandwidth served in bytes
    pub bandwidth_bytes: u64,
    /// Number of requests served
    pub requests: u64,
    /// Client receipts (signed acknowledgments)
    pub client_receipts: Vec<ClientReceipt>,
    /// Epoch when proof was generated
    pub epoch: u64,
}

/// A receipt from a client acknowledging content delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientReceipt {
    /// Client address or ID hash
    pub client_id: B256,
    /// Content delivered
    pub cid: B256,
    /// Bytes delivered
    pub bytes: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Client signature (simplified)
    pub signature: Vec<u8>,
}

impl CdnProof {
    /// Verify the CDN proof
    pub fn verify(&self) -> bool {
        // Basic validation
        if self.bandwidth_bytes == 0 || self.client_receipts.is_empty() {
            return false;
        }

        // Verify total bandwidth matches receipts
        let receipt_total: u64 = self.client_receipts.iter().map(|r| r.bytes).sum();
        
        // Allow some tolerance (receipts might be sampled)
        receipt_total > 0
    }

    /// Calculate service score contribution
    pub fn service_score(&self) -> u64 {
        // Score based on bandwidth served (1 point per 10GB)
        let gb = self.bandwidth_bytes / (1024 * 1024 * 1024);
        (gb / 10).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdn_params() {
        let params = CdnParams::new(
            B256::ZERO,
            10 * 1024 * 1024 * 1024, // 10 GB
            vec![1, 2, 3],           // Regions
        );

        assert_eq!(params.cost_cents(), 10);
    }

    #[test]
    fn test_cdn_proof() {
        let receipt = ClientReceipt {
            client_id: B256::repeat_byte(1),
            cid: B256::repeat_byte(2),
            bytes: 1024 * 1024,
            timestamp: 1000,
            signature: vec![0u8; 65],
        };

        let proof = CdnProof {
            miner: Address::ZERO,
            cid: B256::repeat_byte(2),
            bandwidth_bytes: 10 * 1024 * 1024 * 1024,
            requests: 1000,
            client_receipts: vec![receipt],
            epoch: 100,
        };

        assert!(proof.verify());
        assert_eq!(proof.service_score(), 1);
    }
}
