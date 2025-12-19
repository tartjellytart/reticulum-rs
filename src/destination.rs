//! Destination handling for Reticulum

use crate::hash::AddressHash;
use crate::identity::Identity;

/// Destination types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestinationType {
    Single = 0x00,
    Group = 0x01,
    Plain = 0x02,
    Link = 0x03,
}

/// Destination for sending packets
pub struct Destination {
    pub destination_type: DestinationType,
    pub hash: AddressHash,
    pub identity: Option<Identity>,
}

impl Destination {
    /// Create a new destination
    pub fn new(
        destination_type: DestinationType,
        hash: AddressHash,
        identity: Option<Identity>,
    ) -> Self {
        Self {
            destination_type,
            hash,
            identity,
        }
    }

    /// Create a single destination from an identity
    pub fn single(identity: Identity) -> Self {
        Self {
            destination_type: DestinationType::Single,
            hash: identity.address_hash().clone(),
            identity: Some(identity),
        }
    }
}

