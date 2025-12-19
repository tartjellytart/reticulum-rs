//! Packet handling for Reticulum

use crate::error::{RnsError, Result};
use crate::hash::AddressHash;

/// Packet types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    Data = 0x00,
    Announce = 0x01,
    LinkRequest = 0x02,
    Proof = 0x03,
}

impl From<u8> for PacketType {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0x00 => PacketType::Data,
            0x01 => PacketType::Announce,
            0x02 => PacketType::LinkRequest,
            0x03 => PacketType::Proof,
            _ => PacketType::Data,
        }
    }
}

/// Header types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderType {
    Header1 = 0x00,
    Header2 = 0x01,
}

impl From<u8> for HeaderType {
    fn from(value: u8) -> Self {
        match (value >> 6) & 0b1 {
            0 => HeaderType::Header1,
            1 => HeaderType::Header2,
            _ => HeaderType::Header1,
        }
    }
}

/// Transport types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    Broadcast = 0x00,
    Transport = 0x01,
    Relay = 0x02,
    Tunnel = 0x03,
}

impl From<u8> for TransportType {
    fn from(value: u8) -> Self {
        match (value >> 4) & 0b11 {
            0x00 => TransportType::Broadcast,
            0x01 => TransportType::Transport,
            0x02 => TransportType::Relay,
            0x03 => TransportType::Tunnel,
            _ => TransportType::Broadcast,
        }
    }
}

/// Packet context types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketContext {
    None = 0x00,
    Resource = 0x01,
    ResourceAdv = 0x02,
    ResourceReq = 0x03,
    Keepalive = 0xFA,
    LinkIdentify = 0xFB,
    LinkClose = 0xFC,
    LinkProof = 0xFD,
    LrRtt = 0xFE,
    LrProof = 0xFF,
}

/// Packet structure
#[derive(Debug, Clone)]
pub struct Packet {
    pub packet_type: PacketType,
    pub header_type: HeaderType,
    pub transport_type: TransportType,
    pub context: PacketContext,
    pub hops: u8,
    pub destination_hash: Option<AddressHash>,
    pub transport_id: Option<AddressHash>,
    pub data: Vec<u8>,
    pub raw: Option<Vec<u8>>,
}

impl Packet {
    /// Create a new packet
    pub fn new(
        packet_type: PacketType,
        destination_hash: AddressHash,
        data: Vec<u8>,
    ) -> Self {
        Self {
            packet_type,
            header_type: HeaderType::Header1,
            transport_type: TransportType::Broadcast,
            context: PacketContext::None,
            hops: 0,
            destination_hash: Some(destination_hash),
            transport_id: None,
            data,
            raw: None,
        }
    }

    /// Pack the packet into raw bytes
    pub fn pack(&mut self) -> Result<()> {
        let mut raw = Vec::new();

        // Pack flags byte
        let flags = ((self.header_type as u8) << 6)
            | ((self.transport_type as u8) << 4)
            | (self.packet_type as u8);
        raw.push(flags);

        // Pack hops
        raw.push(self.hops);

        // Pack destination hash (16 bytes for truncated hash)
        if let Some(ref dest_hash) = self.destination_hash {
            raw.extend_from_slice(dest_hash.as_bytes());
        } else {
            return Err(RnsError::InvalidPacketFormat);
        }

        // Pack context
        raw.push(self.context as u8);

        // Pack data
        raw.extend_from_slice(&self.data);

        self.raw = Some(raw);
        Ok(())
    }

    /// Unpack raw bytes into a packet
    pub fn unpack(raw: &[u8]) -> Result<Self> {
        if raw.len() < 19 {
            return Err(RnsError::InvalidPacketFormat);
        }

        let flags = raw[0];
        let hops = raw[1];

        let header_type = HeaderType::from(flags);
        let transport_type = TransportType::from(flags);
        let packet_type = PacketType::from(flags);

        let destination_hash = AddressHash::from_bytes(&raw[2..18])?;
        let context_byte = raw[18];
        let context = match context_byte {
            0x00 => PacketContext::None,
            0x01 => PacketContext::Resource,
            0xFA => PacketContext::Keepalive,
            0xFB => PacketContext::LinkIdentify,
            0xFC => PacketContext::LinkClose,
            0xFD => PacketContext::LinkProof,
            0xFE => PacketContext::LrRtt,
            0xFF => PacketContext::LrProof,
            _ => PacketContext::None,
        };

        let data = raw[19..].to_vec();

        Ok(Self {
            packet_type,
            header_type,
            transport_type,
            context,
            hops,
            destination_hash: Some(destination_hash),
            transport_id: None,
            data,
            raw: Some(raw.to_vec()),
        })
    }

    /// Get the raw packet bytes
    pub fn raw(&self) -> Option<&[u8]> {
        self.raw.as_deref()
    }

    /// Get packet hash
    pub fn hash(&self) -> Result<crate::hash::Hash> {
        if let Some(ref raw) = self.raw {
            Ok(crate::hash::Hash::compute(raw))
        } else {
            Err(RnsError::InvalidPacketFormat)
        }
    }
}

