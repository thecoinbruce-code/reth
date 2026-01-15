//! Difficulty adjustment algorithm for Permia

use alloy_consensus::Header;
use alloy_primitives::U256;

/// Target block time in milliseconds
const TARGET_BLOCK_TIME_MS: u64 = 400;

/// Difficulty adjustment calculator
#[derive(Debug, Clone)]
pub struct DifficultyCalculator {
    /// Target block time in milliseconds
    target_time_ms: u64,
    /// Maximum adjustment per block (fraction)
    max_adjustment: f64,
    /// Minimum difficulty
    min_difficulty: U256,
}

impl DifficultyCalculator {
    /// Create calculator with default parameters
    pub fn new() -> Self {
        Self {
            target_time_ms: TARGET_BLOCK_TIME_MS,
            max_adjustment: 0.25, // 25% max change per block
            min_difficulty: U256::from(1u64 << 20),
        }
    }
    
    /// Get minimum difficulty
    pub fn min_difficulty(&self) -> U256 {
        self.min_difficulty
    }
    
    /// Calculate difficulty for next block
    pub fn calculate(&self, parent: &Header, timestamp: u64) -> U256 {
        // Time since parent block
        let time_diff = timestamp.saturating_sub(parent.timestamp);
        
        // If timestamps are same, increase difficulty slightly
        if time_diff == 0 {
            return self.apply_adjustment(parent.difficulty, 0.1);
        }
        
        // Calculate adjustment based on actual vs target time
        let target = self.target_time_ms as f64;
        let actual = time_diff as f64;
        
        // adjustment = (target - actual) / target * 0.1
        let raw_adjustment = (target - actual) / target * 0.1;
        
        // Clamp to max adjustment
        let adjustment = raw_adjustment.clamp(-self.max_adjustment, self.max_adjustment);
        
        self.apply_adjustment(parent.difficulty, adjustment)
    }
    
    /// Apply adjustment to difficulty
    fn apply_adjustment(&self, difficulty: U256, adjustment: f64) -> U256 {
        let multiplier = 1.0 + adjustment;
        
        // Convert to fixed-point math
        let multiplier_fixed = (multiplier * 1_000_000.0) as u64;
        let new_difficulty = difficulty * U256::from(multiplier_fixed) / U256::from(1_000_000u64);
        
        // Enforce minimum
        if new_difficulty < self.min_difficulty {
            return self.min_difficulty;
        }
        
        new_difficulty
    }
}

impl Default for DifficultyCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, B256, Bloom, Bytes};
    
    fn test_header(difficulty: U256, timestamp: u64) -> Header {
        Header {
            parent_hash: B256::ZERO,
            ommers_hash: B256::ZERO,
            beneficiary: Address::ZERO,
            state_root: B256::ZERO,
            transactions_root: B256::ZERO,
            receipts_root: B256::ZERO,
            logs_bloom: Bloom::ZERO,
            difficulty,
            number: 1,
            gas_limit: 30_000_000,
            gas_used: 0,
            timestamp,
            extra_data: Bytes::new(),
            mix_hash: B256::ZERO,
            nonce: alloy_primitives::FixedBytes::ZERO,
            base_fee_per_gas: Some(1_000_000_000),
            withdrawals_root: None,
            blob_gas_used: None,
            excess_blob_gas: None,
            parent_beacon_block_root: None,
            requests_hash: None,
        }
    }
    
    #[test]
    fn test_difficulty_increase_on_fast_block() {
        let calc = DifficultyCalculator::new();
        let parent = test_header(U256::from(1_000_000u64), 1000);
        
        // Block arrived 200ms after parent (faster than 400ms target)
        let new_diff = calc.calculate(&parent, 1200);
        
        // Difficulty should increase
        assert!(new_diff > parent.difficulty);
    }
    
    #[test]
    fn test_difficulty_decrease_on_slow_block() {
        let calc = DifficultyCalculator::new();
        let parent = test_header(U256::from(10_000_000u64), 1000);
        
        // Block arrived 2000ms after parent (5x slower than 400ms target)
        let new_diff = calc.calculate(&parent, 3000);
        
        // Difficulty should decrease
        assert!(new_diff < parent.difficulty);
    }
}
