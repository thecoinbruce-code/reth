//! Genesis configuration types

use alloy_primitives::{Address, U256};
use serde::{Deserialize, Serialize};

use crate::constants;

/// Network type for genesis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkType {
    /// Production mainnet
    Mainnet,
    /// Public testnet
    Testnet,
    /// Local development
    Devnet,
}

impl NetworkType {
    /// Get the chain ID for this network
    pub fn chain_id(&self) -> u64 {
        match self {
            NetworkType::Mainnet => constants::MAINNET_CHAIN_ID,
            NetworkType::Testnet => constants::TESTNET_CHAIN_ID,
            NetworkType::Devnet => constants::DEVNET_CHAIN_ID,
        }
    }

    /// Get the initial difficulty for this network
    pub fn initial_difficulty(&self) -> u64 {
        match self {
            NetworkType::Mainnet => constants::INITIAL_DIFFICULTY,
            NetworkType::Testnet => constants::INITIAL_DIFFICULTY / 10, // Easier for testnet
            NetworkType::Devnet => 100_000, // Very easy for local dev
        }
    }
}

impl Default for NetworkType {
    fn default() -> Self {
        NetworkType::Devnet
    }
}

/// An account allocation in genesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    /// Account address
    pub address: Address,
    /// Initial balance in wei
    pub balance: U256,
    /// Vesting period in blocks (0 = no vesting)
    pub vesting_blocks: u64,
    /// Description/purpose
    pub description: String,
}

impl Allocation {
    /// Create a new allocation
    pub fn new(address: Address, balance: U256, description: impl Into<String>) -> Self {
        Self {
            address,
            balance,
            vesting_blocks: 0,
            description: description.into(),
        }
    }

    /// Add vesting period
    pub fn with_vesting(mut self, blocks: u64) -> Self {
        self.vesting_blocks = blocks;
        self
    }
}

/// Genesis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Network type
    pub network: NetworkType,
    /// Genesis timestamp
    pub timestamp: u64,
    /// Extra data in genesis block
    pub extra_data: Vec<u8>,
    /// Account allocations
    pub allocations: Vec<Allocation>,
    /// Foundation address
    pub foundation_address: Option<Address>,
    /// Team multisig address
    pub team_address: Option<Address>,
    /// Community grants address
    pub community_address: Option<Address>,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self {
            network: NetworkType::Devnet,
            timestamp: 0,
            extra_data: b"Permia Genesis".to_vec(),
            allocations: Vec::new(),
            foundation_address: None,
            team_address: None,
            community_address: None,
        }
    }
}

impl GenesisConfig {
    /// Create a new config for the specified network
    pub fn new(network: NetworkType) -> Self {
        Self {
            network,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ..Default::default()
        }
    }

    /// Create a devnet config (no allocations, easy mining)
    pub fn devnet() -> Self {
        Self::new(NetworkType::Devnet)
    }

    /// Create a testnet config with standard allocations
    pub fn testnet(foundation: Address, team: Address, community: Address) -> Self {
        let mut config = Self::new(NetworkType::Testnet);
        config.foundation_address = Some(foundation);
        config.team_address = Some(team);
        config.community_address = Some(community);
        config.add_standard_allocations();
        config
    }

    /// Create a mainnet config with standard allocations
    pub fn mainnet(foundation: Address, team: Address, community: Address) -> Self {
        let mut config = Self::new(NetworkType::Mainnet);
        config.foundation_address = Some(foundation);
        config.team_address = Some(team);
        config.community_address = Some(community);
        config.add_standard_allocations();
        config
    }

    /// Add standard allocations (foundation, team, community)
    pub fn add_standard_allocations(&mut self) {
        if let Some(addr) = self.foundation_address {
            self.allocations.push(
                Allocation::new(addr, constants::foundation_allocation(), "Foundation (10%)")
                    .with_vesting(constants::BLOCKS_PER_YEAR), // 1 year vest
            );
        }

        if let Some(addr) = self.team_address {
            self.allocations.push(
                Allocation::new(addr, constants::team_allocation(), "Team (5%)")
                    .with_vesting(constants::BLOCKS_PER_YEAR * 4), // 4 year vest
            );
        }

        if let Some(addr) = self.community_address {
            self.allocations.push(
                Allocation::new(addr, constants::community_allocation(), "Community (5%)")
                    .with_vesting(constants::BLOCKS_PER_YEAR / 2), // 6 month vest
            );
        }
    }

    /// Get chain ID
    pub fn chain_id(&self) -> u64 {
        self.network.chain_id()
    }

    /// Get initial difficulty
    pub fn initial_difficulty(&self) -> u64 {
        self.network.initial_difficulty()
    }

    /// Calculate total allocated
    pub fn total_allocated(&self) -> U256 {
        self.allocations.iter().fold(U256::ZERO, |acc, a| acc + a.balance)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), crate::GenesisError> {
        // Check for duplicate addresses
        let mut seen = std::collections::HashSet::new();
        for alloc in &self.allocations {
            if !seen.insert(alloc.address) {
                return Err(crate::GenesisError::InvalidConfig(
                    format!("Duplicate allocation address: {}", alloc.address)
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_types() {
        assert_eq!(NetworkType::Mainnet.chain_id(), 42069);
        assert_eq!(NetworkType::Testnet.chain_id(), 42070);
        assert_eq!(NetworkType::Devnet.chain_id(), 42071);
    }

    #[test]
    fn test_devnet_config() {
        let config = GenesisConfig::devnet();
        assert_eq!(config.network, NetworkType::Devnet);
        assert!(config.allocations.is_empty());
    }

    #[test]
    fn test_mainnet_config() {
        let foundation = Address::repeat_byte(1);
        let team = Address::repeat_byte(2);
        let community = Address::repeat_byte(3);

        let config = GenesisConfig::mainnet(foundation, team, community);
        
        assert_eq!(config.network, NetworkType::Mainnet);
        assert_eq!(config.allocations.len(), 3);
        assert!(config.total_allocated() > U256::ZERO);
    }

    #[test]
    fn test_validation() {
        let mut config = GenesisConfig::devnet();
        
        // Add same address twice
        let addr = Address::repeat_byte(1);
        config.allocations.push(Allocation::new(addr, U256::from(100), "Test 1"));
        config.allocations.push(Allocation::new(addr, U256::from(200), "Test 2"));
        
        assert!(config.validate().is_err());
    }
}
