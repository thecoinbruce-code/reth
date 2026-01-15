//! Node-integrated miner for Permia
//!
//! This module provides a miner that integrates with the Reth node,
//! automatically mining blocks when the node is running.

use crate::{BlockTemplate, MiningConfig, MiningError, MiningResult, MiningWorker};
use alloy_primitives::{Address, B256, U256};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Configuration for the node-integrated miner
#[derive(Debug, Clone)]
pub struct NodeMinerConfig {
    /// Miner address to receive block rewards
    pub beneficiary: Address,
    /// Mining threads
    pub threads: usize,
    /// Target block time in milliseconds
    pub target_block_time_ms: u64,
    /// Whether to mine empty blocks
    pub mine_empty_blocks: bool,
    /// Maximum time to spend mining a single block
    pub max_mining_time: Duration,
}

impl Default for NodeMinerConfig {
    fn default() -> Self {
        Self {
            beneficiary: Address::ZERO,
            threads: num_cpus::get(),
            target_block_time_ms: 400, // Permia target block time
            mine_empty_blocks: true,
            max_mining_time: Duration::from_secs(60),
        }
    }
}

impl NodeMinerConfig {
    /// Create config with specific beneficiary
    pub fn with_beneficiary(mut self, addr: Address) -> Self {
        self.beneficiary = addr;
        self
    }

    /// Create config with specific thread count
    pub fn with_threads(mut self, threads: usize) -> Self {
        self.threads = threads.max(1);
        self
    }
}

/// A mined block ready for submission
#[derive(Debug, Clone)]
pub struct MinedBlock {
    /// Block number
    pub number: u64,
    /// Parent hash
    pub parent_hash: B256,
    /// The block hash
    pub hash: B256,
    /// Nonce that solved the PoW
    pub nonce: u64,
    /// Mix hash from PermiaHash
    pub mix_hash: B256,
    /// Difficulty
    pub difficulty: U256,
    /// Mining result with stats
    pub mining_result: MiningResult,
}

/// Messages sent to the node miner
#[derive(Debug)]
pub enum MinerMessage {
    /// Start mining a new block
    StartMining {
        /// Parent block hash
        parent_hash: B256,
        /// Parent block number
        parent_number: u64,
        /// State root after pending transactions
        state_root: B256,
        /// Transactions root
        transactions_root: B256,
        /// Receipts root
        receipts_root: B256,
        /// Difficulty for this block
        difficulty: U256,
        /// Gas used
        gas_used: u64,
    },
    /// Stop current mining
    Stop,
    /// Shutdown the miner
    Shutdown,
}

/// Handle to control the node miner
#[derive(Debug, Clone)]
pub struct NodeMinerHandle {
    tx: mpsc::Sender<MinerMessage>,
    running: Arc<AtomicBool>,
}

impl NodeMinerHandle {
    /// Check if the miner is currently running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Start mining a new block
    pub async fn start_mining(
        &self,
        parent_hash: B256,
        parent_number: u64,
        state_root: B256,
        transactions_root: B256,
        receipts_root: B256,
        difficulty: U256,
        gas_used: u64,
    ) -> Result<(), mpsc::error::SendError<MinerMessage>> {
        self.tx
            .send(MinerMessage::StartMining {
                parent_hash,
                parent_number,
                state_root,
                transactions_root,
                receipts_root,
                difficulty,
                gas_used,
            })
            .await
    }

    /// Stop current mining
    pub async fn stop(&self) -> Result<(), mpsc::error::SendError<MinerMessage>> {
        self.tx.send(MinerMessage::Stop).await
    }

    /// Shutdown the miner completely
    pub async fn shutdown(&self) -> Result<(), mpsc::error::SendError<MinerMessage>> {
        self.tx.send(MinerMessage::Shutdown).await
    }
}

/// Node-integrated miner for Permia PoW
pub struct NodeMiner {
    config: NodeMinerConfig,
    rx: mpsc::Receiver<MinerMessage>,
    mined_tx: mpsc::Sender<MinedBlock>,
    running: Arc<AtomicBool>,
    worker: MiningWorker,
}

