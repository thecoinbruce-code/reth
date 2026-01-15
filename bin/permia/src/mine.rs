//! Mining command for Permia
//!
//! Provides CPU mining functionality for the Permia network.

use alloy_primitives::{Address, B256, U256};
use clap::Parser;
use permia_miner::{BlockTemplate, MiningConfig, MiningWorker};
use std::time::Duration;
use tracing::info;

/// Mining command arguments
#[derive(Debug, Parser)]
pub struct MineArgs {
    /// Miner address to receive rewards
    #[arg(long, default_value = "0x0000000000000000000000000000000000000001")]
    pub miner: Address,

    /// Number of mining threads (0 = auto-detect)
    #[arg(long, default_value = "0")]
    pub threads: usize,

    /// Difficulty (for testing, 0 = use chain difficulty)
    #[arg(long, default_value = "1000000")]
    pub difficulty: u64,

    /// Maximum blocks to mine (0 = unlimited)
    #[arg(long, default_value = "1")]
    pub blocks: u64,
}

impl MineArgs {
    /// Run the miner
    pub fn run(&self) -> eyre::Result<()> {
        let threads = if self.threads == 0 {
            num_cpus::get()
        } else {
            self.threads
        };

        info!(
            target: "permia::mine",
            miner = %self.miner,
            threads = threads,
            difficulty = self.difficulty,
            "Starting Permia CPU miner"
        );

        let config = MiningConfig {
            threads,
            batch_size: 10_000,
            max_duration: Some(Duration::from_secs(60)),
        };

        let worker = MiningWorker::new(config);
        let mut blocks_mined = 0u64;
        let mut parent_hash = B256::ZERO;
        let mut block_number = 0u64;

        loop {
            // Check if we've mined enough blocks
            if self.blocks > 0 && blocks_mined >= self.blocks {
                info!(
                    target: "permia::mine",
                    blocks = blocks_mined,
                    "Mining complete"
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
                self.miner,
                U256::from(self.difficulty),
            );

            info!(
                target: "permia::mine",
                block = block_number,
                difficulty = self.difficulty,
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
                        hashrate = format!("{:.2} H/s", result.hashrate()),
                        duration_ms = result.duration.as_millis(),
                        "Block mined!"
                    );

                    // Update for next block
                    parent_hash = result.hash;
                    block_number += 1;
                    blocks_mined += 1;
                }
                Err(e) => {
                    tracing::error!(target: "permia::mine", error = %e, "Mining failed");
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }
}

/// Run a quick mining demo
pub fn demo_mine() -> eyre::Result<()> {
    let args = MineArgs {
        miner: Address::ZERO,
        threads: 1,
        difficulty: 100, // Very easy for demo
        blocks: 3,
    };
    args.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mine_demo() {
        // Just test that the mining code compiles and can be called
        let args = MineArgs {
            miner: Address::ZERO,
            threads: 1,
            difficulty: 1, // Minimum difficulty
            blocks: 1,
        };
        
        // This should complete quickly with difficulty=1
        let result = args.run();
        assert!(result.is_ok());
    }
}
