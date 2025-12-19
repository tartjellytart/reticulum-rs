//! Transport layer for Reticulum

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::error::{RnsError, Result};
use crate::packet::{Packet, PacketType, TransportType};
use crate::hash::{AddressHash, Hash};
use crate::interfaces::wrapper::InterfaceWrapper;

macro_rules! lock_mutex {
    ($mutex:expr) => {
        $mutex.lock().map_err(|_| RnsError::LockError)?
    };
}

/// Path entry in the path table
#[derive(Debug, Clone)]
pub struct PathEntry {
    pub timestamp: f64,
    pub received_from: AddressHash,
    pub hops: u8,
    pub expires: f64,
    pub random_blobs: Vec<Vec<u8>>,
    pub receiving_interface_hash: AddressHash,
    pub announce_packet_hash: Hash,
}

impl PathEntry {
    pub fn new(
        received_from: AddressHash,
        hops: u8,
        interface_hash: AddressHash,
        announce_hash: Hash,
    ) -> Self {
        let now = Instant::now().elapsed().as_secs_f64();
        // Default expiration: 6 seconds per hop
        let expires = now + (hops as f64 * 6.0);
        
        Self {
            timestamp: now,
            received_from,
            hops,
            expires,
            random_blobs: Vec::new(),
            receiving_interface_hash: interface_hash,
            announce_packet_hash: announce_hash,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        let now = Instant::now().elapsed().as_secs_f64();
        now > self.expires
    }
}

/// Announce entry in the announce table
#[derive(Debug, Clone)]
pub struct AnnounceEntry {
    pub packet: Packet,
    pub retries: u8,
    pub retransmit_timeout: f64,
    pub block_rebroadcasts: bool,
    pub attached_interface_hash: Option<AddressHash>,
    pub hops: u8,
}

impl AnnounceEntry {
    pub fn new(packet: Packet, interface_hash: Option<AddressHash>) -> Self {
        let now = Instant::now().elapsed().as_secs_f64();
        Self {
            packet,
            retries: 0,
            retransmit_timeout: now + 2.0, // Initial retransmit after 2 seconds
            block_rebroadcasts: false,
            attached_interface_hash: interface_hash,
            hops: 0,
        }
    }
    
    pub fn should_retransmit(&self) -> bool {
        let now = Instant::now().elapsed().as_secs_f64();
        now >= self.retransmit_timeout && self.retries < 3
    }
}

/// Link entry in the link table
#[derive(Debug, Clone)]
pub struct LinkEntry {
    pub destination_hash: AddressHash,
    pub next_hop_interface_hash: AddressHash,
    pub received_interface_hash: AddressHash,
    pub hops: u8,
    pub timestamp: f64,
    pub validated: bool,
    pub proof_timeout: f64,
}

impl LinkEntry {
    pub fn new(
        destination_hash: AddressHash,
        next_hop: AddressHash,
        received_from: AddressHash,
        hops: u8,
    ) -> Self {
        let now = Instant::now().elapsed().as_secs_f64();
        Self {
            destination_hash,
            next_hop_interface_hash: next_hop,
            received_interface_hash: received_from,
            hops,
            timestamp: now,
            validated: false,
            proof_timeout: now + 10.0, // Proof timeout
        }
    }
    
