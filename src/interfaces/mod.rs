//! Interface implementations for Reticulum
//!
//! Provides transport-independent interface abstraction for various link types
//! including Serial, TCP, Ethernet, and WiFi.

pub mod hdlc;
pub mod serial;
pub mod tcp;
pub mod ethernet;
pub mod wifi;
pub mod wrapper;

use crate::hash::AddressHash;
use crate::error::Result;

/// Interface mode enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterfaceMode {
    /// Full duplex mode
    Full,
    /// Point-to-Point mode
    PointToPoint,
    /// Access Point mode
    AccessPoint,
    /// Roaming mode
    Roaming,
    /// Boundary mode
    Boundary,
}

/// Common interface state fields for all interface implementations
/// 
/// This struct consolidates the common fields shared across Serial, TCP, Ethernet, and WiFi
/// interfaces to reduce code duplication and ensure consistency.
pub struct InterfaceStateFields {
    /// Interface name identifier
    pub name: alloc::string::String,
    /// Online/connected status
    pub online: bool,
    /// Interface bitrate in bits per second
    pub bitrate: u64,
    /// Maximum transmission unit in bytes
    pub mtu: usize,
    /// Received bytes counter
    pub rx_bytes: u64,
    /// Transmitted bytes counter
    pub tx_bytes: u64,
}

impl InterfaceStateFields {
    /// Create new interface state with given parameters
    pub fn new(name: &str, bitrate: u64, mtu: usize) -> Self {
        Self {
            name: name.to_string(),
            online: false,
            bitrate,
            mtu,
            rx_bytes: 0,
            tx_bytes: 0,
        }
    }
}

/// Core interface trait
/// 
/// Defines the common interface for all transport implementations (Serial, TCP, Ethernet, WiFi).
/// Implementations should handle framing, error recovery, and state management specific to their transport.
pub trait Interface: Send {
    /// Get unique hash for this interface
    /// 
    /// Returns a cryptographically secure hash derived from interface name and transport parameters.
    /// Used for routing and interface identification in the Reticulum network.
    fn interface_hash(&self) -> AddressHash;
    
    /// Process incoming data from the transport
    /// 
    /// Called when data is received from the physical transport layer.
    /// Implementation should frame/unframe data and forward to the transport handler.
    /// 
    /// # Errors
    /// Returns error if the data cannot be processed (framing error, malformed packet, etc.)
    fn process_incoming(&mut self, data: &[u8]) -> Result<()>;
    
    /// Process outgoing data for transmission
    /// 
    /// Called when the transport layer needs to send data.
    /// Implementation should frame the data and queue it for transmission.
    /// 
    /// # Errors
    /// Returns error if the interface is offline or data cannot be processed
    fn process_outgoing(&mut self, data: &[u8]) -> Result<()>;
    
    /// Check if interface is currently online and operational
    /// 
    /// Returns true if the interface is connected and ready to transmit/receive.
    /// Implementations should check the underlying transport status.
    fn is_online(&self) -> bool;
    
    /// Get the human-readable name of this interface
    /// 
    /// Used for logging, debugging, and route selection.
    /// Should be unique within a Reticulum instance.
    fn name(&self) -> &str;
    
    /// Get the operational mode of this interface
    /// 
    /// Indicates whether the interface operates in full-duplex, point-to-point,
    /// access point, or other modes.
    fn mode(&self) -> InterfaceMode;
    
    /// Get the bitrate in bits per second
    /// 
    /// Used for link capacity estimation and route metric calculation.
    /// For variable-rate interfaces, return the typical bitrate.
    fn bitrate(&self) -> u64;
    
    /// Get maximum transmission unit in bytes
    /// 
    /// The largest single packet that can be transmitted on this interface.
    /// Affects packet fragmentation decisions in the routing layer.
    fn mtu(&self) -> usize;
    
    /// Get total received bytes count
    /// 
    /// Used for statistics and monitoring. Implementation should track all bytes
    /// received at the application layer (after framing/deframing).
    fn rxb(&self) -> u64;
    
    /// Get total transmitted bytes count
    /// 
    /// Used for statistics and monitoring. Implementation should track all bytes
    /// transmitted at the application layer (after framing/deframing).
    fn txb(&self) -> u64;
}
