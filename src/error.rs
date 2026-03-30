//! Error types for Reticulum
//! 
//! Provides specific, actionable error types for production diagnostics
//! and error handling in embedded and networked systems.

use alloc::string::String;
use core::fmt;

/// Main error type for Reticulum operations
/// 
/// Provides specific error variants for different failure modes, enabling
/// precise error handling and diagnostics in embedded systems.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RnsError {
    /// Memory allocation failed
    OutOfMemory,

    /// Invalid argument provided to function
    InvalidArgument,

    /// Cryptographic signature verification failed
    IncorrectSignature,

    /// Hash verification failed
    IncorrectHash,

    /// Cryptography operation failed
    CryptoError,

    /// Packet parsing or construction failed
    PacketError,

    /// Network connection failed or closed
    ConnectionError,

    /// Operation timeout
    Timeout,

    /// Invalid packet format or structure
    InvalidPacketFormat,

    /// Destination address not found or unreachable
    DestinationNotFound,

    /// Link establishment failed
    LinkNotEstablished,

    /// Transport layer error
    TransportError,

    /// Interface-specific error with context
    InterfaceError { 
        /// Name of the interface that failed
        interface_name: String 
    },

    /// Serialization or encoding failed
    SerializationError,

    /// Generic I/O error with description
    IoError(String),

    /// Mutex lock poisoned
    LockError,

    /// Device not found or not available
    DeviceNotFound,

    /// Permission denied for operation
    PermissionDenied,

    /// Invalid configuration
    InvalidConfiguration,

    /// Operation not supported on this platform/interface
    NotSupported,
}

impl fmt::Display for RnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfMemory => write!(f, "Out of memory"),
            Self::InvalidArgument => write!(f, "Invalid argument"),
            Self::IncorrectSignature => write!(f, "Incorrect signature"),
            Self::IncorrectHash => write!(f, "Incorrect hash"),
            Self::CryptoError => write!(f, "Cryptography error"),
            Self::PacketError => write!(f, "Packet error"),
            Self::ConnectionError => write!(f, "Connection error"),
            Self::Timeout => write!(f, "Timeout"),
            Self::InvalidPacketFormat => write!(f, "Invalid packet format"),
            Self::DestinationNotFound => write!(f, "Destination not found"),
            Self::LinkNotEstablished => write!(f, "Link not established"),
            Self::TransportError => write!(f, "Transport error"),
            Self::InterfaceError { interface_name } => {
                write!(f, "Interface error: {interface_name}")
            }
            Self::SerializationError => write!(f, "Serialization error"),
            Self::IoError(err) => write!(f, "IO error: {err}"),
            Self::LockError => write!(f, "Mutex lock error"),
            Self::DeviceNotFound => write!(f, "Device not found"),
            Self::PermissionDenied => write!(f, "Permission denied"),
            Self::InvalidConfiguration => write!(f, "Invalid configuration"),
            Self::NotSupported => write!(f, "Operation not supported"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RnsError {}

impl From<core::fmt::Error> for RnsError {
    fn from(_: core::fmt::Error) -> Self {
        RnsError::InvalidArgument
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for RnsError {
    fn from(e: std::io::Error) -> Self {
        RnsError::IoError(e.to_string())
    }
}

#[cfg(feature = "std")]
impl<T> From<std::sync::PoisonError<T>> for RnsError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        RnsError::LockError
    }
}

pub type Result<T> = core::result::Result<T, RnsError>;

