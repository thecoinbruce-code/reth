//! Validator set management for Permia BFT
//!
//! Validators are the top 100 miners by stake + service score.

use alloy_primitives::{Address, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A validator in the active set
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    /// Validator address
    pub address: Address,
    /// Staked amount in wei
    pub stake: U256,
    /// Service score (from service proofs)
    pub service_score: u64,
    /// Combined weight for selection
    pub weight: U256,
    /// Whether currently active
    pub active: bool,
}

impl Validator {
    /// Create a new validator
    pub fn new(address: Address, stake: U256, service_score: u64) -> Self {
        // Weight = stake + (service_score * 1e18)
        let service_weight = U256::from(service_score) * U256::from(1_000_000_000_000_000_000u64);
        let weight = stake.saturating_add(service_weight);
        
        Self {
            address,
            stake,
            service_score,
            weight,
            active: true,
        }
    }

    /// Check if validator meets minimum stake requirement
    pub fn meets_minimum_stake(&self) -> bool {
        self.stake >= U256::from(crate::config::MIN_STAKE)
    }
    
    /// Get minimum stake as U256
    pub fn min_stake() -> U256 {
        U256::from(crate::config::MIN_STAKE)
    }
}

/// The active validator set
#[derive(Debug, Clone, Default)]
pub struct ValidatorSet {
    /// Validators indexed by address
    validators: HashMap<Address, Validator>,
    /// Ordered list of validator addresses by weight
    ordered: Vec<Address>,
    /// Current epoch
    pub epoch: u64,
    /// Block number when this set became active
    pub active_from_block: u64,
}

impl ValidatorSet {
    /// Create a new empty validator set
    pub fn new(epoch: u64, active_from_block: u64) -> Self {
        Self {
            validators: HashMap::new(),
            ordered: Vec::new(),
            epoch,
            active_from_block,
        }
    }

    /// Create a validator set from a list of validators
    pub fn from_validators(validators: Vec<Validator>, epoch: u64, active_from_block: u64) -> Self {
        let mut set = Self::new(epoch, active_from_block);
        
        for validator in validators {
            set.validators.insert(validator.address, validator);
        }
        
        set.reorder();
        set
    }

    /// Add or update a validator
    pub fn upsert(&mut self, validator: Validator) {
        self.validators.insert(validator.address, validator);
        self.reorder();
    }

    /// Remove a validator
    pub fn remove(&mut self, address: &Address) {
        self.validators.remove(address);
        self.reorder();
    }

    /// Reorder validators by weight
    fn reorder(&mut self) {
        let mut validators: Vec<_> = self.validators.values().collect();
        validators.sort_by(|a, b| b.weight.cmp(&a.weight));
        
        // Keep only top N validators
        self.ordered = validators
            .into_iter()
            .take(crate::config::VALIDATOR_SET_SIZE)
            .map(|v| v.address)
            .collect();
    }

    /// Check if an address is an active validator
    pub fn is_validator(&self, address: &Address) -> bool {
        self.ordered.contains(address)
    }

    /// Get a validator by address
    pub fn get(&self, address: &Address) -> Option<&Validator> {
        if self.is_validator(address) {
            self.validators.get(address)
        } else {
            None
        }
    }

    /// Get all active validators
    pub fn active_validators(&self) -> Vec<&Validator> {
        self.ordered
            .iter()
            .filter_map(|addr| self.validators.get(addr))
            .collect()
    }

    /// Get the number of active validators
    pub fn len(&self) -> usize {
        self.ordered.len()
    }

    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.ordered.is_empty()
    }

    /// Get the finality threshold (2/3 + 1)
    pub fn finality_threshold(&self) -> usize {
        (self.len() * 2 / 3) + 1
    }

    /// Get total stake of active validators
    pub fn total_stake(&self) -> U256 {
        self.active_validators()
            .iter()
            .fold(U256::ZERO, |acc, v| acc.saturating_add(v.stake))
    }
}

/// Update to the validator set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSetUpdate {
    /// New epoch number
    pub epoch: u64,
    /// Block number this update applies from
    pub from_block: u64,
    /// Validators to add or update
    pub additions: Vec<Validator>,
    /// Validators to remove
    pub removals: Vec<Address>,
}

impl ValidatorSetUpdate {
    /// Apply this update to a validator set
    pub fn apply(&self, set: &mut ValidatorSet) {
        set.epoch = self.epoch;
        set.active_from_block = self.from_block;
        
        for validator in &self.additions {
            set.upsert(validator.clone());
        }
        
        for address in &self.removals {
            set.remove(address);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let addr = Address::ZERO;
        let stake = U256::from(10_000_000_000_000_000_000_000u128); // 10,000 MIA
        let validator = Validator::new(addr, stake, 100);
        
        assert!(validator.meets_minimum_stake());
        assert!(validator.weight > stake); // Service score adds weight
    }

    #[test]
    fn test_validator_set() {
        let validators = vec![
            Validator::new(Address::repeat_byte(1), U256::from(100u64), 10),
            Validator::new(Address::repeat_byte(2), U256::from(200u64), 20),
            Validator::new(Address::repeat_byte(3), U256::from(150u64), 15),
        ];
        
        let set = ValidatorSet::from_validators(validators, 1, 0);
        
        assert_eq!(set.len(), 3);
        assert!(set.is_validator(&Address::repeat_byte(2)));
        
        // Highest weight should be first
        let active = set.active_validators();
        assert_eq!(active[0].address, Address::repeat_byte(2));
    }

    #[test]
    fn test_finality_threshold() {
        let mut validators = Vec::new();
        for i in 0..100u8 {
            validators.push(Validator::new(
                Address::repeat_byte(i),
                U256::from(100u64),
                10,
            ));
        }
        
        let set = ValidatorSet::from_validators(validators, 1, 0);
        assert_eq!(set.finality_threshold(), 67); // 2/3 + 1
    }
}
