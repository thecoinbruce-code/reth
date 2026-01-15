//! Permia Service Proofs
//!
//! This crate defines the service proof types for Permia's full-stack mining:
//! - **Storage**: Proof of Spacetime (PoST) for content storage
//! - **CDN**: Delivery receipts for content delivery
//! - **Compute**: Execution proofs for WASM computation
//!
//! # Service Multipliers (from PROTOCOL_SPEC_v4.md)
//!
//! ```text
//! Miner Reward = Base Block Reward × (1 + Service Multiplier)
//!
//! Service Multiplier Components:
//! ├── Storage Proof (valid): +0.1 to +0.3
//! ├── Compute Proof (valid): +0.1 to +0.3
//! ├── CDN Proof (bandwidth): +0.05 to +0.15
//! ├── Uptime Bonus (99%+): +0.1
//! └── Geographic Bonus (rare region): +0.2 to +0.5
//!
//! Maximum Multiplier: 2.0x
//! ```

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub mod proof;
pub mod storage;
pub mod cdn;
pub mod compute;
pub mod multiplier;

pub use proof::{ServiceProof, ServiceProofType, ServiceProofData};
pub use storage::{StorageProof, StorageParams};
pub use cdn::{CdnProof, CdnParams};
pub use compute::{ComputeProof, ComputeParams};
pub use multiplier::{ServiceMultiplier, calculate_multiplier};

use alloy_primitives::{Address, B256};
use thiserror::Error;

/// Service proof errors
#[derive(Debug, Error)]
pub enum ServiceError {
    /// Invalid proof data
    #[error("Invalid proof data: {0}")]
    InvalidProof(String),
    
    /// Proof verification failed
    #[error("Proof verification failed: {0}")]
    VerificationFailed(String),
    
    /// Unknown service type
    #[error("Unknown service type: {0}")]
    UnknownServiceType(u8),
    
    /// Proof expired
    #[error("Proof expired at epoch {0}, current epoch is {1}")]
    ProofExpired(u64, u64),
}

/// Service type identifiers (from PROTOCOL_SPEC_v4.md)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum ServiceType {
    /// Content storage
    Storage = 0x01,
    /// Content delivery
    Cdn = 0x02,
    /// WASM execution
    Compute = 0x03,
}

impl TryFrom<u8> for ServiceType {
    type Error = ServiceError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(ServiceType::Storage),
            0x02 => Ok(ServiceType::Cdn),
            0x03 => Ok(ServiceType::Compute),
            _ => Err(ServiceError::UnknownServiceType(value)),
        }
    }
}

impl From<ServiceType> for u8 {
    fn from(st: ServiceType) -> u8 {
        st as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_type_conversion() {
        assert_eq!(u8::from(ServiceType::Storage), 0x01);
        assert_eq!(u8::from(ServiceType::Cdn), 0x02);
        assert_eq!(u8::from(ServiceType::Compute), 0x03);
        
        assert_eq!(ServiceType::try_from(0x01).unwrap(), ServiceType::Storage);
        assert!(ServiceType::try_from(0xFF).is_err());
    }
}
