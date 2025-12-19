//! TCP Client driver traits for testable architecture

use crate::error::Result;

/// TCP Client configuration
#[derive(Debug, Clone)]
pub struct TcpClientConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: Option<u64>,
    pub reconnect_interval_seconds: Option<u64>,
    pub i2p_tunnel: bool,
}

/// TCP Client driver abstraction for testability
pub trait TcpClientDriver: Send + Sync {
    /// Connect to the TCP server
    async fn connect(&mut self, host: &str, port: u16) -> Result<()>;
    
    /// Disconnect from the TCP server
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Check if connected
    fn is_connected(&self) -> bool;
    
    /// Read data from the TCP stream (non-blocking)
    /// Returns number of bytes read, or None if no data available
    async fn read(&mut self, buffer: &mut [u8]) -> Result<Option<usize>>;
    
    /// Write data to the TCP stream
    async fn write(&mut self, data: &[u8]) -> Result<usize>;
    
    /// Flush the TCP stream
    async fn flush(&mut self) -> Result<()>;
    
    /// Get the remote address (host, port)
    fn get_remote_addr(&self) -> Option<(String, u16)>;
}

