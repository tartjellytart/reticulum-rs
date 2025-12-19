//! Main Reticulum struct

use crate::error::Result;
use crate::transport::Transport;
use crate::interfaces::Interface;
use crate::packet::Packet;

/// Maximum Transmission Unit (bytes)
pub const MTU: usize = 500;

/// Truncated hash length in bits
pub const TRUNCATED_HASHLENGTH: usize = 128;

/// Header maximum size
pub const HEADER_MAXSIZE: usize = 2 + 1 + (TRUNCATED_HASHLENGTH / 8) * 2;

/// Minimum bitrate (bits per second)
pub const MINIMUM_BITRATE: usize = 5;

/// Default per-hop timeout (seconds)
pub const DEFAULT_PER_HOP_TIMEOUT: usize = 6;

/// Main Reticulum instance
pub struct Reticulum {
    transport: Transport,
}

impl Reticulum {
    /// Create a new Reticulum instance
    pub fn new() -> Self {
        Self {
            transport: Transport::new(),
        }
    }
    
    /// Add an interface to Reticulum
    pub fn add_interface<I: Interface + Send + 'static>(&self, interface: I) -> Result<()> {
        self.transport.register_interface(interface)
    }
    
    /// Unregister an interface from Reticulum
    pub fn unregister_interface(&self, interface_hash: &crate::hash::AddressHash) -> Result<()> {
        self.transport.unregister_interface(interface_hash)
    }
    
    /// Get the number of registered interfaces
    pub fn interface_count(&self) -> Result<usize> {
        self.transport.interface_count()
    }
    
    /// Send a packet through Reticulum
    pub fn send(&self, packet: &mut Packet) -> Result<()> {
        self.transport.outbound(packet)
    }
    
    /// Check if we have a path to a destination
    pub fn has_path(&self, destination_hash: &crate::hash::AddressHash) -> Result<bool> {
        self.transport.has_path(destination_hash)
    }
    
    /// Get hop count to a destination
    pub fn hops_to(&self, destination_hash: &crate::hash::AddressHash) -> Result<Option<u8>> {
        self.transport.hops_to(destination_hash)
    }
    
    /// Get the transport layer (for advanced usage)
    pub fn transport(&self) -> &Transport {
        &self.transport
    }
}

impl Default for Reticulum {
    fn default() -> Self {
        Self::new()
    }
}

