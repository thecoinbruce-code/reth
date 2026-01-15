//! Service multiplier calculation for mining rewards

use crate::{ServiceProof, ServiceProofType, ServiceType};

/// Maximum service multiplier (2.0x)
pub const MAX_MULTIPLIER: f64 = 2.0;

/// Service multiplier components
#[derive(Debug, Clone, Default)]
pub struct ServiceMultiplier {
    /// Storage proof bonus (0.1 to 0.3)
    pub storage: f64,
    /// Compute proof bonus (0.1 to 0.3)
    pub compute: f64,
    /// CDN proof bonus (0.05 to 0.15)
    pub cdn: f64,
    /// Uptime bonus (0.1 for 99%+)
    pub uptime: f64,
    /// Geographic bonus (0.2 to 0.5)
    pub geographic: f64,
}

impl ServiceMultiplier {
    /// Create a new multiplier with no bonuses
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate total multiplier (capped at MAX_MULTIPLIER)
    pub fn total(&self) -> f64 {
        let sum = 1.0 + self.storage + self.compute + self.cdn + self.uptime + self.geographic;
        sum.min(MAX_MULTIPLIER)
    }

    /// Add storage bonus based on proof quality
    pub fn with_storage(mut self, proof_quality: f64) -> Self {
        // quality: 0.0 to 1.0 -> bonus: 0.1 to 0.3
        self.storage = 0.1 + (proof_quality.clamp(0.0, 1.0) * 0.2);
        self
    }

    /// Add compute bonus based on proof quality
    pub fn with_compute(mut self, proof_quality: f64) -> Self {
        // quality: 0.0 to 1.0 -> bonus: 0.1 to 0.3
        self.compute = 0.1 + (proof_quality.clamp(0.0, 1.0) * 0.2);
        self
    }

    /// Add CDN bonus based on bandwidth served
    pub fn with_cdn(mut self, bandwidth_factor: f64) -> Self {
        // factor: 0.0 to 1.0 -> bonus: 0.05 to 0.15
        self.cdn = 0.05 + (bandwidth_factor.clamp(0.0, 1.0) * 0.1);
        self
    }

    /// Add uptime bonus
    pub fn with_uptime(mut self, uptime_percent: f64) -> Self {
        // 99%+ uptime gets full bonus
        if uptime_percent >= 99.0 {
            self.uptime = 0.1;
        } else if uptime_percent >= 95.0 {
            self.uptime = 0.05;
        }
        self
    }

    /// Add geographic bonus based on region rarity
    pub fn with_geographic(mut self, rarity_factor: f64) -> Self {
        // factor: 0.0 to 1.0 -> bonus: 0.2 to 0.5
        self.geographic = 0.2 + (rarity_factor.clamp(0.0, 1.0) * 0.3);
        self
    }
}

/// Calculate multiplier from a set of service proofs
pub fn calculate_multiplier(
    proofs: &[ServiceProof],
    uptime_percent: f64,
    geographic_rarity: f64,
) -> ServiceMultiplier {
    let mut multiplier = ServiceMultiplier::new();

    // Check for each proof type
    let mut has_storage = false;
    let mut has_compute = false;
    let mut has_cdn = false;

    for proof in proofs {
        match proof.proof_type {
            ServiceProofType::StoragePoST => {
                has_storage = true;
            }
            ServiceProofType::ComputeExecution => {
                has_compute = true;
            }
            ServiceProofType::CdnDelivery => {
                has_cdn = true;
            }
        }
    }

    // Apply bonuses for valid proofs (simplified - using 0.5 quality for valid proofs)
    if has_storage {
        multiplier = multiplier.with_storage(0.5);
    }
    if has_compute {
        multiplier = multiplier.with_compute(0.5);
    }
    if has_cdn {
        multiplier = multiplier.with_cdn(0.5);
    }

    // Apply uptime and geographic bonuses
    multiplier = multiplier.with_uptime(uptime_percent);
    
    if geographic_rarity > 0.0 {
        multiplier = multiplier.with_geographic(geographic_rarity);
    }

    multiplier
}

/// Calculate final reward with multiplier
pub fn apply_multiplier(base_reward: u128, multiplier: &ServiceMultiplier) -> u128 {
    let factor = multiplier.total();
    ((base_reward as f64) * factor) as u128
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_multiplier() {
        let m = ServiceMultiplier::new();
        assert_eq!(m.total(), 1.0);
    }

    #[test]
    fn test_full_multiplier() {
        let m = ServiceMultiplier::new()
            .with_storage(1.0)      // +0.3
            .with_compute(1.0)      // +0.3
            .with_cdn(1.0)          // +0.15
            .with_uptime(99.5)      // +0.1
            .with_geographic(1.0);  // +0.5
        
        // 1.0 + 0.3 + 0.3 + 0.15 + 0.1 + 0.5 = 2.35, capped at 2.0
        assert_eq!(m.total(), MAX_MULTIPLIER);
    }

    #[test]
    fn test_partial_multiplier() {
        let m = ServiceMultiplier::new()
            .with_storage(0.5)  // +0.2
            .with_uptime(99.0); // +0.1
        
        assert!((m.total() - 1.3).abs() < 0.01);
    }

    #[test]
    fn test_apply_multiplier() {
        let base = 1000u128;
        let m = ServiceMultiplier::new().with_storage(0.5); // 1.2x
        
        let result = apply_multiplier(base, &m);
        assert_eq!(result, 1200);
    }
}
