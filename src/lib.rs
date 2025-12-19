//! # Reticulum-rs
//!
//! Rust implementation of the Reticulum Network Stack - a cryptographic,
//! decentralised, and resilient mesh networking protocol.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod error;
pub mod hash;
pub mod identity;
pub mod packet;
pub mod crypt;
pub mod buffer;
pub mod destination;
pub mod link;
pub mod transport;
pub mod interfaces;
pub mod reticulum;

// Re-export commonly used types
pub use error::{RnsError, Result};
pub use identity::{Identity, FullIdentity};
pub use packet::Packet;
pub use destination::Destination;
pub use link::Link;
pub use reticulum::{Reticulum, MTU};


