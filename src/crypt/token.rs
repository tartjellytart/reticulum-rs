//! Token encryption/decryption (Fernet-like, without version/timestamp)

use aes::Aes256;
use cbc::{Decryptor, Encryptor};
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use alloc::vec::Vec;
use cbc::cipher::block_padding::Pkcs7;
use crate::crypt::hmac::HmacSha256;
use crate::error::{RnsError, Result};
use rand_core::{CryptoRngCore, RngCore};

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

const TOKEN_OVERHEAD: usize = 48; // 16 (IV) + 32 (HMAC)
const BLOCKSIZE: usize = 16;

/// Token for encryption/decryption (Fernet-like without version/timestamp)
pub struct Token {
    signing_key: [u8; 32],
    encryption_key: [u8; 32],
}

impl Token {
    /// Create a new token from a 64-byte key (32 signing + 32 encryption)
    pub fn new(key: &[u8]) -> Result<Self> {
        if key.len() != 64 {
            return Err(RnsError::InvalidArgument);
        }

        let mut signing_key = [0u8; 32];
        let mut encryption_key = [0u8; 32];
        signing_key.copy_from_slice(&key[..32]);
        encryption_key.copy_from_slice(&key[32..]);

        Ok(Self {
            signing_key,
            encryption_key,
        })
    }

    /// Generate a random key
    pub fn generate_key<R: RngCore + CryptoRngCore>(rng: &mut R) -> [u8; 64] {
        let mut key = [0u8; 64];
        rng.fill_bytes(&mut key);
        key
    }

    /// Encrypt data
    pub fn encrypt<R: RngCore + CryptoRngCore>(
        &self,
        rng: &mut R,
        data: &[u8],
    ) -> Result<Vec<u8>> {
        // Generate random IV
        let mut iv = [0u8; BLOCKSIZE];
        rng.fill_bytes(&mut iv);

        // Encrypt using CBC mode (will pad automatically)
        let cipher = Aes256CbcEnc::new_from_slices(&self.encryption_key, &iv)
            .map_err(|_| RnsError::CryptoError)?;
        
        let mut buffer = data.to_vec();
        let buffer = cipher.encrypt_padded_mut::<Pkcs7>(&mut buffer, data.len())
            .map_err(|_| RnsError::CryptoError)?;

        // Build signed parts: IV + ciphertext
        let mut signed_parts = Vec::with_capacity(iv.len() + buffer.len());
        signed_parts.extend_from_slice(&iv);
        signed_parts.extend_from_slice(&buffer);

        // Compute HMAC
        let hmac = HmacSha256::compute(&self.signing_key, &signed_parts);

        // Return: IV + ciphertext + HMAC
        signed_parts.extend_from_slice(&hmac);
        Ok(signed_parts)
    }

    /// Decrypt data
    pub fn decrypt(&self, token: &[u8]) -> Result<Vec<u8>> {
        if token.len() < TOKEN_OVERHEAD {
            return Err(RnsError::InvalidArgument);
        }

        // Split token: signed_parts + hmac
        let hmac_start = token.len() - 32;
        let signed_parts = &token[..hmac_start];
        let received_hmac = &token[hmac_start..];

        // Verify HMAC
        let expected_hmac = HmacSha256::compute(&self.signing_key, signed_parts);
        if received_hmac != expected_hmac.as_slice() {
            return Err(RnsError::IncorrectSignature);
        }

        // Extract IV and ciphertext
        let iv = &signed_parts[..BLOCKSIZE];
        let ciphertext = &signed_parts[BLOCKSIZE..];

        // Decrypt using CBC mode (will unpad automatically)
        let cipher = Aes256CbcDec::new_from_slices(&self.encryption_key, iv)
            .map_err(|_| RnsError::CryptoError)?;
        
        let mut buffer = ciphertext.to_vec();
        let plaintext = cipher.decrypt_padded_mut::<Pkcs7>(&mut buffer)
            .map_err(|_| RnsError::CryptoError)?;

        Ok(plaintext.to_vec())
    }
}

