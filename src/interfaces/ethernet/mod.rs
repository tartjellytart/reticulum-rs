//! Ethernet interface implementation

mod traits;

pub use traits::{EthernetDriver, EthernetConfig};

use crate::error::{RnsError, Result};
use crate::interfaces::{Interface, InterfaceMode, InterfaceStateFields};
use crate::hash::{AddressHash, HashGenerator};

/// Ethernet interface for Reticulum
pub struct EthernetInterface<D: EthernetDriver> {
    ethernet_driver: D,
    ethernet_config: EthernetConfig,
    state: InterfaceStateFields,
    udp_port: u16,
}

impl<D: EthernetDriver> EthernetInterface<D> {
    /// Create a new Ethernet interface
    /// 
    /// Initializes an Ethernet connection interface for local network communication.
    /// The interface uses UDP for packet transport over Ethernet.
    /// 
    /// # Arguments
    /// * `ethernet_driver` - The underlying Ethernet driver implementation
    /// * `ethernet_config` - Ethernet configuration
    /// * `name` - Human-readable name for this interface
    /// * `udp_port` - UDP port for Reticulum packet transmission
    pub fn new(ethernet_driver: D, ethernet_config: EthernetConfig, name: &str, udp_port: u16) -> Result<Self> {
        Ok(Self {
            ethernet_driver,
            ethernet_config,
            state: InterfaceStateFields::new(name, 100_000_000, 1500),
            udp_port,
        })
    }
    
    /// Start the Ethernet interface
    /// 
    /// Initializes the Ethernet hardware and checks for link status.
    /// The interface transitions to online state once link is established.
    /// 
    /// # Errors
    /// Returns error if initialization fails or link is not available
    pub async fn start(&mut self) -> Result<()> {
        self.ethernet_driver.init(&self.ethernet_config)?;
        
        if !self.ethernet_driver.link_status() {
            return Err(RnsError::ConnectionError);
        }
        
        self.state.online = true;
        Ok(())
    }
    
    /// Stop the Ethernet interface
    /// 
    /// Marks the interface as offline. Does not disable hardware.
    pub fn stop(&mut self) -> Result<()> {
        self.state.online = false;
        Ok(())
    }
    
    /// Read and process incoming UDP packets
    /// 
    /// Asynchronously receives UDP packets from the configured port and processes them.
    /// Calls [`process_incoming()`](Interface::process_incoming) for each packet received.
    /// 
    /// # Errors
    /// Returns error if UDP receive fails
    pub async fn read_and_process(&mut self) -> Result<()> {
        let mut buffer = [0u8; 1500]; // Ethernet MTU
        
        // Check for UDP packets
        if let Some((len, _source)) = self.ethernet_driver.recv_udp(&mut buffer, self.udp_port).await? {
            self.state.rx_bytes += len as u64;
            self.process_incoming(&buffer[..len])?;
        }
        
        Ok(())
    }
    
    /// Get UDP port
    pub fn udp_port(&self) -> u16 {
        self.udp_port
    }
}

impl<D: EthernetDriver> Interface for EthernetInterface<D> {
    fn process_incoming(&mut self, _data: &[u8]) -> Result<()> {
        // Process incoming Reticulum packet
        // This would call Transport.inbound() in the full implementation
        Ok(())
    }
    
    fn process_outgoing(&mut self, data: &[u8]) -> Result<()> {
        if !self.state.online {
            return Err(RnsError::ConnectionError);
        }
        
        // For now, use broadcast address
        // In full implementation, would determine destination from packet
        let _dest = ([255, 255, 255, 255], self.udp_port);
        
        // Note: This is sync but send_udp is async
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
        InterfaceMode::Full
    }
    
    fn is_online(&self) -> bool {
        self.state.online
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
            .chain_update(&self.ethernet_driver.get_mac())
            .chain_update(&self.udp_port.to_le_bytes())
            .finalize();
        AddressHash::new_from_hash(hash.as_bytes())
    }
}