    pub fn is_proof_expired(&self) -> bool {
        let now = Instant::now().elapsed().as_secs_f64();
        now > self.proof_timeout
    }
}

/// Reverse table entry for proof generation
#[derive(Debug, Clone)]
pub struct ReverseEntry {
    pub destination_hash: AddressHash,
    pub outbound_interface_hash: AddressHash,
    pub received_interface_hash: AddressHash,
    pub timestamp: f64,
}

impl ReverseEntry {
    pub fn new(
        destination_hash: AddressHash,
        outbound: AddressHash,
        received: AddressHash,
    ) -> Self {
        Self {
            destination_hash,
            outbound_interface_hash: outbound,
            received_interface_hash: received,
            timestamp: Instant::now().elapsed().as_secs_f64(),
        }
    }
}

/// Path request entry for tracking pending path discovery
#[derive(Debug, Clone)]
pub struct PathRequestEntry {
    pub destination_hash: AddressHash,
    pub timestamp: f64,
    pub expires: f64,
    pub request_packet_hash: Hash,
}

impl PathRequestEntry {
    pub fn new(destination_hash: AddressHash, request_hash: Hash) -> Self {
        let now = Instant::now().elapsed().as_secs_f64();
        // Path requests expire after 5 seconds
        Self {
            destination_hash,
            timestamp: now,
            expires: now + 5.0,
            request_packet_hash: request_hash,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        let now = Instant::now().elapsed().as_secs_f64();
        now > self.expires
    }
}

/// Receipt entry for tracking outgoing packets that need proof
#[derive(Debug, Clone)]
pub struct ReceiptEntry {
    pub packet_hash: Hash,
    pub truncated_hash: Hash, // Truncated hash for matching proofs
    pub destination_hash: AddressHash,
    pub timestamp: f64,
    pub expires: f64,
    pub delivered: bool,
    pub retries: u8,
}

impl ReceiptEntry {
    pub fn new(packet_hash: Hash, destination_hash: AddressHash) -> Self {
        let now = Instant::now().elapsed().as_secs_f64();
        // Receipts expire after 10 seconds
        let expires = now + 10.0;
        
        // Create truncated hash (first 16 bytes, padded to 32 for Hash type)
        let mut truncated_bytes = [0u8; 32];
        truncated_bytes[..crate::hash::TRUNCATED_HASH_LENGTH].copy_from_slice(
            &packet_hash.as_bytes()[..crate::hash::TRUNCATED_HASH_LENGTH]
        );
        let truncated_hash = Hash::new(truncated_bytes);
        
        Self {
            packet_hash,
            truncated_hash,
            destination_hash,
            timestamp: now,
            expires,
            delivered: false,
            retries: 0,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        let now = Instant::now().elapsed().as_secs_f64();
        now > self.expires
    }
    
    pub fn should_retry(&self) -> bool {
        !self.delivered && !self.is_expired() && self.retries < 3
    }
}

/// Transport layer for Reticulum
#[derive(Clone)]
pub struct Transport {
    // Path table: destination_hash -> PathEntry
    path_table: Arc<Mutex<HashMap<AddressHash, PathEntry>>>,
    
    // Announce table: destination_hash -> AnnounceEntry
    announce_table: Arc<Mutex<HashMap<AddressHash, AnnounceEntry>>>,
    
    // Link table: link_id -> LinkEntry
    link_table: Arc<Mutex<HashMap<AddressHash, LinkEntry>>>,
    
    // Reverse table: truncated_packet_hash -> ReverseEntry
    reverse_table: Arc<Mutex<HashMap<Hash, ReverseEntry>>>,
    
    // Packet hashlist for duplicate detection
    packet_hashlist: Arc<Mutex<VecDeque<Hash>>>,
    
    // Maximum hashlist size
    max_hashlist_size: usize,
    
    // Interface registry: interface_hash -> InterfaceWrapper
    interfaces: Arc<Mutex<HashMap<AddressHash, InterfaceWrapper>>>,
    
    // Path request table: destination_hash -> PathRequestEntry
    path_requests: Arc<Mutex<HashMap<AddressHash, PathRequestEntry>>>,
    
