//! Mining worker implementation
//!
//! Handles parallel nonce search using PermiaHash.

use crate::{BlockTemplate, MiningError};
use alloy_primitives::{B256, U256, FixedBytes};
use permia_consensus::pow::{permia_hash_with_epoch, HashResult};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Mining configuration
#[derive(Debug, Clone)]
pub struct MiningConfig {
    /// Number of mining threads
    pub threads: usize,
    /// Nonces to try per batch before checking for cancellation
    pub batch_size: u64,
    /// Maximum time to mine before giving up (None = forever)
    pub max_duration: Option<Duration>,
}

impl Default for MiningConfig {
    fn default() -> Self {
        Self {
            threads: num_cpus::get().max(1),
            batch_size: 10_000,
            max_duration: None,
        }
    }
}

impl MiningConfig {
    /// Create config for single-threaded mining
    pub fn single_thread() -> Self {
        Self {
            threads: 1,
            ..Default::default()
        }
    }

    /// Create config with specific thread count
    pub fn with_threads(threads: usize) -> Self {
        Self {
            threads: threads.max(1),
            ..Default::default()
        }
    }
}

/// Result of successful mining
#[derive(Debug, Clone)]
pub struct MiningResult {
    /// The winning nonce
    pub nonce: u64,
    /// The mix hash
    pub mix_hash: B256,
    /// The final hash (must be < target)
    pub hash: B256,
    /// Number of hashes computed
    pub hashes_computed: u64,
    /// Time taken to find solution
    pub duration: Duration,
}

impl MiningResult {
    /// Get hashrate in H/s
    pub fn hashrate(&self) -> f64 {
        self.hashes_computed as f64 / self.duration.as_secs_f64()
    }
}

/// Mining worker that searches for valid nonces
pub struct MiningWorker {
    config: MiningConfig,
    cancelled: Arc<AtomicBool>,
    total_hashes: Arc<AtomicU64>,
}

impl MiningWorker {
    /// Create a new mining worker
    pub fn new(config: MiningConfig) -> Self {
        Self {
            config,
            cancelled: Arc::new(AtomicBool::new(false)),
            total_hashes: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Cancel ongoing mining
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Reset cancellation flag
    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
        self.total_hashes.store(0, Ordering::SeqCst);
    }

    /// Get current hash count
    pub fn hash_count(&self) -> u64 {
        self.total_hashes.load(Ordering::Relaxed)
    }

    /// Mine a block template (blocking, single-threaded for simplicity)
    pub fn mine(&self, template: &BlockTemplate) -> Result<MiningResult, MiningError> {
        let start = Instant::now();
        let seal_hash = template.seal_hash();
        let target = template.target();
        let block_number = template.number;

        info!(
            target: "permia::miner",
            block = block_number,
            difficulty = %template.difficulty,
            "Starting mining"
        );

        let mut nonce: u64 = rand::random();
        let start_nonce = nonce;

        loop {
            // Check cancellation
            if self.cancelled.load(Ordering::Relaxed) {
                return Err(MiningError::Cancelled);
            }

            // Check timeout
            if let Some(max_dur) = self.config.max_duration {
                if start.elapsed() > max_dur {
                    return Err(MiningError::NoSolution {
                        start: start_nonce,
                        end: nonce,
                    });
                }
            }

            // Try batch of nonces
            for _ in 0..self.config.batch_size {
                let result = permia_hash_with_epoch(&seal_hash, nonce, block_number);
                self.total_hashes.fetch_add(1, Ordering::Relaxed);

                let hash_value = U256::from_be_bytes(result.hash.0);

                if hash_value <= target {
                    let duration = start.elapsed();
                    let hashes = self.total_hashes.load(Ordering::Relaxed);

                    info!(
                        target: "permia::miner",
                        block = block_number,
                        nonce = nonce,
                        hashes = hashes,
                        duration_ms = duration.as_millis(),
                        hashrate = hashes as f64 / duration.as_secs_f64(),
                        "Block mined!"
                    );

                    return Ok(MiningResult {
                        nonce,
                        mix_hash: result.mix_digest,
                        hash: result.hash,
                        hashes_computed: hashes,
                        duration,
                    });
                }

                nonce = nonce.wrapping_add(1);
            }

            // Log progress periodically
            let hashes = self.total_hashes.load(Ordering::Relaxed);
            if hashes % 100_000 == 0 {
                let elapsed = start.elapsed();
                let hashrate = hashes as f64 / elapsed.as_secs_f64();
                debug!(
                    target: "permia::miner",
                    hashes = hashes,
                    hashrate = format!("{:.2} H/s", hashrate),
                    "Mining in progress"
                );
            }
        }
    }

    /// Mine with async support
    pub async fn mine_async(&self, template: BlockTemplate) -> Result<MiningResult, MiningError> {
        let worker = self.clone_internals();
        tokio::task::spawn_blocking(move || {
            let miner = MiningWorker {
                config: worker.0,
                cancelled: worker.1,
                total_hashes: worker.2,
            };
            miner.mine(&template)
        })
        .await
        .map_err(|_| MiningError::Cancelled)?
    }

    fn clone_internals(&self) -> (MiningConfig, Arc<AtomicBool>, Arc<AtomicU64>) {
        (
            self.config.clone(),
            Arc::clone(&self.cancelled),
            Arc::clone(&self.total_hashes),
        )
    }
}

/// Search a nonce range for a valid solution
pub fn search_nonce_range(
    seal_hash: &B256,
    block_number: u64,
    target: U256,
    start: u64,
    end: u64,
) -> Option<(u64, HashResult)> {
    for nonce in start..end {
        let result = permia_hash_with_epoch(seal_hash, nonce, block_number);
        let hash_value = U256::from_be_bytes(result.hash.0);

        if hash_value <= target {
            return Some((nonce, result));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::Address;

    #[test]
    fn test_mining_config() {
        let config = MiningConfig::default();
        assert!(config.threads >= 1);
        assert_eq!(config.batch_size, 10_000);
    }

    #[test]
    fn test_mine_easy_difficulty() {
        // Use very low difficulty so we find a solution quickly
        let template = BlockTemplate::new(
            B256::ZERO,
            1,
            1000,
            Address::ZERO,
            U256::from(1u64), // Minimum difficulty = easy to find
        );

        let config = MiningConfig {
            threads: 1,
            batch_size: 1000,
            max_duration: Some(Duration::from_secs(10)),
        };

        let worker = MiningWorker::new(config);
        let result = worker.mine(&template);

        assert!(result.is_ok(), "Should find solution with low difficulty");
        let mining_result = result.unwrap();
        assert!(mining_result.hashes_computed > 0);
        println!(
            "Found solution: nonce={}, hashrate={:.2} H/s",
            mining_result.nonce,
            mining_result.hashrate()
        );
    }
}
