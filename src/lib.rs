//! # Reticulum-rs
//!
//! Rust implementation of the Reticulum Network Stack - a cryptographic,
//! decentralised, and resilient mesh networking protocol.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod error;
pub mod hash;
pub mod identity;
pub mod packet;
pub mod crypt;
pub mod buffer;
pub mod destination;
pub mod link;
#[cfg(feature = "std")]
pub mod transport;
pub mod interfaces;
#[cfg(feature = "std")]
pub mod reticulum;

// Re-export commonly used types
pub use error::{RnsError, Result};
pub use identity::{Identity, FullIdentity};
pub use packet::Packet;
pub use destination::Destination;
pub use link::Link;
#[cfg(feature = "std")]
pub use reticulum::{Reticulum, MTU};


