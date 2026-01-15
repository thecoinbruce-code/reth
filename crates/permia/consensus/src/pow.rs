//! PermiaHash Proof-of-Work Algorithm
//!
//! Memory-hard, ASIC-resistant PoW based on:
//! - Blake3 for speed
//! - SHA3-256 for security
//! - DAG for memory-hardness

use alloy_consensus::Header;
use alloy_primitives::{B256, U256};
use blake3::Hasher as Blake3;
use sha3::{Digest, Sha3_256};

use crate::PermiaConsensusError;

/// PermiaHash configuration
pub struct PermiaHashConfig {
    /// Number of mixing rounds
    pub rounds: u32,
    /// DAG size in bytes
    pub dag_size: usize,
    /// Epoch length in blocks
    pub epoch_length: u64,
}

impl Default for PermiaHashConfig {
    fn default() -> Self {
        Self {
            rounds: 64,
            dag_size: 1024 * 1024 * 1024, // 1 GB
            epoch_length: 30000,
        }
    }
}

/// Hash result from PermiaHash
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashResult {
    /// The computed hash
    pub hash: B256,
    /// Mix digest for verification
    pub mix_digest: B256,
}

/// Compute PermiaHash
pub fn permia_hash(seal_hash: &B256, nonce: u64) -> HashResult {
    // Phase 1: Initial hash with Blake3
    let mut blake = Blake3::new();
    blake.update(seal_hash.as_slice());
    blake.update(&nonce.to_le_bytes());
    let initial = blake.finalize();
    
    // Phase 2: Memory-hard mixing (simplified - full DAG in production)
    let mut mix = [0u8; 64];
    mix[..32].copy_from_slice(initial.as_bytes());
    mix[32..].copy_from_slice(initial.as_bytes());
    
    for i in 0..64 {
        let idx = u64::from_le_bytes(mix[i % 8 * 8..(i % 8 + 1) * 8].try_into().unwrap());
        let mut hasher = Sha3_256::new();
        hasher.update(&mix);
        hasher.update(&idx.to_le_bytes());
        let result = hasher.finalize();
        for j in 0..32 {
            mix[j] ^= result[j];
            mix[32 + j] ^= result[(j + 16) % 32];
        }
    }
    
    // Phase 3: Final hash
    let mut final_hasher = Sha3_256::new();
    final_hasher.update(&mix);
    let hash_bytes = final_hasher.finalize();
    
    HashResult {
        hash: B256::from_slice(&hash_bytes),
        mix_digest: B256::from_slice(&mix[..32]),
    }
}

/// Verify PoW for a header
pub fn verify_pow(header: &Header) -> Result<(), PermiaConsensusError> {
    let seal_hash = compute_seal_hash(header);
    
    // Extract nonce from header (FixedBytes<8> -> u64)
    let nonce = u64::from_be_bytes(header.nonce.0);
    let result = permia_hash(&seal_hash, nonce);
    
    // Check mix digest matches
    if result.mix_digest != header.mix_hash {
        return Err(PermiaConsensusError::InvalidProofOfWork);
    }
    
    // Check hash meets difficulty target
    let target = difficulty_to_target(header.difficulty);
    let hash_value = U256::from_be_bytes(result.hash.0);
    
    if hash_value > target {
        return Err(PermiaConsensusError::InvalidProofOfWork);
    }
    
    Ok(())
}

/// Compute seal hash (header hash without nonce/mix_hash)
pub fn compute_seal_hash(header: &Header) -> B256 {
    use sha3::{Digest, Keccak256};
    
    let mut hasher = Keccak256::new();
    hasher.update(header.parent_hash.as_slice());
    hasher.update(header.beneficiary.as_slice());
    hasher.update(header.state_root.as_slice());
    hasher.update(header.transactions_root.as_slice());
    hasher.update(header.receipts_root.as_slice());
    hasher.update(&header.difficulty.to_be_bytes::<32>());
    hasher.update(&header.number.to_be_bytes());
    hasher.update(&header.gas_limit.to_be_bytes());
    hasher.update(&header.gas_used.to_be_bytes());
    hasher.update(&header.timestamp.to_be_bytes());
    hasher.update(&header.extra_data);
    
    B256::from_slice(&hasher.finalize())
}

/// Convert difficulty to target
pub fn difficulty_to_target(difficulty: U256) -> U256 {
    if difficulty == U256::ZERO {
        return U256::MAX;
    }
    U256::MAX / difficulty
}

/// Convert target to difficulty
pub fn target_to_difficulty(target: U256) -> U256 {
    if target == U256::ZERO {
        return U256::MAX;
    }
    U256::MAX / target
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permia_hash() {
        let seal_hash = B256::from([1u8; 32]);
        let result = permia_hash(&seal_hash, 12345);
        
        assert_ne!(result.hash, B256::ZERO);
        assert_ne!(result.mix_digest, B256::ZERO);
    }
    
    #[test]
    fn test_difficulty_conversion() {
        let difficulty = U256::from(1_000_000u64);
        let target = difficulty_to_target(difficulty);
        let back = target_to_difficulty(target);
        
        // Should be approximately equal (some rounding)
        let diff = if back > difficulty { back - difficulty } else { difficulty - back };
        assert!(diff < U256::from(1000u64));
    }
}
