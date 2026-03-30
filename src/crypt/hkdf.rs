//! HKDF (HMAC-based Key Derivation Function) implementation

use alloc::vec;
use alloc::vec::Vec;
use hkdf::Hkdf;
use sha2::Sha256;
use crate::error::{RnsError, Result};

/// Derive a key using HKDF
pub fn hkdf(
    length: usize,
    derive_from: &[u8],
    salt: Option<&[u8]>,
    context: Option<&[u8]>,
) -> Result<Vec<u8>> {
    if length == 0 {
        return Err(RnsError::InvalidArgument);
    }

    if derive_from.is_empty() {
        return Err(RnsError::InvalidArgument);
    }

    let salt = salt.unwrap_or(&[0u8; 32]);
    let context = context.unwrap_or(&[]);

    let hk = Hkdf::<Sha256>::new(Some(salt), derive_from);
    let mut okm = vec![0u8; length];
    hk.expand(context, &mut okm)
        .map_err(|_| RnsError::CryptoError)?;

    Ok(okm)
}

