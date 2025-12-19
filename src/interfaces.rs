//! Interface system for Reticulum

pub mod wifi;
pub mod serial;
pub mod ethernet;
pub mod tcp;
pub mod wrapper;

use crate::error::Result;
use crate::hash::AddressHash;

/// Interface modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceMode {
    Full,
    PointToPoint,
    AccessPoint,
    Roaming,
    Boundary,
    Gateway,
}

/// Direction flags for interface traffic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    In,
    Out,
    Forward,
    Repeat,
}

/// Interface trait for Reticulum network interfaces
pub trait Interface {
    /// Process incoming data from the interface
    fn process_incoming(&mut self, data: &[u8]) -> Result<()>;
    
    /// Process outgoing data to the interface
    fn process_outgoing(&mut self, data: &[u8]) -> Result<()>;
    
    /// Get the interface name
    fn name(&self) -> &str;
    
    /// Get the interface mode
    fn mode(&self) -> InterfaceMode;
    
    /// Check if interface is online
    fn is_online(&self) -> bool;
    
    /// Get interface bitrate (bits per second)
    fn bitrate(&self) -> u64;
    
    /// Get interface MTU
    fn mtu(&self) -> usize;
    
    /// Get received bytes count
    fn rxb(&self) -> u64;
    
    /// Get transmitted bytes count
    fn txb(&self) -> u64;
    
    /// Get interface hash (for identification)
    fn interface_hash(&self) -> AddressHash;
}

