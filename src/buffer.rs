//! Buffer utilities for Reticulum

use crate::error::{RnsError, Result};

/// Static buffer with compile-time size
#[derive(Debug, Clone)]
pub struct StaticBuffer<const N: usize> {
    buffer: [u8; N],
    len: usize,
}

impl<const N: usize> StaticBuffer<N> {
    /// Create a new empty buffer
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; N],
            len: 0,
        }
    }

    /// Create from a slice
    pub fn from_slice(data: &[u8]) -> Result<Self> {
        if data.len() > N {
            return Err(RnsError::OutOfMemory);
        }
        let mut buffer = Self::new();
        buffer.write(data)?;
        Ok(buffer)
    }

    /// Write data to the buffer
    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        if self.len + data.len() > N {
            return Err(RnsError::OutOfMemory);
        }
        let written = data.len();
        self.buffer[self.len..self.len + written].copy_from_slice(data);
        self.len += written;
        Ok(written)
    }

    /// Get the current length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the buffer as a slice
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer[..self.len]
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Get the capacity
    pub fn capacity(&self) -> usize {
        N
    }
}

impl<const N: usize> Default for StaticBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

