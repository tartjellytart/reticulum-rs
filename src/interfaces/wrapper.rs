//! Type-erased interface wrapper for transport layer integration

use crate::error::Result;
use crate::interfaces::{Interface, InterfaceMode};
use crate::hash::AddressHash;
use std::sync::{Arc, Mutex};

/// Type-erased interface wrapper
pub struct InterfaceWrapper {
    inner: Arc<Mutex<dyn Interface + Send>>,
    interface_hash: AddressHash,
}

impl InterfaceWrapper {
    /// Create a new interface wrapper
    pub fn new<I: Interface + Send + 'static>(interface: I) -> Self {
        let hash = interface.interface_hash();
        
        Self {
            inner: Arc::new(Mutex::new(interface)),
            interface_hash: hash,
        }
    }
    
    /// Get the interface hash
    pub fn interface_hash(&self) -> AddressHash {
        self.interface_hash.clone()
    }
    
    /// Process incoming data
    pub fn process_incoming(&self, data: &[u8]) -> Result<()> {
        let mut guard = self.inner.lock().map_err(|_| crate::error::RnsError::LockError)?;
        guard.process_incoming(data)
    }
    
    /// Process outgoing data
    pub fn process_outgoing(&self, data: &[u8]) -> Result<()> {
        let mut guard = self.inner.lock().map_err(|_| crate::error::RnsError::LockError)?;
        guard.process_outgoing(data)
    }
    
    /// Check if interface is online
    pub fn is_online(&self) -> bool {
        let guard = self.inner.lock().ok();
        guard.map(|g| g.is_online()).unwrap_or(false)
    }
    
    /// Get interface name
    pub fn name(&self) -> String {
        let guard = self.inner.lock().ok();
        guard.map(|g| g.name().to_string()).unwrap_or_else(|| "unknown".to_string())
    }
    
    /// Get interface mode
    pub fn mode(&self) -> InterfaceMode {
        let guard = self.inner.lock().ok();
        guard.map(|g| g.mode()).unwrap_or(InterfaceMode::Full)
    }
}

impl Clone for InterfaceWrapper {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            interface_hash: self.interface_hash.clone(),
        }
    }
}
