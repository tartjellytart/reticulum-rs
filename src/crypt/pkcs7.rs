//! PKCS7 padding implementation

use crate::error::{RnsError, Result};

const BLOCKSIZE: usize = 16;

/// Pad data using PKCS7
pub fn pkcs7_pad(data: &[u8], block_size: usize) -> Vec<u8> {
    let padding_len = block_size - (data.len() % block_size);
    let padding_value = padding_len as u8;
    let mut padded = data.to_vec();
    padded.resize(data.len() + padding_len, padding_value);
    padded
}

/// Unpad data using PKCS7
pub fn pkcs7_unpad(data: &[u8], block_size: usize) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(RnsError::InvalidArgument);
    }

    let padding_len = data[data.len() - 1] as usize;
    
    if padding_len == 0 || padding_len > block_size || padding_len > data.len() {
        return Err(RnsError::InvalidArgument);
    }

    // Verify all padding bytes are the same
    for i in (data.len() - padding_len)..data.len() {
        if data[i] != padding_len as u8 {
            return Err(RnsError::InvalidArgument);
        }
    }

    Ok(data[..data.len() - padding_len].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkcs7_pad_unpad() {
        let data = b"hello";
        let padded = pkcs7_pad(data, BLOCKSIZE);
        let unpadded = pkcs7_unpad(&padded, BLOCKSIZE).unwrap();
        assert_eq!(data, unpadded.as_slice());
    }
}

