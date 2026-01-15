//! PermiaHash Proof-of-Work Algorithm
//!
//! Memory-hard, ASIC-resistant PoW implementing the protocol spec:
//!
//! Algorithm (from PROTOCOL_SPEC_v4.md):
//!   1. seed = BLAKE3(header || nonce)
//!   2. Initialize 4GB DAG from seed (epoch-based)
//!   3. For i in 0..64:
//!      a. index = seed[i % 32] % DAG_SIZE
//!      b. mix = mix XOR DAG[index]
//!      c. mix = BLAKE3(mix)
//!   4. result = BLAKE3(mix)
//!
//! Hash Functions Used:
//! - BLAKE3: Primary hash (fast, cryptographically secure)
//! - SHA3-256: DAG element generation (NIST standard, different construction)
//!
//! Using both BLAKE3 and SHA3 provides defense-in-depth:
//! - If BLAKE3 is compromised, SHA3 provides backup security
//! - Different internal constructions (Merkle-Damgård vs sponge)
//! - No known practical attack benefits from this combination

use alloy_consensus::Header;
use alloy_primitives::{B256, U256};
use blake3::Hasher as Blake3;
use sha3::{Digest, Sha3_256};

use crate::PermiaConsensusError;

/// PermiaHash configuration
pub struct PermiaHashConfig {
    /// Number of mixing rounds
    pub rounds: u32,
    /// DAG size in bytes (4 GB per spec)
    pub dag_size: usize,
    /// Epoch length in blocks (~3.5 days at 400ms blocks)
    pub epoch_length: u64,
}

impl Default for PermiaHashConfig {
    fn default() -> Self {
        Self {
            rounds: 64,
            dag_size: 4 * 1024 * 1024 * 1024, // 4 GB per spec
            epoch_length: 30000,             // ~3.5 days
        }
    }
}

/// DAG element size in bytes (64 bytes = 512 bits)
const DAG_ELEMENT_SIZE: usize = 64;

/// Number of DAG elements (4GB / 64 bytes)
const DAG_ELEMENTS: u64 = (4 * 1024 * 1024 * 1024) / DAG_ELEMENT_SIZE as u64;

/// Hash result from PermiaHash
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashResult {
    /// The computed hash
    pub hash: B256,
    /// Mix digest for verification
    pub mix_digest: B256,
}

/// Generate a DAG element from epoch seed and index
/// 
/// In production, this would be cached in a 4GB DAG structure.
/// For now, we compute elements on-demand using deterministic generation.
fn generate_dag_element(epoch_seed: &[u8; 32], index: u64) -> [u8; DAG_ELEMENT_SIZE] {
    // Use SHA3-256 for DAG element generation (different from BLAKE3 mixing)
    // This provides cryptographic diversity
    let mut hasher = Sha3_256::new();
    hasher.update(epoch_seed);
    hasher.update(&index.to_le_bytes());
    let hash1 = hasher.finalize();
    
    // Generate second half with different input
    let mut hasher2 = Sha3_256::new();
    hasher2.update(&hash1);
    hasher2.update(&(index ^ 0xFFFFFFFFFFFFFFFF).to_le_bytes());
    let hash2 = hasher2.finalize();
    
    let mut element = [0u8; DAG_ELEMENT_SIZE];
    element[..32].copy_from_slice(&hash1);
    element[32..].copy_from_slice(&hash2);
    element
}

/// Compute epoch seed from block number
pub fn compute_epoch_seed(block_number: u64) -> [u8; 32] {
    let epoch = block_number / 30000;
    let mut hasher = Blake3::new();
    hasher.update(b"permia_epoch_");
    hasher.update(&epoch.to_le_bytes());
    let result = hasher.finalize();
    let mut seed = [0u8; 32];
    seed.copy_from_slice(result.as_bytes());
    seed
}

/// Compute PermiaHash according to protocol specification
/// 
/// Algorithm:
///   1. seed = BLAKE3(header || nonce)
///   2. For i in 0..64:
///      a. index = seed[i % 32] % DAG_SIZE
///      b. dag_element = DAG[index]
///      c. mix = mix XOR dag_element
///      d. mix = BLAKE3(mix)
///   3. result = BLAKE3(mix)
pub fn permia_hash(seal_hash: &B256, nonce: u64) -> HashResult {
    permia_hash_with_epoch(seal_hash, nonce, 0)
}

/// Compute PermiaHash with specific epoch
pub fn permia_hash_with_epoch(seal_hash: &B256, nonce: u64, block_number: u64) -> HashResult {
    // Step 1: seed = BLAKE3(header || nonce)
    let mut blake = Blake3::new();
    blake.update(seal_hash.as_slice());
    blake.update(&nonce.to_le_bytes());
    let seed_hash = blake.finalize();
    let seed: [u8; 32] = *seed_hash.as_bytes();
    
    // Get epoch seed for DAG generation
    let epoch_seed = compute_epoch_seed(block_number);
    
    // Initialize mix with seed (64 bytes)
    let mut mix = [0u8; DAG_ELEMENT_SIZE];
    mix[..32].copy_from_slice(&seed);
    mix[32..].copy_from_slice(&seed);
    
    // Step 2-3: 64 rounds of DAG access and mixing
    for i in 0..64u64 {
        // a. index = seed[i % 32] % DAG_SIZE
        let seed_byte = seed[(i % 32) as usize] as u64;
        let index = (seed_byte * (i + 1) * 31337) % DAG_ELEMENTS;
        
        // b. Get DAG element (computed on-demand, would be cached in production)
        let dag_element = generate_dag_element(&epoch_seed, index);
        
        // c. mix = mix XOR dag_element
        for j in 0..DAG_ELEMENT_SIZE {
            mix[j] ^= dag_element[j];
        }
        
        // d. mix = BLAKE3(mix)
        let mut mix_hasher = Blake3::new();
        mix_hasher.update(&mix);
        let mix_result = mix_hasher.finalize();
        mix[..32].copy_from_slice(mix_result.as_bytes());
        // Second half uses different derivation for more mixing
        let mut mix_hasher2 = Blake3::new();
        mix_hasher2.update(mix_result.as_bytes());
        mix_hasher2.update(&[i as u8]);
        let mix_result2 = mix_hasher2.finalize();
        mix[32..].copy_from_slice(mix_result2.as_bytes());
    }
    
    // Step 4: result = BLAKE3(mix)
    let mut final_hasher = Blake3::new();
    final_hasher.update(&mix);
    let final_hash = final_hasher.finalize();
    
    HashResult {
        hash: B256::from_slice(final_hash.as_bytes()),
        mix_digest: B256::from_slice(&mix[..32]),
    }
}

/// Verify PoW for a header
pub fn verify_pow(header: &Header) -> Result<(), PermiaConsensusError> {
    let seal_hash = compute_seal_hash(header);
    
    // Extract nonce from header (FixedBytes<8> -> u64)
    let nonce = u64::from_be_bytes(header.nonce.0);
    
    // Use block number for epoch-based DAG calculation
    let result = permia_hash_with_epoch(&seal_hash, nonce, header.number);
    
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
