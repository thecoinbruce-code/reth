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
//! at regular intervals using Reth's LocalMiner infrastructure.

#![allow(missing_docs)]

use clap::Parser;
use permia_cli::PermiaChainSpecParser;
use permia_node::PermiaConsensusBuilder;
use reth_ethereum_cli::Cli;
use reth_node_ethereum::EthereumNode;
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
            
            // Log consensus info
            let consensus = PermiaConsensusBuilder::new().build_standalone();
            let min_difficulty = consensus.min_difficulty();
            info!(
                target: "permia::cli",
                min_difficulty = %min_difficulty,
                "PermiaHash consensus initialized"
            );
            
            // Use EthereumNode as base with debug capabilities for dev mode
            // This enables LocalMiner when --dev flag is passed, which:
            // - Builds payloads via PayloadBuilder
            // - Submits blocks via Engine API (newPayload + forkchoiceUpdated)
            // - Persists blocks to the chain database
            let handle = builder
                .node(EthereumNode::default())
                .launch_with_debug_capabilities()
                .await?;
            
            info!(
                target: "permia::cli",
                chain_id = %handle.node.chain_spec().chain.id(),
                "Permia node running"
            );
            
            handle.wait_for_node_exit().await
        })
    {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
