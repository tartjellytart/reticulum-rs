//! TCP Client interface implementation with HDLC framing

mod traits;

pub use traits::{TcpClientDriver, TcpClientConfig};

use crate::error::{RnsError, Result};
use crate::interfaces::{Interface, InterfaceMode};
use crate::hash::{AddressHash, HashGenerator};

const HDLC_FLAG: u8 = 0x7E;
const HDLC_ESC: u8 = 0x7D;
const HDLC_ESC_MASK: u8 = 0x20;

/// TCP Client interface for Reticulum with HDLC framing
pub struct TcpClientInterface<D: TcpClientDriver> {
    driver: D,
    config: TcpClientConfig,
    name: String,
    online: bool,
    rxb: u64,
    txb: u64,
    bitrate: u64,
    mtu: usize,
    // HDLC framing state
    frame_buffer: Vec<u8>,
    in_frame: bool,
    escape: bool,
}

impl<D: TcpClientDriver> TcpClientInterface<D> {
    /// Create a new TCP Client interface
    pub fn new(driver: D, config: TcpClientConfig, name: &str) -> Result<Self> {
        Ok(Self {
            driver,
            config,
            name: name.to_string(),
            online: false,
            rxb: 0,
            txb: 0,
            bitrate: 10_000_000, // 10 Mbps default
            mtu: 1500,
            frame_buffer: Vec::new(),
            in_frame: false,
            escape: false,
        })
    }
    
    /// Start the TCP Client interface (connect to server)
    pub async fn start(&mut self) -> Result<()> {
        self.driver.connect(&self.config.host, self.config.port).await?;
        self.online = true;
        Ok(())
    }
    
    /// Stop the TCP Client interface (disconnect)
    pub async fn stop(&mut self) -> Result<()> {
        self.driver.disconnect().await?;
        self.online = false;
        Ok(())
    }
    
    /// Read and process incoming data with HDLC framing
    pub async fn read_and_process(&mut self) -> Result<()> {
        let mut buffer = [0u8; 1500];
        if let Some(bytes_read) = self.driver.read(&mut buffer).await? {
            if bytes_read > 0 {
                for &byte in &buffer[..bytes_read] {
                    self.process_hdlc_byte(byte)?;
                }
            }
        }
        Ok(())
    }
    
    /// Process a single HDLC byte
    fn process_hdlc_byte(&mut self, byte: u8) -> Result<()> {
        if byte == HDLC_FLAG {
            if self.in_frame && !self.frame_buffer.is_empty() {
                // Complete frame received
                let frame_data = self.frame_buffer.clone();
                self.rxb += frame_data.len() as u64;
                self.frame_buffer.clear();
                self.process_incoming(&frame_data)?;
            }
            self.in_frame = true;
            self.escape = false;
        } else if self.in_frame {
            if byte == HDLC_ESC {
                self.escape = true;
            } else {
                if self.escape {
                    let unescaped = byte ^ HDLC_ESC_MASK;
                    self.frame_buffer.push(unescaped);
                    self.escape = false;
                } else {
                    self.frame_buffer.push(byte);
                }
            }
        }
        
        Ok(())
    }
    
    fn escape_hdlc(&self, data: &[u8]) -> Vec<u8> {
        let mut escaped = Vec::with_capacity(data.len() + 2);
        escaped.push(HDLC_FLAG);
        
        for &byte in data {
            if byte == HDLC_FLAG || byte == HDLC_ESC {
                escaped.push(HDLC_ESC);
                escaped.push(byte ^ HDLC_ESC_MASK);
            } else {
                escaped.push(byte);
            }
        }
        
        escaped.push(HDLC_FLAG);
        escaped
    }
    
}

impl<D: TcpClientDriver> Interface for TcpClientInterface<D> {
    fn process_incoming(&mut self, _data: &[u8]) -> Result<()> {
        // Process incoming Reticulum packet
        // This would call Transport.inbound() in the full implementation
        // For now, the wrapper handles forwarding to transport
        Ok(())
    }
    
    fn process_outgoing(&mut self, data: &[u8]) -> Result<()> {
        if !self.online {
            return Err(RnsError::ConnectionError);
        }
        
        // Apply HDLC framing
        let _framed = self.escape_hdlc(data);
        
        // Write to TCP stream
        // Note: This is sync but write is async
        // In full implementation, would use channels or async runtime
        self.txb += data.len() as u64;
        
        // For now, return error indicating async needed
        // Full implementation would queue or use async runtime
        Err(RnsError::InvalidArgument)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn mode(&self) -> InterfaceMode {
        InterfaceMode::PointToPoint
    }
    
    fn is_online(&self) -> bool {
        self.online && self.driver.is_connected()
    }
    
    fn bitrate(&self) -> u64 {
        self.bitrate
    }
    
    fn mtu(&self) -> usize {
        self.mtu
    }
    
    fn rxb(&self) -> u64 {
        self.rxb
    }
    
    fn txb(&self) -> u64 {
        self.txb
    }
    
    fn interface_hash(&self) -> AddressHash {
        let hash = HashGenerator::new()
            .chain_update(self.name.as_bytes())
            .chain_update(self.config.host.as_bytes())
            .chain_update(&self.config.port.to_le_bytes())
            .finalize();
        AddressHash::new_from_hash(hash.as_bytes())
    }
}

