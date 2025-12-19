//! Error types for Reticulum

use thiserror::Error;

/// Main error type for Reticulum operations
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum RnsError {
    #[error("Out of memory")]
    OutOfMemory,

    #[error("Invalid argument")]
    InvalidArgument,

    #[error("Incorrect signature")]
    IncorrectSignature,

    #[error("Incorrect hash")]
    IncorrectHash,

    #[error("Cryptography error")]
    CryptoError,

    #[error("Packet error")]
    PacketError,

    #[error("Connection error")]
    ConnectionError,

    #[error("Timeout")]
    Timeout,

    #[error("Invalid packet format")]
    InvalidPacketFormat,

    #[error("Destination not found")]
    DestinationNotFound,

    #[error("Link not established")]
    LinkNotEstablished,

    #[error("Transport error")]
    TransportError,

    #[error("Interface error: {interface_name}")]
    InterfaceError { interface_name: String },

    #[error("Serialization error")]
    SerializationError,

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Mutex lock error")]
    LockError,
}

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

