//! Permia P2P Block Gossip
//!
//! This crate provides PermiaHash PoW block validation and gossip for the Permia network.
//! It implements the `BlockImport` trait to handle incoming block announcements from peers,
//! validate them using PermiaHash proof-of-work, and propagate valid blocks.
//!
//! # Architecture
//!
//! ```text
//! Peer A (Miner)                    Peer B (Syncing)
//!      │                                  │
//!      │  NewBlock (via eth protocol)     │
//!      │─────────────────────────────────►│
//!      │                                  │
//!      │                          PermiaPoWBlockImport
//!      │                                  │
//!      │                          1. Validate PermiaHash
//!      │                          2. Check difficulty
//!      │                          3. Submit to Engine API
//!      │                                  │
//!      │                          BlockValidation::ValidBlock
//!      │                                  │
//!      │◄─────────────────────────────────│
//!      │  NewBlockHashes (relay)          │
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use permia_gossip::PermiaPoWBlockImport;
//!
//! let block_import = PermiaPoWBlockImport::new(
//!     consensus,
//!     engine_handle,
//!     provider,
//! );
//! ```

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

mod announcer;
mod block_import;
mod error;

pub use announcer::{PermiaBlockAnnouncer, spawn_block_announcer};
pub use block_import::PermiaPoWBlockImport;
pub use error::PermiaGossipError;

/// Re-export core types
pub use reth_network::import::{BlockImport, BlockImportEvent, BlockValidation, NewBlockEvent};
