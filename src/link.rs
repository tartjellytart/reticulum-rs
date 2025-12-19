//! Link management for Reticulum

use crate::destination::Destination;
use crate::identity::Identity;

/// Link states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    Pending = 0x00,
    Handshake = 0x01,
    Active = 0x02,
    Stale = 0x03,
    Closed = 0x04,
}

/// Link for establishing encrypted connections
pub struct Link {
    pub destination: Destination,
    pub state: LinkState,
    pub remote_identity: Option<Identity>,
}

impl Link {
    /// Create a new link
    pub fn new(destination: Destination) -> Self {
        Self {
            destination,
            state: LinkState::Pending,
            remote_identity: None,
        }
    }
}

