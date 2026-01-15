//! Permia Node Binary
//!
//! Entry point for the Permia blockchain node.

#![allow(missing_docs)]

use clap::Parser;
use reth_chainspec::{PERMIA_DEV, PERMIA_MAINNET, PERMIA_TESTNET, ChainSpec};
use permia_node::PermiaConsensusBuilder;
use std::sync::Arc;
use tracing::info;

/// Permia node CLI
#[derive(Parser, Debug)]
#[command(name = "permia", about = "Permia blockchain node", version)]
struct Cli {
    /// Network to connect to
    #[arg(long, short, default_value = "dev")]
    network: String,
    
    /// Data directory
    #[arg(long, default_value = "./data")]
    datadir: String,
    
    /// RPC port
    #[arg(long, default_value = "8545")]
    rpc_port: u16,
    
    /// P2P port
    #[arg(long, default_value = "30303")]
    p2p_port: u16,
    
    /// Enable mining
    #[arg(long)]
    mine: bool,
}

fn main() {
    // Install signal handlers
    reth_cli_util::sigsegv_handler::install();
    
    // Enable backtraces
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    // Select chain spec based on network
    let chain_spec: Arc<ChainSpec> = match cli.network.as_str() {
        "mainnet" | "main" => PERMIA_MAINNET.clone(),
        "testnet" | "test" => PERMIA_TESTNET.clone(),
        "dev" | "devnet" => PERMIA_DEV.clone(),
        _ => {
            eprintln!("Unknown network: {}. Use: mainnet, testnet, or dev", cli.network);
            std::process::exit(1);
        }
    };
    
    info!(
        target: "permia",
        chain_id = chain_spec.chain.id(),
        "Starting Permia node"
    );
    
    // Build consensus
    let consensus = PermiaConsensusBuilder::new().build();
    
    info!(
        target: "permia",
        min_difficulty = %consensus.min_difficulty(),
        "Consensus initialized"
    );
    
    // TODO: Full node integration with Reth
    // For now, just print configuration
    info!(
        target: "permia",
        datadir = %cli.datadir,
        rpc_port = cli.rpc_port,
        p2p_port = cli.p2p_port,
        mining = cli.mine,
        "Node configuration loaded"
    );
    
    info!(target: "permia", "Permia node started successfully!");
    info!(target: "permia", "Full Reth integration coming in next phase...");
}
