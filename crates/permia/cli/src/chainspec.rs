//! Permia chain specification parser

use reth_chainspec::{
    ChainSpec, PERMIA_DEV, PERMIA_MAINNET, PERMIA_TESTNET,
};
use reth_cli::chainspec::{parse_genesis, ChainSpecParser};
use std::sync::Arc;

/// Chains supported by Permia
pub const SUPPORTED_CHAINS: &[&str] = &[
    "permia",
    "permia-mainnet", 
    "permia-testnet",
    "permia-dev",
    "mainnet",
    "testnet", 
    "dev",
];

/// Parse a chain specification string into a ChainSpec
pub fn chain_value_parser(s: &str) -> eyre::Result<Arc<ChainSpec>, eyre::Error> {
    Ok(match s.to_lowercase().as_str() {
        "permia" | "permia-mainnet" | "mainnet" => PERMIA_MAINNET.clone(),
        "permia-testnet" | "testnet" => PERMIA_TESTNET.clone(),
        "permia-dev" | "dev" => PERMIA_DEV.clone(),
        _ => Arc::new(parse_genesis(s)?.into()),
    })
}

/// Permia chain specification parser
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct PermiaChainSpecParser;

impl ChainSpecParser for PermiaChainSpecParser {
    type ChainSpec = ChainSpec;

    const SUPPORTED_CHAINS: &'static [&'static str] = SUPPORTED_CHAINS;

    fn parse(s: &str) -> eyre::Result<Arc<ChainSpec>> {
        chain_value_parser(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_known_chain_spec() {
        for &chain in PermiaChainSpecParser::SUPPORTED_CHAINS {
            let result = <PermiaChainSpecParser as ChainSpecParser>::parse(chain);
            assert!(result.is_ok(), "Failed to parse chain: {}", chain);
        }
    }

    #[test]
    fn parse_permia_mainnet() {
        let spec = <PermiaChainSpecParser as ChainSpecParser>::parse("permia-mainnet").unwrap();
        assert_eq!(spec.chain.id(), 42069);
    }

    #[test]
    fn parse_permia_dev() {
        let spec = <PermiaChainSpecParser as ChainSpecParser>::parse("dev").unwrap();
        assert_eq!(spec.chain.id(), 42071);
    }
}
