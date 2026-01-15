//! Compute service proofs (WASM Execution)

use alloy_primitives::{Address, B256, Bytes};
use serde::{Deserialize, Serialize};

/// Compute service parameters (from PROTOCOL_SPEC_v4.md)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeParams {
    /// WASM binary CID
    pub wasm_cid: B256,
    /// Entry function name
    pub function: String,
    /// Function arguments
    pub args: Vec<u8>,
    /// Maximum compute cycles allowed
    pub max_cycles: u64,
}

impl ComputeParams {
    /// Create new compute params
    pub fn new(wasm_cid: B256, function: String, args: Vec<u8>, max_cycles: u64) -> Self {
        Self {
            wasm_cid,
            function,
            args,
            max_cycles,
        }
    }

    /// Calculate compute cost in USD cents (simplified)
    pub fn cost_cents(&self) -> u64 {
        // $0.000001 per 1M cycles
        let m_cycles = self.max_cycles / 1_000_000;
        (m_cycles / 10).max(1) // 0.1 cents per 1M cycles
    }
}

/// Compute execution proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeProof {
    /// Miner who executed the computation
    pub miner: Address,
    /// WASM binary executed
    pub wasm_cid: B256,
    /// Hash of input data
    pub input_hash: B256,
    /// Hash of output data
    pub output_hash: B256,
    /// Cycles consumed
    pub cycles: u64,
    /// Execution trace hash (for verification)
    pub trace_hash: B256,
    /// Epoch when proof was generated
    pub epoch: u64,
}

impl ComputeProof {
    /// Verify the compute proof
    pub fn verify(&self) -> bool {
        // Basic validation
        // In production, would verify the execution trace
        self.cycles > 0 && self.trace_hash != B256::ZERO
    }

    /// Calculate service score contribution
    pub fn service_score(&self) -> u64 {
        // Score based on cycles executed (1 point per 1B cycles)
        let b_cycles = self.cycles / 1_000_000_000;
        b_cycles.max(1)
    }
}

/// Result of a compute execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Output data (if successful)
    pub output: Vec<u8>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Cycles consumed
    pub cycles: u64,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_params() {
        let params = ComputeParams::new(
            B256::ZERO,
            "main".to_string(),
            vec![1, 2, 3],
            1_000_000_000, // 1B cycles
        );

        assert!(params.cost_cents() > 0);
    }

    #[test]
    fn test_compute_proof() {
        let proof = ComputeProof {
            miner: Address::ZERO,
            wasm_cid: B256::repeat_byte(1),
            input_hash: B256::repeat_byte(2),
            output_hash: B256::repeat_byte(3),
            cycles: 1_000_000_000,
            trace_hash: B256::repeat_byte(4),
            epoch: 100,
        };

        assert!(proof.verify());
        assert_eq!(proof.service_score(), 1);
    }
}
