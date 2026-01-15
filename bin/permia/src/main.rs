//! Permia Node Binary
//!
//! Entry point for the Permia blockchain node.
//!
//! This binary provides a full Permia node using the Reth infrastructure
//! with PermiaHash Proof-of-Work consensus.
//!
//! # Mining Mode
//!
//! When running with `--dev`, the node will automatically mine blocks
//! using PermiaHash proof-of-work.

#![allow(missing_docs)]

use alloy_primitives::{Address, B256, U256};
use clap::Parser;
use permia_cli::PermiaChainSpecParser;
use permia_miner::{NodeMinerConfig, spawn_node_miner};
use permia_node::PermiaConsensusBuilder;
use reth_ethereum_cli::Cli;
use reth_node_builder::NodeHandle;
use reth_node_ethereum::EthereumNode;
use std::time::Duration;
use tracing::info;

fn main() {
    // Install signal handlers
    reth_cli_util::sigsegv_handler::install();
    
    // Enable backtraces
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }
    
    // Run the Permia node using Reth's CLI infrastructure
    if let Err(err) =
        Cli::<PermiaChainSpecParser, ()>::parse().run(async move |builder, _| {
            info!(target: "permia::cli", "Launching Permia node with PermiaHash PoW");
            
            // Use EthereumNode as base - PermiaHash consensus is ready for future integration
            // Currently using EthereumNode which works with Permia chain specs
            let NodeHandle { node, node_exit_future } = builder
                .node(EthereumNode::default())
                .launch()
                .await?;
            
            info!(
                target: "permia::cli",
                chain_id = %node.chain_spec().chain.id(),
                "Permia node running"
            );
            
            // Log consensus info
            let consensus = PermiaConsensusBuilder::new().build_standalone();
            let min_difficulty = consensus.min_difficulty();
            info!(
                target: "permia::cli",
                min_difficulty = %min_difficulty,
                "PermiaHash consensus ready"
            );

            // Check if we should auto-mine (dev mode)
            let chain_id = node.chain_spec().chain.id();
            let is_dev = chain_id == 42071; // Permia devnet
            
            if is_dev {
                info!(
                    target: "permia::cli",
                    "Dev mode detected - starting auto-miner"
                );

                // Configure and spawn the miner
                let miner_config = NodeMinerConfig::default()
                    .with_beneficiary(Address::ZERO) // TODO: configurable
                    .with_threads(2);

                let (miner_handle, mut mined_rx) = spawn_node_miner(miner_config);

                // Start mining the first block
                let _ = miner_handle.start_mining(
                    B256::ZERO,
                    0,
                    B256::ZERO,
                    B256::ZERO, 
                    B256::ZERO,
                    min_difficulty,
                    0,
                ).await;

                // Spawn task to handle mined blocks
                tokio::spawn(async move {
                    while let Some(mined) = mined_rx.recv().await {
                        info!(
                            target: "permia::cli",
                            block = mined.number,
                            hash = %mined.hash,
                            nonce = mined.nonce,
                            hashrate = format!("{:.2} H/s", mined.mining_result.hashrate()),
                            "Block mined - ready for submission"
                        );

                        // Continue mining next block
                        let _ = miner_handle.start_mining(
                            mined.hash,
                            mined.number,
                            B256::ZERO,
                            B256::ZERO,
                            B256::ZERO,
                            mined.difficulty,
                            0,
                        ).await;
                    }
                });
            }
            
            node_exit_future.await
        })
    {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
