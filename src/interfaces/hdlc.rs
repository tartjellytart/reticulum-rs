//! HDLC (High-Level Data Link Control) framing implementation
//! 
//! Provides stateful HDLC frame parsing and encoding for encapsulating
//! raw packets over unreliable transport layers (serial, TCP, etc.)

use crate::error::{RnsError, Result};
use alloc::vec::Vec;

const HDLC_FLAG: u8 = 0x7E;
const HDLC_ESC: u8 = 0x7D;
const HDLC_ESC_MASK: u8 = 0x20;

/// Stateful HDLC frame decoder
pub struct HdlcDecoder {
    frame_buffer: Vec<u8>,
    in_frame: bool,
    escape_next: bool,
}

impl HdlcDecoder {
    /// Create a new HDLC frame decoder
    /// 
    /// Initializes a stateful HDLC decoder for parsing frames from a byte stream.
    /// The decoder buffers incomplete frames and returns complete frames when the flag byte is received.
    pub fn new() -> Self {
        Self {
            frame_buffer: Vec::new(),
            in_frame: false,
            escape_next: false,
        }
    }

    /// Process a single byte and return completed frame if available
    /// 
    /// Maintains HDLC decoding state across calls. When a complete frame is detected
    /// (flag byte received), returns the unescaped frame data.
    /// 
    /// # Arguments
    /// * `byte` - The next byte from the input stream
    /// 
    /// # Returns
    /// * Ok(Some(frame)) - A complete frame has been decoded
    /// * Ok(None) - Byte processed but frame not yet complete
    /// * Err(RnsError::PacketError) - Frame exceeds maximum size (2048 bytes)
    pub fn process_byte(&mut self, byte: u8) -> Result<Option<Vec<u8>>> {
        match byte {
            HDLC_FLAG => {
                if self.in_frame && !self.frame_buffer.is_empty() {
                    // Frame complete
                    let frame = self.frame_buffer.clone();
                    self.frame_buffer.clear();
                    self.escape_next = false;
                    return Ok(Some(frame));
                }
                // Start new frame
                self.in_frame = true;
                self.frame_buffer.clear();
                self.escape_next = false;
                Ok(None)
            }
            HDLC_ESC if self.in_frame => {
                self.escape_next = true;
                Ok(None)
            }
            _ if self.in_frame => {
                let actual_byte = if self.escape_next {
                    self.escape_next = false;
                    byte ^ HDLC_ESC_MASK
                } else {
                    byte
                };

                if self.frame_buffer.len() >= 2048 {
                    self.frame_buffer.clear();
                    self.in_frame = false;
                    return Err(RnsError::PacketError);
                }

                self.frame_buffer.push(actual_byte);
                Ok(None)
            }
            _ => Ok(None), // Ignore bytes outside frames
        }
    }

    /// Reset decoder state
    /// 
    /// Clears any buffered incomplete frames and returns to initial state.
    /// Useful when switching streams or recovering from errors.
    pub fn reset(&mut self) {
        self.frame_buffer.clear();
        self.in_frame = false;
        self.escape_next = false;
    }
}

/// HDLC frame encoder
/// 
/// Stateless HDLC encoder for framing data. Escapes special bytes and adds frame delimiters.
pub struct HdlcEncoder;

impl HdlcEncoder {
    /// Encode data frame with HDLC framing
    /// 
    /// Adds HDLC frame flag bytes (0x7E) at start and end, and escapes any bytes
    /// that match the flag or escape characters using the escape mechanism.
    /// 
    /// # Arguments
    /// * `data` - The raw payload data to encode
    /// 
    /// # Returns
    /// Complete HDLC frame ready for transmission
    pub fn encode(data: &[u8]) -> Vec<u8> {
        let mut frame = Vec::with_capacity(data.len() + 2);
        frame.push(HDLC_FLAG);

        for &byte in data {
            if byte == HDLC_FLAG || byte == HDLC_ESC {
                frame.push(HDLC_ESC);
                frame.push(byte ^ HDLC_ESC_MASK);
            } else {
                frame.push(byte);
            }
        }

        frame.push(HDLC_FLAG);
        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_frame() {
        let data = b"hello";
        let encoded = HdlcEncoder::encode(data);

        let mut decoder = HdlcDecoder::new();
        let mut result = None;

        for &byte in &encoded {
            if let Ok(Some(frame)) = decoder.process_byte(byte) {
                result = Some(frame);
            }
        }

        assert_eq!(result, Some(data.to_vec()));
    }

    #[test]
    fn test_escape_handling() {
        let data = &[0x7E, 0x7D, 0x42]; // FLAG, ESC, 'B'
        let encoded = HdlcEncoder::encode(data);

        let mut decoder = HdlcDecoder::new();
        let mut result = None;

        for &byte in &encoded {
            if let Ok(Some(frame)) = decoder.process_byte(byte) {
                result = Some(frame);
            }
        }

        assert_eq!(result, Some(data.to_vec()));
    }
}
