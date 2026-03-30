//! TCP Client interface implementation with HDLC framing

mod traits;

pub use traits::{TcpClientDriver, TcpClientConfig};

use crate::error::{RnsError, Result};
use crate::interfaces::{Interface, InterfaceMode, InterfaceStateFields, hdlc::{HdlcDecoder, HdlcEncoder}};
use crate::hash::{AddressHash, HashGenerator};

/// TCP Client interface for Reticulum with HDLC framing
pub struct TcpClientInterface<D: TcpClientDriver> {
    tcp_driver: D,
    tcp_config: TcpClientConfig,
    state: InterfaceStateFields,
    hdlc_decoder: HdlcDecoder,
}

impl<D: TcpClientDriver> TcpClientInterface<D> {
    /// Create a new TCP Client interface
    /// 
    /// Establishes a client-side TCP connection interface with HDLC packet framing.
    /// The interface is created in offline state and must be connected with [`start()`](Self::start).
    /// 
    /// # Arguments
    /// * `tcp_driver` - The underlying TCP driver implementation
    /// * `tcp_config` - TCP configuration (host, port)
    /// * `name` - Human-readable name for this interface
    pub fn new(tcp_driver: D, tcp_config: TcpClientConfig, name: &str) -> Result<Self> {
        Ok(Self {
            tcp_driver,
            tcp_config,
            state: InterfaceStateFields::new(name, 10_000_000, 1500),
            hdlc_decoder: HdlcDecoder::new(),
        })
    }
    
    /// Start the TCP Client interface (connect to server)
    /// 
    /// Asynchronously connects to the configured TCP server.
    /// Marks the interface as online once connection succeeds.
    /// 
    /// # Errors
    /// Returns error if connection fails (host unreachable, connection refused, etc.)
    pub async fn start(&mut self) -> Result<()> {
        self.tcp_driver.connect(&self.tcp_config.host, self.tcp_config.port).await?;
        self.state.online = true;
        Ok(())
    }
    
    /// Stop the TCP Client interface (disconnect)
    /// 
    /// Asynchronously disconnects from the TCP server and marks the interface as offline.
    /// 
    /// # Errors
    /// Returns error if disconnect fails
    pub async fn stop(&mut self) -> Result<()> {
        self.tcp_driver.disconnect().await?;
        self.state.online = false;
        Ok(())
    }
    
    /// Read and process incoming data with HDLC framing
    /// 
    /// Asynchronously reads available data from the TCP connection and processes
    /// it through the HDLC decoder. Calls [`process_incoming()`](Interface::process_incoming)
    /// for each complete frame received.
    /// 
    /// # Errors
    /// Returns error if TCP read fails or HDLC frame processing fails
    pub async fn read_and_process(&mut self) -> Result<()> {
        let mut buffer = [0u8; 1500];
        if let Some(bytes_read) = self.tcp_driver.read(&mut buffer).await? {
            if bytes_read > 0 {
                for &byte in &buffer[..bytes_read] {
                    if let Some(frame_data) = self.hdlc_decoder.process_byte(byte)? {
                        self.state.rx_bytes += frame_data.len() as u64;
                        self.process_incoming(&frame_data)?;
                    }
                }
            }
        }
        Ok(())
    }
    
    fn escape_hdlc(&self, data: &[u8]) -> Vec<u8> {
        HdlcEncoder::encode(data)
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
        if !self.state.online {
            return Err(RnsError::ConnectionError);
        }
        
        // Apply HDLC framing
        let _framed = self.escape_hdlc(data);
        
        // Write to TCP stream
        // Note: This is sync but write is async
        // In full implementation, would use channels or async runtime
        self.state.tx_bytes += data.len() as u64;
        
        // For now, return error indicating async needed
        // Full implementation would queue or use async runtime
        Err(RnsError::InvalidArgument)
    }
    
    fn name(&self) -> &str {
        &self.state.name
    }
    
    fn mode(&self) -> InterfaceMode {
        InterfaceMode::PointToPoint
    }
    
    fn is_online(&self) -> bool {
        self.state.online && self.tcp_driver.is_connected()
    }
    
    fn bitrate(&self) -> u64 {
        self.state.bitrate
    }
    
    fn mtu(&self) -> usize {
        self.state.mtu
    }
    
    fn rxb(&self) -> u64 {
        self.state.rx_bytes
    }
    
    fn txb(&self) -> u64 {
        self.state.tx_bytes
    }
    
    fn interface_hash(&self) -> AddressHash {
        let hash = HashGenerator::new()
            .chain_update(self.state.name.as_bytes())
            .chain_update(self.tcp_config.host.as_bytes())
            .chain_update(&self.tcp_config.port.to_le_bytes())
            .finalize();
        AddressHash::new_from_hash(hash.as_bytes())
    }
}

