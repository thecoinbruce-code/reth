//! Permia Node Binary
//!
//! Entry point for the Permia blockchain node.
//!
//! This binary provides a full Permia node using the Reth infrastructure.

#![allow(missing_docs)]

use clap::Parser;
use permia_cli::PermiaChainSpecParser;
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
            info!(target: "permia::cli", "Launching Permia node");
            
            // For now, use EthereumNode as the base
            // TODO: Create PermiaNode with custom consensus
            let NodeHandle { node, node_exit_future } =
                builder.node(EthereumNode::default()).launch().await?;
            
            info!(
                target: "permia::cli",
                chain_id = %node.chain_spec().chain.id(),
                "Permia node running"
            );
            
            node_exit_future.await
        })
    {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
