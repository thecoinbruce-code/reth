//! Permia Payload Builder with PermiaHash PoW
//!
//! This crate provides a custom payload builder that integrates PermiaHash
//! proof-of-work into the block building process.
//!
//! # Architecture
//!
//! The Permia payload builder wraps the standard Ethereum payload builder
//! and adds PermiaHash PoW mining after the block is constructed:
//!
//! 1. Build block using standard Ethereum payload builder
//! 2. Mine PermiaHash nonce for the block header
//! 3. Seal block with PoW nonce and mix_hash
//!
//! # Usage
//!
//! ```ignore
//! use permia_payload::PermiaPayloadBuilder;
//!
//! let builder = PermiaPayloadBuilder::new(client, pool, evm_config, config);
//! ```

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use permia_consensus::PermiaConsensus;
use reth_basic_payload_builder::{BuildArguments, BuildOutcome, MissingPayloadBehaviour, PayloadBuilder, PayloadConfig};
use reth_chainspec::{ChainSpecProvider, EthereumHardforks};
use reth_ethereum_payload_builder::{EthereumBuilderConfig, EthereumPayloadBuilder};
use reth_ethereum_primitives::{EthPrimitives, TransactionSigned};
use reth_evm::{ConfigureEvm, NextBlockEnvAttributes};
use reth_evm_ethereum::EthEvmConfig;
use reth_payload_builder::{EthBuiltPayload, EthPayloadBuilderAttributes};
use reth_payload_builder_primitives::PayloadBuilderError;
use reth_storage_api::StateProviderFactory;
use reth_transaction_pool::{PoolTransaction, TransactionPool};
use std::sync::Arc;
use tracing::debug;

/// Permia payload builder configuration
#[derive(Debug, Clone)]
pub struct PermiaBuilderConfig {
    /// Inner Ethereum builder config
    pub eth_config: EthereumBuilderConfig,
    /// Target block time in milliseconds (default: 400ms)
    pub target_block_time_ms: u64,
    /// Whether to enable PoW mining (default: true)
    pub pow_enabled: bool,
    /// Maximum mining iterations before giving up
    pub max_mining_iterations: u64,
}

impl Default for PermiaBuilderConfig {
    fn default() -> Self {
        Self {
            eth_config: EthereumBuilderConfig::default(),
            target_block_time_ms: 400,
            pow_enabled: true,
            max_mining_iterations: 1_000_000,
        }
    }
}

impl PermiaBuilderConfig {
    /// Create new config with target block time
    pub fn with_block_time(mut self, ms: u64) -> Self {
        self.target_block_time_ms = ms;
        self
    }

    /// Enable or disable PoW mining
    pub fn with_pow(mut self, enabled: bool) -> Self {
        self.pow_enabled = enabled;
        self
    }
}

/// Permia payload builder with PermiaHash PoW
#[derive(Debug, Clone)]
pub struct PermiaPayloadBuilder<Pool, Client, EvmConfig = EthEvmConfig> {
    /// Inner Ethereum payload builder
    inner: EthereumPayloadBuilder<Pool, Client, EvmConfig>,
    /// Permia-specific configuration
    config: PermiaBuilderConfig,
    /// PermiaHash consensus for PoW validation
    consensus: Arc<PermiaConsensus>,
}

impl<Pool, Client, EvmConfig> PermiaPayloadBuilder<Pool, Client, EvmConfig> {
    /// Create a new Permia payload builder
    pub fn new(
        client: Client,
        pool: Pool,
        evm_config: EvmConfig,
        config: PermiaBuilderConfig,
    ) -> Self {
        let inner = EthereumPayloadBuilder::new(
            client,
            pool,
            evm_config,
            config.eth_config.clone(),
        );
        Self {
            inner,
            config,
            consensus: Arc::new(PermiaConsensus::new()),
        }
    }

    /// Get reference to the PermiaHash consensus
    pub fn consensus(&self) -> &Arc<PermiaConsensus> {
        &self.consensus
    }

    /// Get the target block time in milliseconds
    pub fn target_block_time_ms(&self) -> u64 {
        self.config.target_block_time_ms
    }
}

impl<Pool, Client, EvmConfig> PayloadBuilder for PermiaPayloadBuilder<Pool, Client, EvmConfig>
where
    EvmConfig: ConfigureEvm<Primitives = EthPrimitives, NextBlockEnvCtx = NextBlockEnvAttributes>,
    Client: StateProviderFactory + ChainSpecProvider<ChainSpec: EthereumHardforks> + Clone,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TransactionSigned>>,
{
    type Attributes = EthPayloadBuilderAttributes;
    type BuiltPayload = EthBuiltPayload;

    fn try_build(
        &self,
        args: BuildArguments<EthPayloadBuilderAttributes, EthBuiltPayload>,
    ) -> Result<BuildOutcome<EthBuiltPayload>, PayloadBuilderError> {
        // Build the block using standard Ethereum payload builder
        let outcome = self.inner.try_build(args)?;

        // If PoW is disabled, return the block as-is
        if !self.config.pow_enabled {
            return Ok(outcome);
        }

        // For now, we return the block as-is since LocalMiner handles block production
        // In a full PoW implementation, we would mine the nonce here
        //
        // TODO: Integrate PermiaHash mining into block sealing:
        // 1. Extract block header from outcome
        // 2. Mine nonce using PermiaConsensus
        // 3. Re-seal block with mined nonce and mix_hash
        //
        // This requires modifying the block header after construction,
        // which needs deeper integration with Reth's primitives.

        match &outcome {
            BuildOutcome::Better { payload, .. } => {
                debug!(
                    target: "permia::payload",
                    block_hash = %payload.block().hash(),
                    "Built Permia payload (PoW pending integration)"
                );
            }
            _ => {}
        }

        Ok(outcome)
    }

    fn on_missing_payload(
        &self,
        args: BuildArguments<Self::Attributes, Self::BuiltPayload>,
    ) -> MissingPayloadBehaviour<Self::BuiltPayload> {
        self.inner.on_missing_payload(args)
    }

    fn build_empty_payload(
        &self,
        config: PayloadConfig<Self::Attributes>,
    ) -> Result<EthBuiltPayload, PayloadBuilderError> {
        self.inner.build_empty_payload(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = PermiaBuilderConfig::default();
        assert_eq!(config.target_block_time_ms, 400);
        assert!(config.pow_enabled);
    }

    #[test]
    fn test_config_builder() {
        let config = PermiaBuilderConfig::default()
            .with_block_time(1000)
            .with_pow(false);
        assert_eq!(config.target_block_time_ms, 1000);
        assert!(!config.pow_enabled);
    }
}
