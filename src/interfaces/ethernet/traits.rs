//! Ethernet driver traits for testable architecture

use crate::error::Result;

/// Ethernet configuration
#[derive(Debug, Clone)]
pub struct EthernetConfig {
    pub mac_address: [u8; 6],
    pub ip_address: Option<[u8; 4]>,
    pub subnet_mask: Option<[u8; 4]>,
    pub gateway: Option<[u8; 4]>,
    pub use_dhcp: bool,
}

/// Ethernet driver abstraction for testability
#[allow(async_fn_in_trait)]
pub trait EthernetDriver: Send + Sync {
    /// Initialize the Ethernet interface
    fn init(&mut self, config: &EthernetConfig) -> Result<()>;
    
    /// Check if interface is up
    fn is_up(&self) -> bool;
    
    /// Get MAC address
    fn get_mac(&self) -> [u8; 6];
    
    /// Get IP address
    fn get_ip(&self) -> Option<[u8; 4]>;
    
    /// Send UDP packet
    async fn send_udp(&self, data: &[u8], dest: ([u8; 4], u16), src_port: u16) -> Result<()>;
    
    /// Receive UDP packet (non-blocking)
    async fn recv_udp(&self, buffer: &mut [u8], port: u16) -> Result<Option<(usize, ([u8; 4], u16))>>;
    
    /// Send TCP packet
    async fn send_tcp(&self, data: &[u8], dest: ([u8; 4], u16)) -> Result<()>;
    
    /// Receive TCP packet (non-blocking)
    async fn recv_tcp(&self, buffer: &mut [u8]) -> Result<Option<usize>>;
    
    /// Get link status (connected/disconnected)
    fn link_status(&self) -> bool;
    
    /// Get link speed (Mbps)
    fn link_speed(&self) -> Option<u32>;
}

