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
//!
//! # P2P Block Validation
//!
//! Incoming blocks from peers are validated using PermiaHash PoW before import.

#![allow(missing_docs)]

use clap::Parser;
use permia_cli::PermiaChainSpecParser;
use permia_gossip::spawn_block_announcer;
use permia_node::{PermiaConsensusBuilder, PermiaNetworkBuilder};
use reth_ethereum_cli::Cli;
use reth_node_builder::Node;
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
            
            // Use EthereumNode as base with Permia's custom network builder
            // - PermiaNetworkBuilder validates incoming P2P blocks with PermiaHash PoW
            // - LocalMiner is enabled in dev mode (--dev flag)
            // - Blocks are submitted via Engine API
            let handle = builder
                .with_types::<EthereumNode>()
                .with_components(
                    EthereumNode::components()
                        .network(PermiaNetworkBuilder::default())
                )
                .with_add_ons(EthereumNode::default().add_ons())
                .launch_with_debug_capabilities()
                .await?;
            
            info!(
                target: "permia::cli",
                chain_id = %handle.node.chain_spec().chain.id(),
                "Permia node running with PermiaHash P2P validation"
            );
            
            // Spawn block announcer to broadcast mined blocks to peers
            let network = handle.node.network.clone();
            let provider = handle.node.provider.clone();
            handle.node.task_executor.spawn_critical("permia-block-announcer", Box::pin(async move {
                info!(target: "permia::cli", "Starting block announcer for P2P propagation");
                spawn_block_announcer(network, provider).await;
            }));
            
            handle.wait_for_node_exit().await
        })
    {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