    // Receipts table: truncated_packet_hash -> ReceiptEntry
    receipts: Arc<Mutex<HashMap<Hash, ReceiptEntry>>>,
}

impl Transport {
    /// Create a new transport layer
    pub fn new() -> Self {
        Self {
            path_table: Arc::new(Mutex::new(HashMap::new())),
            announce_table: Arc::new(Mutex::new(HashMap::new())),
            link_table: Arc::new(Mutex::new(HashMap::new())),
            reverse_table: Arc::new(Mutex::new(HashMap::new())),
            packet_hashlist: Arc::new(Mutex::new(VecDeque::new())),
            max_hashlist_size: 1000,
            interfaces: Arc::new(Mutex::new(HashMap::new())),
            path_requests: Arc::new(Mutex::new(HashMap::new())),
            receipts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Register an interface with the transport layer
    pub fn register_interface<I: crate::interfaces::Interface + Send + 'static>(&self, interface: I) -> Result<()> {
        let wrapper = InterfaceWrapper::new(interface);
        let interface_hash = wrapper.interface_hash();
        
        let mut interfaces = lock_mutex!(self.interfaces);
        interfaces.insert(interface_hash, wrapper);
        Ok(())
    }
    
    /// Unregister an interface from the transport layer
    pub fn unregister_interface(&self, interface_hash: &AddressHash) -> Result<()> {
        let mut interfaces = lock_mutex!(self.interfaces);
        interfaces.remove(interface_hash);
        Ok(())
    }
    
    /// Get all registered interface hashes
    pub fn get_interface_hashes(&self) -> Result<Vec<AddressHash>> {
        let interfaces = lock_mutex!(self.interfaces);
        Ok(interfaces.keys().cloned().collect())
    }
    
    /// Get interface count
    pub fn interface_count(&self) -> Result<usize> {
        let interfaces = lock_mutex!(self.interfaces);
        Ok(interfaces.len())
    }
    
    /// Process an incoming packet from an interface
    pub fn inbound(&self, data: &[u8], interface_hash: AddressHash) -> Result<()> {
        // Unpack the packet
        let packet = Packet::unpack(data)?;
        
        // Check for duplicates
        let packet_hash = packet.hash()?;
        if self.is_duplicate(&packet_hash)? {
            return Ok(()); // Duplicate packet, silently ignore
        }
        
        // Add to hashlist
        self.add_to_hashlist(packet_hash.clone())?;
        
        // Clone interface_hash before moving packet
        let interface_hash_clone = interface_hash.clone();
        
        // Update reverse table for proof generation
        if let Some(ref dest_hash) = packet.destination_hash {
            self.update_reverse_table(&packet_hash, dest_hash.clone(), interface_hash_clone.clone())?;
        }
        
        // Process based on packet type
        match packet.packet_type {
            PacketType::Announce => {
                self.process_announce(packet, interface_hash_clone.clone())?;
            }
            PacketType::Data => {
                self.process_data(packet, interface_hash_clone)?;
            }
            PacketType::LinkRequest => {
                self.process_link_request(packet, interface_hash_clone)?;
            }
            PacketType::Proof => {
                self.process_proof(packet, interface_hash_clone)?;
            }
        }
        
        Ok(())
    }
    
    /// Send a packet outbound
    pub fn outbound(&self, packet: &mut Packet) -> Result<()> {
        // Pack the packet if not already packed
        if packet.raw().is_none() {
            packet.pack()?;
        }
        
        // Create receipt for packets that need proof (Transport type)
        if packet.transport_type == TransportType::Transport {
            if let Some(ref dest_hash) = packet.destination_hash {
                let packet_hash = packet.hash()?;
                self.create_receipt(packet_hash, dest_hash.clone())?;
            }
        }
        
        // Determine routing
        if let Some(ref dest_hash) = packet.destination_hash {
            // Check if we have a path
            if let Some(path_entry) = self.get_path(dest_hash)? {
                // Route via path
                self.route_via_path(packet, &path_entry)?;
            } else {
                // Broadcast or request path
                match packet.transport_type {
                    TransportType::Broadcast => {
                        self.broadcast_packet(packet)?;
                    }
                    TransportType::Transport => {
                        // Request path discovery
                        self.request_path(dest_hash)?;
                        // Also broadcast for now
                        self.broadcast_packet(packet)?;
                    }
                    _ => {
                        return Err(RnsError::InvalidArgument);
                    }
                }
            }
        } else {
            return Err(RnsError::InvalidPacketFormat);
        }
        
        Ok(())
    }
    
    /// Create a receipt for tracking an outgoing packet
    pub fn create_receipt(&self, packet_hash: Hash, destination_hash: AddressHash) -> Result<()> {
        let receipt = ReceiptEntry::new(packet_hash.clone(), destination_hash);
        let truncated_hash = receipt.truncated_hash.clone();
        
        let mut receipts = lock_mutex!(self.receipts);
        receipts.insert(truncated_hash, receipt);
        Ok(())
    }
    
    /// Get receipt status for a packet hash
    pub fn get_receipt_status(&self, packet_hash: &Hash) -> Result<Option<bool>> {
        // Create truncated hash (first 16 bytes, padded to 32 for Hash type)
        let mut truncated_bytes = [0u8; 32];
        truncated_bytes[..crate::hash::TRUNCATED_HASH_LENGTH].copy_from_slice(
            &packet_hash.as_bytes()[..crate::hash::TRUNCATED_HASH_LENGTH]
        );
        let truncated_hash = Hash::new(truncated_bytes);
        
        let receipts = lock_mutex!(self.receipts);
        Ok(receipts.get(&truncated_hash).map(|r| r.delivered))
    }
    
    /// Check receipt timeouts and clean up expired receipts
    pub fn check_receipt_timeouts(&self) -> Result<()> {
        let mut receipts = lock_mutex!(self.receipts);
        receipts.retain(|_, receipt| !receipt.is_expired());
        Ok(())
    }
    
    /// Get all pending receipts (not delivered, not expired)
    pub fn get_pending_receipts(&self) -> Result<Vec<Hash>> {
        let receipts = lock_mutex!(self.receipts);
        Ok(receipts.values()
            .filter(|r| !r.delivered && !r.is_expired())
            .map(|r| r.packet_hash.clone())
            .collect())
    }
    
    /// Get receipt count
    pub fn receipt_count(&self) -> Result<usize> {
        let receipts = lock_mutex!(self.receipts);
        Ok(receipts.len())
    }
    
    /// Check if a packet hash is a duplicate
    fn is_duplicate(&self, packet_hash: &Hash) -> Result<bool> {
        let hashlist = lock_mutex!(self.packet_hashlist);
        Ok(hashlist.contains(packet_hash))
    }
    
    /// Add a packet hash to the hashlist
    fn add_to_hashlist(&self, packet_hash: Hash) -> Result<()> {
        let mut hashlist = lock_mutex!(self.packet_hashlist);
        hashlist.push_back(packet_hash);
        
        // Trim if too large
        while hashlist.len() > self.max_hashlist_size {
            hashlist.pop_front();
        }
        Ok(())
    }
    
    /// Update reverse table for proof generation
    fn update_reverse_table(
        &self,
        packet_hash: &Hash,
        destination_hash: AddressHash,
        received_interface: AddressHash,
    ) -> Result<()> {
        // For now, we don't know the outbound interface yet
        // This would be set when we actually route the packet
        let mut reverse_table = lock_mutex!(self.reverse_table);
        
        // Use truncated hash for reverse table key (first 16 bytes, padded to 32)
        let mut truncated_bytes = [0u8; 32];
        truncated_bytes[..16].copy_from_slice(&packet_hash.as_bytes()[..16]);
        let truncated_hash = Hash::new(truncated_bytes);
        
        let received_interface_clone = received_interface.clone();
        reverse_table.insert(
            truncated_hash,
            ReverseEntry::new(destination_hash, received_interface_clone.clone(), received_interface_clone),
        );
        
        Ok(())
    }
    
    /// Process an announce packet
    fn process_announce(&self, packet: Packet, interface_hash: AddressHash) -> Result<()> {
        if let Some(ref dest_hash) = packet.destination_hash {
            let announce_hash = packet.hash()?;
            let dest_hash_clone = dest_hash.clone();
            let interface_hash_clone = interface_hash.clone();
            
            // Create or update path entry
            let path_entry = PathEntry::new(
                interface_hash_clone.clone(),
                packet.hops,
                interface_hash_clone.clone(),
                announce_hash,
            );
            
            // Check if we already have a path
            let mut path_table = lock_mutex!(self.path_table);
            let should_update = path_table
                .get(&dest_hash_clone)
                .map(|existing| {
                    existing.hops > path_entry.hops || existing.is_expired()
                })
                .unwrap_or(true);
            
            if should_update {
                path_table.insert(dest_hash_clone.clone(), path_entry);
                
                // If we had a pending path request for this destination, remove it
                self.remove_path_request(&dest_hash_clone)?;
            }
            drop(path_table);
            
            // Add to announce table
            let mut announce_table = lock_mutex!(self.announce_table);
            announce_table.insert(
                dest_hash_clone.clone(),
                AnnounceEntry::new(packet, Some(interface_hash_clone)),
            );
            drop(announce_table);
            
            // Propagate announce to other interfaces
            self.propagate_announce(&dest_hash_clone)?;
        }
        
        Ok(())
    }
    
    /// Process a data packet
    fn process_data(&self, packet: Packet, _interface_hash: AddressHash) -> Result<()> {
        // Check if packet is for us or needs forwarding
        if let Some(ref dest_hash) = packet.destination_hash {
            // TODO: Check if destination is local
            // For now, just forward if we have a path
            if let Some(path_entry) = self.get_path(dest_hash)? {
                // Forward packet
                self.forward_packet(&packet, &path_entry)?;
            }
        }
        
        Ok(())
    }
    
    /// Get a path entry for a destination
    fn get_path(&self, destination_hash: &AddressHash) -> Result<Option<PathEntry>> {
        let path_table = lock_mutex!(self.path_table);
        Ok(path_table.get(destination_hash).cloned())
    }
    
    /// Check if we have a path to a destination
    pub fn has_path(&self, destination_hash: &AddressHash) -> Result<bool> {
        let path_table = lock_mutex!(self.path_table);
        Ok(path_table.get(destination_hash)
            .map(|entry| !entry.is_expired())
            .unwrap_or(false))
    }
    
    /// Route a packet via a known path
    fn route_via_path(&self, packet: &Packet, path_entry: &PathEntry) -> Result<()> {
        let interfaces = lock_mutex!(self.interfaces);
        
        if let Some(interface) = interfaces.get(&path_entry.receiving_interface_hash) {
            if let Some(raw) = packet.raw() {
                interface.process_outgoing(raw).map_err(|_e| {
                    RnsError::InterfaceError {
                        interface_name: interface.name(),
                    }
                })?;
            }
        }
        
        Ok(())
    }
    
    /// Broadcast a packet to all interfaces
    fn broadcast_packet(&self, packet: &Packet) -> Result<()> {
        let interfaces = lock_mutex!(self.interfaces);
        
        if let Some(raw) = packet.raw() {
            for interface in interfaces.values() {
                if interface.is_online() {
                    // Log errors but don't fail the entire broadcast
                    if let Err(e) = interface.process_outgoing(raw) {
                        log::warn!("Failed to broadcast on interface {}: {:?}", 
                                  interface.name(), e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Forward a packet to the next hop
    fn forward_packet(&self, packet: &Packet, path_entry: &PathEntry) -> Result<()> {
        // Increment hop count
        let mut forwarded_packet = packet.clone();
        forwarded_packet.hops += 1;
        forwarded_packet.pack()?;
        
        // Route via path
        self.route_via_path(&forwarded_packet, path_entry)
    }
    
    /// Request path discovery for a destination
    fn request_path(&self, destination_hash: &AddressHash) -> Result<()> {
        // Check if we already have a pending request
        let path_requests = lock_mutex!(self.path_requests);
        if path_requests.contains_key(destination_hash) {
            // Already have a pending request, don't send another
            return Ok(());
        }
        drop(path_requests);
        
        // Check if we already have a path
        if self.get_path(destination_hash)?.is_some() {
            // Already have a path, no need to request
            return Ok(());
        }
        
        // Create a LinkRequest packet
        let mut link_request = Packet {
            packet_type: PacketType::LinkRequest,
            header_type: crate::packet::HeaderType::Header1,
            transport_type: TransportType::Transport,
            context: crate::packet::PacketContext::None,
            hops: 0,
            destination_hash: Some(destination_hash.clone()),
            transport_id: None,
            data: Vec::new(),
            raw: None,
        };
        
        link_request.pack()?;
        let request_hash = link_request.hash()?;
        
        // Track the path request
        let mut path_requests = lock_mutex!(self.path_requests);
        path_requests.insert(
            destination_hash.clone(),
            PathRequestEntry::new(destination_hash.clone(), request_hash),
        );
        drop(path_requests);
        
        // Broadcast the path request to all interfaces
        self.broadcast_packet(&link_request)?;
        
        Ok(())
    }
    
    /// Process an incoming link request
    /// Link requests are used for both path discovery and link establishment
    fn process_link_request(&self, packet: Packet, interface_hash: AddressHash) -> Result<()> {
        if let Some(ref dest_hash) = packet.destination_hash {
            // Check if we have a path to this destination (path discovery)
            if let Some(path_entry) = self.get_path(dest_hash)? {
                // We have a path! Respond with an announce
                let announce_table = lock_mutex!(self.announce_table);
                
                if let Some(announce_entry) = announce_table.get(dest_hash) {
                    // Create a response announce with the path information
                    let mut response_announce = announce_entry.packet.clone();
                    response_announce.hops = path_entry.hops;
                    response_announce.pack()?;
                    
                    // Send the announce back to the requesting interface
                    let interfaces = lock_mutex!(self.interfaces);
                    if let Some(interface) = interfaces.get(&interface_hash) {
                        if let Some(raw) = response_announce.raw() {
                            if interface.is_online() {
                                if let Err(e) = interface.process_outgoing(raw) {
                                    log::warn!("Failed to send path response on interface {}: {:?}",
                                              interface.name(), e);
                                }
                            }
                        }
                    }
                }
                
                // Also create/update a link entry for tracking
                // The link_id is the destination_hash
                self.add_link_entry(
                    dest_hash.clone(),
                    dest_hash.clone(),
                    path_entry.receiving_interface_hash.clone(),
                    interface_hash.clone(),
                    packet.hops,
                )?;
            } else {
                // No path yet, but create a link entry anyway for tracking
                // This will be validated when path is discovered
                self.add_link_entry(
                    dest_hash.clone(),
                    dest_hash.clone(),
                    interface_hash.clone(), // Use receiving interface as next hop for now
                    interface_hash.clone(),
                    packet.hops,
                )?;
            }
        }
        
        // Forward the link request to other interfaces (if not already processed)
        // This allows path discovery to propagate through the network
        let packet_hash = packet.hash()?;
        if !self.is_duplicate(&packet_hash)? {
            self.add_to_hashlist(packet_hash)?;
            // Forward with incremented hops
            let mut forwarded = packet.clone();
            forwarded.hops += 1;
            forwarded.pack()?;
            
            let interfaces = lock_mutex!(self.interfaces);
            if let Some(raw) = forwarded.raw() {
                for (hash, interface) in interfaces.iter() {
                    if hash != &interface_hash && interface.is_online() {
                        if let Err(e) = interface.process_outgoing(raw) {
                            log::warn!("Failed to forward link request on interface {}: {:?}",
                                      interface.name(), e);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Process an incoming proof packet
    /// Proofs are used to validate that packets were received
    fn process_proof(&self, packet: Packet, _interface_hash: AddressHash) -> Result<()> {
        // Validate the proof and mark corresponding receipt as delivered
        self.validate_proof(&packet)?;
        
        if let Some(ref dest_hash) = packet.destination_hash {
            // Look up the link for this destination
            let link_entry = self.get_link_entry(dest_hash)?;
            
            if let Some(entry) = link_entry {
                // Validate the link if proof is received
                if !entry.validated {
                    self.validate_link(dest_hash)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate a proof packet and mark corresponding receipt as delivered
    pub fn validate_proof(&self, proof_packet: &Packet) -> Result<()> {
        // The proof packet data contains the original packet hash
        if proof_packet.data.len() < 32 {
            return Err(RnsError::InvalidPacketFormat);
        }
        
        // Extract the original packet hash from proof data
        let original_hash_bytes: [u8; 32] = proof_packet.data[..32]
            .try_into()
            .map_err(|_| RnsError::InvalidPacketFormat)?;
        let original_hash = Hash::new(original_hash_bytes);
        
        // Create truncated hash for lookup (first 16 bytes, padded to 32 for Hash type)
        let mut truncated_bytes = [0u8; 32];
        truncated_bytes[..crate::hash::TRUNCATED_HASH_LENGTH].copy_from_slice(
            &original_hash.as_bytes()[..crate::hash::TRUNCATED_HASH_LENGTH]
        );
        let truncated_hash = Hash::new(truncated_bytes);
        
        // Look up receipt
        let mut receipts = lock_mutex!(self.receipts);
        if let Some(receipt) = receipts.get_mut(&truncated_hash) {
            // Verify the full hash matches
            if receipt.packet_hash.as_bytes() == original_hash.as_bytes() {
                receipt.delivered = true;
            }
        }
        
        Ok(())
    }
    
    /// Generate a proof packet for a received packet
    /// This should be called when we receive a packet that requires proof
    pub fn generate_proof(&self, packet_hash: &Hash, destination_hash: &AddressHash) -> Result<Packet> {
        // Create a proof packet
        let mut proof = Packet {
            packet_type: PacketType::Proof,
            header_type: crate::packet::HeaderType::Header1,
            transport_type: TransportType::Transport,
            context: crate::packet::PacketContext::LinkProof,
            hops: 0,
            destination_hash: Some(destination_hash.clone()),
            transport_id: None,
            data: packet_hash.as_bytes().to_vec(), // Include the packet hash in proof
            raw: None,
        };
        
        proof.pack()?;
        Ok(proof)
    }
    
    /// Clean up expired path requests
    pub fn cleanup_path_requests(&self) -> Result<()> {
        let mut path_requests = lock_mutex!(self.path_requests);
        path_requests.retain(|_, entry| !entry.is_expired());
        Ok(())
    }
    
    /// Check if we have a pending path request for a destination
    pub fn has_pending_path_request(&self, destination_hash: &AddressHash) -> Result<bool> {
        let path_requests = lock_mutex!(self.path_requests);
        Ok(path_requests.contains_key(destination_hash))
    }
    
    /// Remove a path request (called when path is discovered)
    fn remove_path_request(&self, destination_hash: &AddressHash) -> Result<()> {
        let mut path_requests = lock_mutex!(self.path_requests);
        path_requests.remove(destination_hash);
        Ok(())
    }
    
    /// Propagate an announce to other interfaces
    fn propagate_announce(&self, destination_hash: &AddressHash) -> Result<()> {
        let announce_table = lock_mutex!(self.announce_table);
        
        if let Some(announce_entry) = announce_table.get(destination_hash) {
            // Create a new announce packet with incremented hops
            let mut announce_packet = announce_entry.packet.clone();
            announce_packet.hops += 1;
            announce_packet.pack()?;
            
            // Get the received interface hash before dropping the lock
            let received_interface_hash = announce_entry.attached_interface_hash.clone();
            drop(announce_table);
            
            // Broadcast to all interfaces except the one we received it from
            let interfaces = lock_mutex!(self.interfaces);
            if let Some(raw) = announce_packet.raw() {
                if let Some(ref received_hash) = received_interface_hash {
                    for (interface_hash, interface) in interfaces.iter() {
                        // Skip the interface we received from
                        if interface_hash != received_hash {
                            if interface.is_online() {
                                if let Err(e) = interface.process_outgoing(raw) {
                                    log::warn!("Failed to propagate announce on interface {}: {:?}", 
                                              interface.name(), e);
                                }
                            }
                        }
                    }
                } else {
                    // No received interface, broadcast to all
                    for interface in interfaces.values() {
                        if interface.is_online() {
                            if let Err(e) = interface.process_outgoing(raw) {
                                log::warn!("Failed to propagate announce on interface {}: {:?}", 
                                          interface.name(), e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    
    /// Get hop count to a destination
    pub fn hops_to(&self, destination_hash: &AddressHash) -> Result<Option<u8>> {
        let path_table = lock_mutex!(self.path_table);
        Ok(path_table
            .get(destination_hash)
            .filter(|entry| !entry.is_expired())
            .map(|entry| entry.hops))
    }
    
    /// Expire old path entries
    pub fn expire_paths(&self) -> Result<()> {
        let mut path_table = lock_mutex!(self.path_table);
        path_table.retain(|_, entry| !entry.is_expired());
        Ok(())
    }
    
    /// Clean up old reverse table entries
    pub fn cleanup_reverse_table(&self) -> Result<()> {
        let mut reverse_table = lock_mutex!(self.reverse_table);
        let now = Instant::now().elapsed().as_secs_f64();
        
        // Remove entries older than 60 seconds
        reverse_table.retain(|_, entry| (now - entry.timestamp) < 60.0);
        Ok(())
    }
    
    /// Clean up expired link entries
    pub fn cleanup_links(&self) -> Result<()> {
        let mut link_table = lock_mutex!(self.link_table);
        link_table.retain(|_, entry| !entry.is_proof_expired());
        Ok(())
    }
    
    /// Add a link entry to the link table
    /// link_id is typically the destination_hash
    pub fn add_link_entry(
        &self,
        link_id: AddressHash,
        destination_hash: AddressHash,
        next_hop: AddressHash,
        received_from: AddressHash,
        hops: u8,
    ) -> Result<()> {
        let mut link_table = lock_mutex!(self.link_table);
        let link_entry = LinkEntry::new(destination_hash, next_hop, received_from, hops);
        link_table.insert(link_id, link_entry);
        Ok(())
    }
    
    /// Get a link entry by link_id
    pub fn get_link_entry(&self, link_id: &AddressHash) -> Result<Option<LinkEntry>> {
        let link_table = lock_mutex!(self.link_table);
        Ok(link_table.get(link_id).cloned())
    }
    
    /// Validate a link (mark it as validated)
    pub fn validate_link(&self, link_id: &AddressHash) -> Result<()> {
        let mut link_table = lock_mutex!(self.link_table);
        if let Some(entry) = link_table.get_mut(link_id) {
            entry.validated = true;
            // Extend proof timeout when validated
            entry.proof_timeout = Instant::now().elapsed().as_secs_f64() + 30.0;
        } else {
            return Err(RnsError::LinkNotEstablished);
        }
        Ok(())
    }
    
    /// Expire a specific link
    pub fn expire_link(&self, link_id: &AddressHash) -> Result<()> {
        let mut link_table = lock_mutex!(self.link_table);
        link_table.remove(link_id);
        Ok(())
    }
    
    /// Check if a link exists and is validated
    pub fn has_validated_link(&self, link_id: &AddressHash) -> Result<bool> {
        let link_table = lock_mutex!(self.link_table);
        Ok(link_table.get(link_id)
            .map(|entry| entry.validated && !entry.is_proof_expired())
            .unwrap_or(false))
    }
    
    /// Get all link IDs
    pub fn get_link_ids(&self) -> Result<Vec<AddressHash>> {
        let link_table = lock_mutex!(self.link_table);
        Ok(link_table.keys().cloned().collect())
    }
    
    /// Get link count
    pub fn link_count(&self) -> Result<usize> {
        let link_table = lock_mutex!(self.link_table);
        Ok(link_table.len())
    }
    
    /// Retransmit announces that need retransmission
    pub fn retransmit_announces(&self) -> Result<()> {
        let announce_table = lock_mutex!(self.announce_table);
        let mut to_retransmit = Vec::new();
        
        // Collect announces that need retransmission
        for (dest_hash, entry) in announce_table.iter() {
            if entry.should_retransmit() {
                to_retransmit.push(dest_hash.clone());
            }
        }
        drop(announce_table);
        
        // Retransmit collected announces
        for dest_hash in to_retransmit {
            // Update retry count and timeout
            let mut announce_table = lock_mutex!(self.announce_table);
            if let Some(existing) = announce_table.get_mut(&dest_hash) {
                existing.retries += 1;
                existing.retransmit_timeout = Instant::now().elapsed().as_secs_f64() + 2.0;
            }
            drop(announce_table);
            
            // Propagate the announce
            self.propagate_announce(&dest_hash)?;
        }
        
        Ok(())
    }
    
    /// Rotate packet hashlist (remove old entries)
    pub fn rotate_hashlist(&self) -> Result<()> {
        let mut hashlist = lock_mutex!(self.packet_hashlist);
        
        // Keep only the most recent entries
        while hashlist.len() > self.max_hashlist_size {
            hashlist.pop_front();
        }
        
        Ok(())
    }
    
    /// Execute all periodic maintenance jobs
    pub fn jobs(&self) -> Result<()> {
        // Clean up expired paths
        self.expire_paths()?;
        
        // Clean up expired path requests
        self.cleanup_path_requests()?;
        
        // Clean up expired links
        self.cleanup_links()?;
        
        // Clean up old reverse table entries
        self.cleanup_reverse_table()?;
        
        // Check receipt timeouts
        self.check_receipt_timeouts()?;
        
        // Retransmit announces that need it
        self.retransmit_announces()?;
        
        // Rotate packet hashlist
        self.rotate_hashlist()?;
        
        Ok(())
    }
    
    /// Start the job loop (async, runs in background)
    /// This should be called from an async context
    /// Returns a JoinHandle that can be used to abort the loop
    #[cfg(all(feature = "std", feature = "tokio"))]
    pub fn start_job_loop(self: Arc<Self>, interval_seconds: u64) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_seconds));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = self.jobs() {
                    log::warn!("Error in job loop: {:?}", e);
                }
            }
        })
    }
}

impl Default for Transport {
    fn default() -> Self {
        Self::new()
    }
}
