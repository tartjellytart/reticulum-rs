mod traits;

pub use traits::{SerialDriver, SerialConfig, StopBits, Parity, FlowControl};

use crate::error::{RnsError, Result};
use crate::interfaces::{Interface, InterfaceMode};
use crate::hash::{AddressHash, HashGenerator};

const HDLC_FLAG: u8 = 0x7E;
const HDLC_ESC: u8 = 0x7D;
const HDLC_ESC_MASK: u8 = 0x20;

pub struct SerialInterface<D: SerialDriver> {
    driver: D,
    config: SerialConfig,
    name: String,
    online: bool,
    rxb: u64,
    txb: u64,
    bitrate: u64,
    mtu: usize,
    frame_buffer: Vec<u8>,
    in_frame: bool,
    escape: bool,
}

impl<D: SerialDriver> SerialInterface<D> {
    pub fn new(driver: D, config: SerialConfig, name: &str) -> Result<Self> {
        let bitrate = config.baud_rate as u64;
        Ok(Self {
            driver,
            config,
            name: name.to_string(),
            online: false,
            rxb: 0,
            txb: 0,
            bitrate,
            mtu: 500,
            frame_buffer: Vec::new(),
            in_frame: false,
            escape: false,
        })
    }
    
    pub fn start(&mut self) -> Result<()> {
        self.driver.open()?;
        self.online = true;
        Ok(())
    }
    
    pub fn stop(&mut self) -> Result<()> {
        self.driver.close()?;
        self.online = false;
        Ok(())
    }
    
    pub fn read_and_process(&mut self) -> Result<()> {
        let mut buffer = [0u8; 256];
        let bytes_read = self.driver.read(&mut buffer)?;
        
        if bytes_read > 0 {
            for &byte in &buffer[..bytes_read] {
                self.process_hdlc_byte(byte)?;
            }
        }
        
        Ok(())
    }
    
    fn process_hdlc_byte(&mut self, byte: u8) -> Result<()> {
        if byte == HDLC_FLAG {
            if self.in_frame && !self.frame_buffer.is_empty() {
                // Complete frame received
                let frame_data = self.frame_buffer.clone();
                self.rxb += frame_data.len() as u64;
                self.frame_buffer.clear();
                self.process_incoming(&frame_data)?;
            }
            self.in_frame = true;
            self.escape = false;
        } else if self.in_frame {
            if byte == HDLC_ESC {
                self.escape = true;
            } else {
                if self.escape {
                    let unescaped = byte ^ HDLC_ESC_MASK;
                    self.frame_buffer.push(unescaped);
                    self.escape = false;
                } else {
                    self.frame_buffer.push(byte);
                }
            }
        }
        
        Ok(())
    }
    
    fn escape_hdlc(&self, data: &[u8]) -> Vec<u8> {
        let mut escaped = Vec::with_capacity(data.len() + 2);
        escaped.push(HDLC_FLAG);
        
        for &byte in data {
            if byte == HDLC_FLAG || byte == HDLC_ESC {
                escaped.push(HDLC_ESC);
                escaped.push(byte ^ HDLC_ESC_MASK);
            } else {
                escaped.push(byte);
            }
        }
        
        escaped.push(HDLC_FLAG);
        escaped
    }
    
}

impl<D: SerialDriver> Interface for SerialInterface<D> {
    fn process_incoming(&mut self, _data: &[u8]) -> Result<()> {
        Ok(())
    }
    
    fn process_outgoing(&mut self, data: &[u8]) -> Result<()> {
        if !self.online {
            return Err(RnsError::ConnectionError);
        }
        
        let framed = self.escape_hdlc(data);
        self.driver.write(&framed)?;
        self.driver.flush()?;
        self.txb += data.len() as u64;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn mode(&self) -> InterfaceMode {
        InterfaceMode::PointToPoint
    }
    
    fn is_online(&self) -> bool {
        self.online
    }
    
    fn bitrate(&self) -> u64 {
        self.bitrate
    }
    
    fn mtu(&self) -> usize {
        self.mtu
    }
    
    fn rxb(&self) -> u64 {
        self.rxb
    }
    
    fn txb(&self) -> u64 {
        self.txb
    }
    
    fn interface_hash(&self) -> AddressHash {
        let hash = HashGenerator::new()
            .chain_update(self.name.as_bytes())
            .chain_update(&self.config.baud_rate.to_le_bytes())
            .finalize();
        AddressHash::new_from_hash(hash.as_bytes())
    }
}

