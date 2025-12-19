//! Ethernet interface implementation

mod traits;

pub use traits::{EthernetDriver, EthernetConfig};

use crate::error::{RnsError, Result};
use crate::interfaces::{Interface, InterfaceMode};
use crate::hash::{AddressHash, HashGenerator};

/// Ethernet interface for Reticulum
pub struct EthernetInterface<D: EthernetDriver> {
    driver: D,
    config: EthernetConfig,
    name: String,
    online: bool,
    rxb: u64,
    txb: u64,
    bitrate: u64,
    mtu: usize,
    udp_port: u16,
}

impl<D: EthernetDriver> EthernetInterface<D> {
    /// Create a new Ethernet interface
    pub fn new(driver: D, config: EthernetConfig, name: &str, udp_port: u16) -> Result<Self> {
        // Use a default bitrate based on typical Ethernet speeds
        // In real implementation, would get from driver
        let bitrate = 100_000_000; // 100 Mbps default
        
        Ok(Self {
            driver,
            config,
            name: name.to_string(),
            online: false,
            rxb: 0,
            txb: 0,
            bitrate,
            mtu: 1500,
            udp_port,
        })
    }
    
    /// Start the Ethernet interface
    pub async fn start(&mut self) -> Result<()> {
        self.driver.init(&self.config)?;
        
        if !self.driver.link_status() {
            return Err(RnsError::ConnectionError);
        }
        
        self.online = true;
        Ok(())
    }
    
    /// Stop the Ethernet interface
    pub fn stop(&mut self) -> Result<()> {
        self.online = false;
        Ok(())
    }
    
    /// Read and process incoming packets
    pub async fn read_and_process(&mut self) -> Result<()> {
        let mut buffer = [0u8; 1500]; // Ethernet MTU
        
        // Check for UDP packets
        if let Some((len, _source)) = self.driver.recv_udp(&mut buffer, self.udp_port).await? {
            self.rxb += len as u64;
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
        if !self.online {
            return Err(RnsError::ConnectionError);
        }
        
        // For now, use broadcast address
        // In full implementation, would determine destination from packet
        let _dest = ([255, 255, 255, 255], self.udp_port);
        
        // Note: This is sync but send_udp is async
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
        InterfaceMode::Full
    }
    
    fn is_online(&self) -> bool {
        self.online
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
            .chain_update(&self.driver.get_mac())
            .chain_update(&self.udp_port.to_le_bytes())
            .finalize();
        AddressHash::new_from_hash(hash.as_bytes())
    }
}

