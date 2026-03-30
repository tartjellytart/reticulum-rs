//! Hash utilities for Reticulum

use alloc::string::String;
use sha2::{Digest, Sha256, Sha512};
use crate::error::{RnsError, Result};

/// Truncated hash length in bits (128 bits = 16 bytes)
pub const TRUNCATED_HASH_LENGTH: usize = 128 / 8;

/// Address hash type (truncated SHA-256)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddressHash {
    hash: [u8; TRUNCATED_HASH_LENGTH],
}

impl Default for AddressHash {
    fn default() -> Self {
        Self {
            hash: [0u8; TRUNCATED_HASH_LENGTH],
        }
    }
}

impl AddressHash {
    /// Create a new address hash from a full hash
    pub fn new_from_hash(hash: &[u8]) -> Self {
        let mut truncated = [0u8; TRUNCATED_HASH_LENGTH];
        truncated.copy_from_slice(&hash[..TRUNCATED_HASH_LENGTH]);
        Self { hash: truncated }
    }

    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != TRUNCATED_HASH_LENGTH {
            return Err(RnsError::InvalidArgument);
        }
        let mut hash = [0u8; TRUNCATED_HASH_LENGTH];
        hash.copy_from_slice(bytes);
        Ok(Self { hash })
    }

    /// Get the hash as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.hash
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.hash)
    }
}

/// Full hash (SHA-256)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hash {
    hash: [u8; 32],
}

impl Hash {
    /// Create a new hash from bytes
    pub fn new(hash: [u8; 32]) -> Self {
        Self { hash }
    }

    /// Create a hash from a slice
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() != 32 {
            return Err(RnsError::InvalidArgument);
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(slice);
        Ok(Self { hash })
    }

    /// Compute SHA-256 hash of data
    pub fn compute(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        Self {
            hash: hasher.finalize().into(),
        }
    }

    /// Get the hash as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.hash
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.hash)
    }
}

/// Hash generator for building hashes incrementally
pub struct HashGenerator {
    hasher: Sha256,
}

impl HashGenerator {
    /// Create a new hash generator
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    /// Chain update data
    pub fn chain_update(mut self, data: &[u8]) -> Self {
        self.hasher.update(data);
        self
    }

    /// Finalize the hash
    pub fn finalize(self) -> Hash {
        Hash {
            hash: self.hasher.finalize().into(),
        }
    }
}

impl Default for HashGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// SHA-512 hash
pub struct Hash512 {
    hash: [u8; 64],
}

impl Hash512 {
    /// Compute SHA-512 hash of data
    pub fn compute(data: &[u8]) -> Self {
        let mut hasher = Sha512::new();
        hasher.update(data);
        Self {
            hash: hasher.finalize().into(),
        }
    }

    /// Get the hash as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.hash
    }
}

