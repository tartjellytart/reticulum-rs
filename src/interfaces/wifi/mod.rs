mod traits;

#[cfg(not(target_arch = "xtensa"))]
pub mod host;

pub use traits::{WiFiDriver, WiFiConfig, WiFiMode};

use crate::error::{RnsError, Result};
use crate::interfaces::{Interface, InterfaceMode};
use crate::hash::{AddressHash, HashGenerator};

pub struct WiFiInterface<D: WiFiDriver> {
    driver: D,
    config: WiFiConfig,
    name: String,
    online: bool,
    rxb: u64,
    txb: u64,
    bitrate: u64,
    mtu: usize,
}

impl<D: WiFiDriver> WiFiInterface<D> {
    pub fn new(driver: D, config: WiFiConfig, name: &str) -> Result<Self> {
        Ok(Self {
            driver,
            config,
            name: name.to_string(),
            online: false,
            rxb: 0,
            txb: 0,
            bitrate: 54_000_000,
            mtu: 1500,
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        match self.config.mode {
            WiFiMode::Station => {
                self.driver.connect(&self.config.ssid, &self.config.password).await?;
            }
            WiFiMode::AccessPoint => {
                // TODO: AP mode
            }
            WiFiMode::Mixed => {
                self.driver.connect(&self.config.ssid, &self.config.password).await?;
            }
        }
        self.online = true;
        Ok(())
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        self.driver.disconnect().await?;
        self.online = false;
        Ok(())
    }
    
    pub async fn read_and_process(&mut self) -> Result<()> {
        let mut buffer = [0u8; 1500];
        if let Some((len, _addr)) = self.driver.recv_udp(&mut buffer).await? {
            self.rxb += len as u64;
            self.process_incoming(&buffer[..len])?;
        }
        Ok(())
    }
    
    pub fn driver(&self) -> &D {
        &self.driver
    }
    
    pub async fn send_broadcast(&mut self, data: &[u8], port: u16) -> Result<()> {
        self.driver.send_udp(data, ([255, 255, 255, 255], port)).await?;
        self.txb += data.len() as u64;
        Ok(())
    }
}

impl<D: WiFiDriver> Interface for WiFiInterface<D> {
    fn process_incoming(&mut self, _data: &[u8]) -> Result<()> {
        Ok(())
    }
    
    fn process_outgoing(&mut self, data: &[u8]) -> Result<()> {
        if !self.online {
            return Err(RnsError::ConnectionError);
        }
        self.txb += data.len() as u64;
        Err(RnsError::InvalidArgument)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn mode(&self) -> InterfaceMode {
        InterfaceMode::Full
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
            .chain_update(&self.driver.get_mac())
            .finalize();
        AddressHash::new_from_hash(hash.as_bytes())
    }
}

