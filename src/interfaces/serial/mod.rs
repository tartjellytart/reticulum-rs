mod traits;

pub use traits::{SerialDriver, SerialConfig, StopBits, Parity, FlowControl};

use crate::error::{RnsError, Result};
use crate::interfaces::{Interface, InterfaceMode, InterfaceStateFields, hdlc::{HdlcDecoder, HdlcEncoder}};
use crate::hash::{AddressHash, HashGenerator};

pub struct SerialInterface<D: SerialDriver> {
    serial_driver: D,
    serial_config: SerialConfig,
    state: InterfaceStateFields,
    hdlc_decoder: HdlcDecoder,
}

impl<D: SerialDriver> SerialInterface<D> {
    /// Create a new Serial interface
    /// 
    /// Initializes a serial port connection with HDLC framing for reliable packet transmission.
    /// The interface is created in offline state and must be started with [`start()`](Self::start).
    /// 
    /// # Arguments
    /// * `serial_driver` - The underlying serial port driver implementation
    /// * `serial_config` - Serial port configuration (baud rate, parity, etc.)
    /// * `name` - Human-readable name for this interface (e.g., "COM1" or "ttyUSB0")
    /// 
    /// # Returns
    /// A new SerialInterface instance ready to be started
    pub fn new(serial_driver: D, serial_config: SerialConfig, name: &str) -> Result<Self> {
        let bitrate = serial_config.baud_rate as u64;
        Ok(Self {
            serial_driver,
            serial_config,
            state: InterfaceStateFields::new(name, bitrate, 500),
            hdlc_decoder: HdlcDecoder::new(),
        })
    }
    
    /// Start the serial interface
    /// 
    /// Opens the serial port and marks the interface as online.
    /// Must be called before the interface can transmit or receive data.
    /// 
    /// # Errors
    /// Returns error if the serial port cannot be opened (already in use, permission denied, etc.)
    pub fn start(&mut self) -> Result<()> {
        self.serial_driver.open()?;
        self.state.online = true;
        Ok(())
    }
    
    /// Stop the serial interface
    /// 
    /// Closes the serial port and marks the interface as offline.
    /// Any queued data is sent before closing.
    /// 
    /// # Errors
    /// Returns error if the serial port cannot be closed cleanly
    pub fn stop(&mut self) -> Result<()> {
        self.serial_driver.close()?;
        self.state.online = false;
        Ok(())
    }
    
    /// Read and process incoming data from the serial port
    /// 
    /// Reads available data from the serial port and processes it through the HDLC decoder.
    /// Calls [`process_incoming()`](Interface::process_incoming) for each complete frame received.
    /// 
    /// This method should be called regularly (e.g., in a main loop or interrupt handler)
    /// to prevent the input buffer from overflowing.
    /// 
    /// # Errors
    /// Returns error if serial read fails or HDLC frame processing fails
    pub fn read_and_process(&mut self) -> Result<()> {
        let mut buffer = [0u8; 256];
        let bytes_read = self.serial_driver.read(&mut buffer)?;
        
        if bytes_read > 0 {
            for &byte in &buffer[..bytes_read] {
                if let Some(frame_data) = self.hdlc_decoder.process_byte(byte)? {
                    self.state.rx_bytes += frame_data.len() as u64;
                    self.process_incoming(&frame_data)?;
                }
            }
        }
        
        Ok(())
    }
    
}

impl<D: SerialDriver> Interface for SerialInterface<D> {
    fn process_incoming(&mut self, _data: &[u8]) -> Result<()> {
        Ok(())
    }
    
    fn process_outgoing(&mut self, data: &[u8]) -> Result<()> {
        if !self.state.online {
            return Err(RnsError::ConnectionError);
        }
        
        let framed = HdlcEncoder::encode(data);
        self.serial_driver.write(&framed)?;
        self.serial_driver.flush()?;
        self.state.tx_bytes += data.len() as u64;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.state.name
    }
    
    fn mode(&self) -> InterfaceMode {
        InterfaceMode::PointToPoint
    }
    
    fn is_online(&self) -> bool {
        self.state.online
    }
    
    fn bitrate(&self) -> u64 {
        self.state.bitrate
    }
    
    fn mtu(&self) -> usize {
        self.state.mtu
    }
    
    fn rxb(&self) -> u64 {
        self.state.rx_bytes
    }
    
    fn txb(&self) -> u64 {
        self.state.tx_bytes
    }
    
    fn interface_hash(&self) -> AddressHash {
        let hash = HashGenerator::new()
            .chain_update(self.state.name.as_bytes())
            .chain_update(&self.serial_config.baud_rate.to_le_bytes())
            .finalize();
        AddressHash::new_from_hash(hash.as_bytes())
    }
}

