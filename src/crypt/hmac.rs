//! HMAC (Hash-based Message Authentication Code) implementation

use hmac::{Hmac, Mac};
use sha2::Sha256;

/// HMAC-SHA256 wrapper
pub struct HmacSha256 {
    mac: Hmac<Sha256>,
}

impl HmacSha256 {
    /// Create a new HMAC instance
    pub fn new(key: &[u8]) -> Self {
        Self {
            mac: Hmac::<Sha256>::new_from_slice(key)
                .expect("HMAC can take key of any size"),
        }
    }

    /// Update with data
    pub fn update(&mut self, data: &[u8]) {
        self.mac.update(data);
    }

    /// Finalize and get the digest
    pub fn finalize(self) -> [u8; 32] {
        self.mac.finalize().into_bytes().into()
    }

    /// Compute HMAC in one step
    pub fn compute(key: &[u8], data: &[u8]) -> [u8; 32] {
        let mut mac = Hmac::<Sha256>::new_from_slice(key)
            .expect("HMAC can take key of any size");
        mac.update(data);
        mac.finalize().into_bytes().into()
    }
}

