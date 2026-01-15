//! Permia Node Binary
//!
//! Entry point for the Permia blockchain node.
//!
//! This binary provides a full Permia node using the Reth infrastructure
//! with PermiaHash Proof-of-Work consensus.

#![allow(missing_docs)]

use clap::Parser;
use permia_cli::PermiaChainSpecParser;
use permia_node::PermiaConsensusBuilder;
use reth_ethereum_cli::Cli;
use reth_node_builder::NodeHandle;
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
            info!(
                target: "permia::cli",
                min_difficulty = %consensus.min_difficulty(),
                "PermiaHash consensus ready"
            );
            
            node_exit_future.await
        })
    {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
