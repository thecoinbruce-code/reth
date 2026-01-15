//! Permia CPU Miner Binary
//!
//! Standalone mining utility for testing PermiaHash.
//!
//! Usage:
//!   permia-mine --difficulty 1000000 --blocks 5

use alloy_primitives::{Address, B256, U256};
use clap::Parser;
use permia_miner::{BlockTemplate, MiningConfig, MiningWorker};
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

/// Permia CPU Miner
#[derive(Debug, Parser)]
#[command(name = "permia-mine")]
#[command(about = "CPU miner for PermiaHash proof-of-work")]
struct Args {
    /// Miner address to receive rewards
    #[arg(long, default_value = "0x0000000000000000000000000000000000000001")]
    miner: String,

    /// Number of mining threads (0 = auto-detect)
    #[arg(long, short = 't', default_value = "0")]
    threads: usize,

    /// Difficulty target
    #[arg(long, short = 'd', default_value = "1000000")]
    difficulty: u64,

    /// Number of blocks to mine (0 = unlimited)
    #[arg(long, short = 'n', default_value = "1")]
    blocks: u64,

    /// Timeout per block in seconds
    #[arg(long, default_value = "300")]
    timeout: u64,
}

fn main() -> eyre::Result<()> {
    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

    let threads = if args.threads == 0 {
        num_cpus::get()
    } else {
        args.threads
    };

    let miner_address: Address = args.miner.parse()
        .unwrap_or(Address::ZERO);

    info!(
        target: "permia::mine",
        miner = %miner_address,
        threads = threads,
        difficulty = args.difficulty,
        blocks = args.blocks,
        "Starting Permia CPU miner"
    );

    let config = MiningConfig {
        threads,
        batch_size: 10_000,
        max_duration: Some(Duration::from_secs(args.timeout)),
    };

    let worker = MiningWorker::new(config);
    let mut blocks_mined = 0u64;
    let mut parent_hash = B256::ZERO;
    let mut block_number = 0u64;
    let mut total_hashes = 0u64;

    let start_time = std::time::Instant::now();

    loop {
        // Check if we've mined enough blocks
        if args.blocks > 0 && blocks_mined >= args.blocks {
            let elapsed = start_time.elapsed();
            info!(
                target: "permia::mine",
                blocks = blocks_mined,
                total_hashes = total_hashes,
                elapsed_secs = elapsed.as_secs(),
                avg_hashrate = format!("{:.2} H/s", total_hashes as f64 / elapsed.as_secs_f64()),
                "Mining complete!"
            );
            break;
        }

        // Create block template
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let template = BlockTemplate::new(
            parent_hash,
            block_number,
            timestamp,
            miner_address,
            U256::from(args.difficulty),
        );

        info!(
            target: "permia::mine",
            block = block_number,
            parent = %parent_hash,
            difficulty = args.difficulty,
            "Mining block..."
        );

        // Mine the block
        worker.reset();
        match worker.mine(&template) {
            Ok(result) => {
                info!(
                    target: "permia::mine",
                    block = block_number,
                    nonce = result.nonce,
                    hash = %result.hash,
                    mix_hash = %result.mix_hash,
                    hashes = result.hashes_computed,
                    hashrate = format!("{:.2} H/s", result.hashrate()),
                    duration_ms = result.duration.as_millis(),
                    "✓ Block mined!"
                );

                // Update for next block
                parent_hash = result.hash;
                block_number += 1;
                blocks_mined += 1;
                total_hashes += result.hashes_computed;
            }
            Err(e) => {
                tracing::error!(target: "permia::mine", error = %e, "Mining failed");
                return Err(e.into());
            }
        }
    }

    Ok(())
}
