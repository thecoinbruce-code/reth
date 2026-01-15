//! Genesis block builder

use alloy_genesis::{Genesis, GenesisAccount};
use alloy_primitives::{Address, B256, Bytes, U256};
use std::collections::BTreeMap;
use std::path::Path;

use crate::{GenesisConfig, GenesisError, constants};

/// Builder for creating Permia genesis blocks
#[derive(Debug)]
pub struct GenesisBuilder {
    config: GenesisConfig,
}

impl GenesisBuilder {
    /// Create a new genesis builder
    pub fn new(config: GenesisConfig) -> Self {
        Self { config }
    }

    /// Create a devnet genesis builder
    pub fn devnet() -> Self {
        Self::new(GenesisConfig::devnet())
    }

    /// Create a testnet genesis builder
    pub fn testnet(foundation: Address, team: Address, community: Address) -> Self {
        Self::new(GenesisConfig::testnet(foundation, team, community))
    }

    /// Create a mainnet genesis builder
    pub fn mainnet(foundation: Address, team: Address, community: Address) -> Self {
        Self::new(GenesisConfig::mainnet(foundation, team, community))
    }

    /// Build the genesis configuration
    pub fn build(&self) -> Result<Genesis, GenesisError> {
        self.config.validate()?;

        // Build alloc map
        let mut alloc: BTreeMap<Address, GenesisAccount> = BTreeMap::new();
        
        for allocation in &self.config.allocations {
            alloc.insert(
                allocation.address,
                GenesisAccount {
                    balance: allocation.balance,
                    nonce: None,
                    code: None,
                    storage: None,
                    private_key: None,
                },
            );
        }

        // Build genesis
        let genesis = Genesis {
            config: self.build_chain_config(),
            nonce: 0,
            timestamp: self.config.timestamp,
            extra_data: Bytes::from(self.config.extra_data.clone()),
            gas_limit: 30_000_000, // 30M gas limit
            difficulty: U256::from(self.config.initial_difficulty()),
            mix_hash: B256::ZERO,
            coinbase: Address::ZERO,
            alloc,
            base_fee_per_gas: Some(1_000_000_000), // 1 gwei
            excess_blob_gas: Some(0),
            blob_gas_used: Some(0),
            number: Some(0),
            ..Default::default()
        };

        Ok(genesis)
    }

    /// Build the chain configuration
    fn build_chain_config(&self) -> alloy_genesis::ChainConfig {
        alloy_genesis::ChainConfig {
            chain_id: self.config.chain_id(),
            homestead_block: Some(0),
            eip150_block: Some(0),
            eip155_block: Some(0),
            eip158_block: Some(0),
            byzantium_block: Some(0),
            constantinople_block: Some(0),
            petersburg_block: Some(0),
            istanbul_block: Some(0),
            berlin_block: Some(0),
            london_block: Some(0),
            shanghai_time: Some(0),
            cancun_time: Some(0),
            prague_time: Some(0),
            terminal_total_difficulty: Some(U256::ZERO),
            terminal_total_difficulty_passed: true,
            ..Default::default()
        }
    }

    /// Write genesis to a JSON file
    pub fn write_json(&self, path: impl AsRef<Path>) -> Result<(), GenesisError> {
        let genesis = self.build()?;
        let json = serde_json::to_string_pretty(&genesis)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Get genesis as JSON string
    pub fn to_json(&self) -> Result<String, GenesisError> {
        let genesis = self.build()?;
        Ok(serde_json::to_string_pretty(&genesis)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devnet_genesis() {
        let builder = GenesisBuilder::devnet();
        let genesis = builder.build().unwrap();
        
        assert_eq!(genesis.config.chain_id, constants::DEVNET_CHAIN_ID);
        assert!(genesis.alloc.is_empty());
    }

    #[test]
    fn test_mainnet_genesis() {
        let builder = GenesisBuilder::mainnet(
            Address::repeat_byte(1),
            Address::repeat_byte(2),
            Address::repeat_byte(3),
        );
        
        let genesis = builder.build().unwrap();
        
        assert_eq!(genesis.config.chain_id, constants::MAINNET_CHAIN_ID);
        assert_eq!(genesis.alloc.len(), 3);
    }

    #[test]
    fn test_genesis_json() {
        let builder = GenesisBuilder::devnet();
        let json = builder.to_json().unwrap();
        
        assert!(json.contains("42071")); // Devnet chain ID
        assert!(json.contains("chainId"));
    }

    #[test]
    fn test_write_genesis_file() {
        let builder = GenesisBuilder::devnet();
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("permia_test_genesis.json");
        
        builder.write_json(&path).unwrap();
        
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("42071"));
        
        // Cleanup
        std::fs::remove_file(&path).ok();
    }
}
