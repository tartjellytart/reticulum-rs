//! Identity management for Reticulum

use alloc::vec::Vec;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer};
use x25519_dalek::{StaticSecret, PublicKey, SharedSecret};
use rand_core::{CryptoRngCore, RngCore};
use crate::hash::{AddressHash, HashGenerator};
use crate::crypt::Token;
use crate::error::{RnsError, Result};

/// Public key length (32 bytes for Ed25519)
pub const PUBLIC_KEY_LENGTH: usize = 32;

/// Complete identity key size (256 bits encryption + 256 bits signing = 512 bits = 64 bytes)
pub const KEYSIZE: usize = 64;

/// Derived key length (512 bits = 64 bytes)
pub const DERIVED_KEY_LENGTH: usize = 64;

/// Identity for encryption and signing
#[derive(Clone)]
pub struct Identity {
    /// X25519 public key for encryption
    pub public_key: PublicKey,
    /// Ed25519 verifying key for signatures
    pub verifying_key: VerifyingKey,
    /// Address hash derived from the keys
    pub address_hash: AddressHash,
}

impl Identity {
    /// Create a new identity from public keys
    pub fn new(public_key: PublicKey, verifying_key: VerifyingKey) -> Self {
        // Compute address hash from both keys
        let hash = HashGenerator::new()
            .chain_update(public_key.as_bytes())
            .chain_update(verifying_key.as_bytes())
            .finalize();

        let address_hash = AddressHash::new_from_hash(hash.as_bytes());

        Self {
            public_key,
            verifying_key,
            address_hash,
        }
    }

    /// Create identity from byte slices
    pub fn from_slices(public_key: &[u8], verifying_key: &[u8]) -> Result<Self> {
        if public_key.len() != PUBLIC_KEY_LENGTH || verifying_key.len() != PUBLIC_KEY_LENGTH {
            return Err(RnsError::InvalidArgument);
        }

        let public_key = {
            let mut key_data = [0u8; PUBLIC_KEY_LENGTH];
            key_data.copy_from_slice(public_key);
            PublicKey::from(key_data)
        };

        let verifying_key = VerifyingKey::from_bytes(verifying_key.try_into().unwrap())
            .map_err(|_| RnsError::InvalidArgument)?;

        Ok(Self::new(public_key, verifying_key))
    }

    /// Get the address hash
    pub fn address_hash(&self) -> &AddressHash {
        &self.address_hash
    }

    /// Get public key bytes
    pub fn public_key_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        *self.public_key.as_bytes()
    }

    /// Get verifying key bytes
    pub fn verifying_key_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.verifying_key.to_bytes()
    }
}

/// Full identity with private keys (for local use)
pub struct FullIdentity {
    /// X25519 secret key for encryption
    secret_key: StaticSecret,
    /// Ed25519 signing key for signatures
    signing_key: SigningKey,
    /// Public identity
    pub identity: Identity,
}

impl FullIdentity {
    /// Generate a new identity
    pub fn generate<R: RngCore + CryptoRngCore>(rng: &mut R) -> Self {
        // Generate X25519 secret key
        let mut secret_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_bytes);
        let secret_key = StaticSecret::from(secret_bytes);
        
        // Generate Ed25519 signing key
        let mut signing_bytes = [0u8; 32];
        rng.fill_bytes(&mut signing_bytes);
        let signing_key = SigningKey::from_bytes(&signing_bytes);

        let public_key = PublicKey::from(&secret_key);
        let verifying_key = signing_key.verifying_key();

        let identity = Identity::new(public_key, verifying_key);

        Self {
            secret_key,
            signing_key,
            identity,
        }
    }

    /// Create from existing keys
    pub fn from_keys(secret_key: StaticSecret, signing_key: SigningKey) -> Self {
        let public_key = PublicKey::from(&secret_key);
        let verifying_key = signing_key.verifying_key();

        let identity = Identity::new(public_key, verifying_key);

        Self {
            secret_key,
            signing_key,
            identity,
        }
    }

    /// Get the public identity
    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Sign data
    pub fn sign(&self, data: &[u8]) -> Signature {
        self.signing_key.sign(data)
    }

    /// Verify a signature
    pub fn verify(&self, data: &[u8], signature: &Signature) -> bool {
        self.identity.verifying_key.verify_strict(data, signature).is_ok()
    }

    /// Derive shared secret with another identity
    pub fn derive_shared_secret(&self, other: &Identity) -> SharedSecret {
        self.secret_key.diffie_hellman(&other.public_key)
    }

    /// Encrypt data for another identity
    pub fn encrypt_for<R: RngCore + CryptoRngCore>(
        &self,
        rng: &mut R,
        other: &Identity,
        data: &[u8],
    ) -> Result<Vec<u8>> {
        // Derive shared secret
        let shared_secret = self.derive_shared_secret(other);

        // Derive encryption key using HKDF
        let derived_key = crate::crypt::hkdf(
            DERIVED_KEY_LENGTH,
            shared_secret.as_bytes(),
            None,
            None,
        )?;

        // Create token and encrypt
        let token = Token::new(&derived_key)?;
        token.encrypt(rng, data)
    }

    /// Decrypt data from another identity
    pub fn decrypt_from(&self, other: &Identity, encrypted: &[u8]) -> Result<Vec<u8>> {
        // Derive shared secret
        let shared_secret = self.secret_key.diffie_hellman(&other.public_key);

        // Derive encryption key using HKDF
        let derived_key = crate::crypt::hkdf(
            DERIVED_KEY_LENGTH,
            shared_secret.as_bytes(),
            None,
            None,
        )?;

        // Create token and decrypt
        let token = Token::new(&derived_key)?;
        token.decrypt(encrypted)
    }
}

impl Identity {
    /// Verify a signature
    pub fn verify(&self, data: &[u8], signature: &Signature) -> bool {
        self.verifying_key.verify_strict(data, signature).is_ok()
    }
}