impl NodeMiner {
    /// Create a new node miner
    pub fn new(
        config: NodeMinerConfig,
    ) -> (Self, NodeMinerHandle, mpsc::Receiver<MinedBlock>) {
        let (tx, rx) = mpsc::channel(16);
        let (mined_tx, mined_rx) = mpsc::channel(16);
        let running = Arc::new(AtomicBool::new(false));

        let mining_config = MiningConfig {
            threads: config.threads,
            batch_size: 10_000,
            max_duration: Some(config.max_mining_time),
        };

        let miner = Self {
            config,
            rx,
            mined_tx,
            running: Arc::clone(&running),
            worker: MiningWorker::new(mining_config),
        };

        let handle = NodeMinerHandle {
            tx,
            running,
        };

        (miner, handle, mined_rx)
    }

    /// Run the miner loop
    pub async fn run(mut self) {
        info!(
            target: "permia::node_miner",
            beneficiary = %self.config.beneficiary,
            threads = self.config.threads,
            "Node miner started"
        );

        while let Some(msg) = self.rx.recv().await {
            match msg {
                MinerMessage::StartMining {
                    parent_hash,
                    parent_number,
                    state_root,
                    transactions_root,
                    receipts_root,
                    difficulty,
                    gas_used,
                } => {
                    self.running.store(true, Ordering::SeqCst);

                    let block_number = parent_number + 1;
                    info!(
                        target: "permia::node_miner",
                        block = block_number,
                        parent = %parent_hash,
                        difficulty = %difficulty,
                        "Starting to mine block"
                    );

                    // Create block template
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;

                    let mut template = BlockTemplate::new(
                        parent_hash,
                        block_number,
                        timestamp,
                        self.config.beneficiary,
                        difficulty,
                    );
                    template.state_root = state_root;
                    template.transactions_root = transactions_root;
                    template.receipts_root = receipts_root;
                    template.gas_used = gas_used;

                    // Mine the block
                    self.worker.reset();
                    match self.worker.mine(&template) {
                        Ok(result) => {
                            info!(
                                target: "permia::node_miner",
                                block = block_number,
                                nonce = result.nonce,
                                hash = %result.hash,
                                hashrate = format!("{:.2} H/s", result.hashrate()),
                                "Block mined!"
                            );

                            let mined_block = MinedBlock {
                                number: block_number,
                                parent_hash,
                                hash: result.hash,
                                nonce: result.nonce,
                                mix_hash: result.mix_hash,
                                difficulty,
                                mining_result: result,
                            };

                            if let Err(e) = self.mined_tx.send(mined_block).await {
                                error!(
                                    target: "permia::node_miner",
                                    error = %e,
                                    "Failed to send mined block"
                                );
                            }
                        }
                        Err(MiningError::Cancelled) => {
                            debug!(
                                target: "permia::node_miner",
                                block = block_number,
                                "Mining cancelled"
                            );
                        }
                        Err(e) => {
                            warn!(
                                target: "permia::node_miner",
                                block = block_number,
                                error = %e,
                                "Mining failed"
                            );
                        }
                    }

                    self.running.store(false, Ordering::SeqCst);
                }
                MinerMessage::Stop => {
                    debug!(target: "permia::node_miner", "Stopping current mining");
                    self.worker.cancel();
                    self.running.store(false, Ordering::SeqCst);
                }
                MinerMessage::Shutdown => {
                    info!(target: "permia::node_miner", "Shutting down node miner");
                    self.worker.cancel();
                    break;
                }
            }
        }
    }
}

/// Spawn the node miner as a background task
pub fn spawn_node_miner(
    config: NodeMinerConfig,
) -> (NodeMinerHandle, mpsc::Receiver<MinedBlock>) {
    let (miner, handle, mined_rx) = NodeMiner::new(config);

    tokio::spawn(async move {
        miner.run().await;
    });

    (handle, mined_rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_miner_creation() {
        let config = NodeMinerConfig::default()
            .with_beneficiary(Address::ZERO)
            .with_threads(1);

        let (handle, mut mined_rx) = spawn_node_miner(config);

        // Start mining with easy difficulty
        handle
            .start_mining(
                B256::ZERO,
                0,
                B256::ZERO,
                B256::ZERO,
                B256::ZERO,
                U256::from(100u64), // Very easy
                0,
            )
            .await
            .unwrap();

        // Wait for mined block
        let mined = tokio::time::timeout(Duration::from_secs(10), mined_rx.recv())
            .await
            .expect("Mining should complete")
            .expect("Should receive mined block");

        assert_eq!(mined.number, 1);
        assert!(mined.nonce > 0 || mined.nonce == 0); // Any nonce is valid

        // Shutdown
        handle.shutdown().await.unwrap();
    }
}
