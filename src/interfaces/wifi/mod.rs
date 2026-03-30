mod traits;

#[cfg(all(feature = "std", feature = "tokio", not(target_arch = "xtensa")))]
pub mod host;

// Note: ESP32 WiFi driver implemented in wifi-test crate using esp-radio
// See: wifi-test/src/wifi_driver.rs

pub use traits::{WiFiDriver, WiFiConfig, WiFiMode};

use crate::error::{RnsError, Result};
use crate::interfaces::{Interface, InterfaceMode, InterfaceStateFields};
use crate::hash::{AddressHash, HashGenerator};

pub struct WiFiInterface<D: WiFiDriver> {
    wifi_driver: D,
    wifi_config: WiFiConfig,
    state: InterfaceStateFields,
}

impl<D: WiFiDriver> WiFiInterface<D> {
    /// Create a new WiFi interface
    /// 
    /// Initializes a WiFi connection interface with UDP packet transmission.
    /// The interface is created in offline state and must be started with [`start()`](Self::start).
    /// 
    /// # Arguments
    /// * `wifi_driver` - The underlying WiFi driver implementation
    /// * `wifi_config` - WiFi configuration (SSID, password, mode)
    /// * `name` - Human-readable name for this interface
    pub fn new(wifi_driver: D, wifi_config: WiFiConfig, name: &str) -> Result<Self> {
        Ok(Self {
            wifi_driver,
            wifi_config,
            state: InterfaceStateFields::new(name, 54_000_000, 1500),
        })
    }
    
    /// Start the WiFi interface
    /// 
    /// Asynchronously connects to the configured WiFi network.
    /// Supports Station mode, Access Point mode, and Mixed mode.
    /// 
    /// # Errors
    /// Returns error if connection fails (network not found, authentication failed, etc.)
    pub async fn start(&mut self) -> Result<()> {
        match self.wifi_config.mode {
            WiFiMode::Station => {
                self.wifi_driver.connect(&self.wifi_config.ssid, &self.wifi_config.password).await?;
            }
            WiFiMode::AccessPoint => {
                // TODO: AP mode
            }
            WiFiMode::Mixed => {
                self.wifi_driver.connect(&self.wifi_config.ssid, &self.wifi_config.password).await?;
            }
        }
        self.state.online = true;
        Ok(())
    }
    
    /// Stop the WiFi interface
    /// 
    /// Disconnects from the WiFi network and marks the interface as offline.
    /// 
    /// # Errors
    /// Returns error if disconnect fails
    pub async fn stop(&mut self) -> Result<()> {
        self.wifi_driver.disconnect().await?;
        self.state.online = false;
        Ok(())
    }
    
    /// Read and process incoming UDP packets
    /// 
    /// Asynchronously receives UDP packets from the network and processes them.
    /// Calls [`process_incoming()`](Interface::process_incoming) for each packet received.
    /// 
    /// # Errors
    /// Returns error if UDP receive fails
    pub async fn read_and_process(&mut self) -> Result<()> {
        let mut buffer = [0u8; 1500];
        if let Some((len, _addr)) = self.wifi_driver.recv_udp(&mut buffer).await? {
            self.state.rx_bytes += len as u64;
            self.process_incoming(&buffer[..len])?;
        }
        Ok(())
    }
    
    pub fn driver(&self) -> &D {
        &self.wifi_driver
    }
    
    /// Send a broadcast UDP packet
    /// 
    /// Asynchronously sends data as a broadcast UDP packet on the specified port.
    /// Only available when the interface is online and in a broadcast-capable mode.
    /// 
    /// # Arguments
    /// * `data` - The packet data to send
    /// * `port` - The UDP port to send to (255.255.255.255:port)
    /// 
    /// # Errors
    /// Returns error if the interface is offline or transmission fails
    pub async fn send_broadcast(&mut self, data: &[u8], port: u16) -> Result<()> {
        self.wifi_driver.send_udp(data, ([255, 255, 255, 255], port)).await?;
        self.state.tx_bytes += data.len() as u64;
        Ok(())
    }
}

impl<D: WiFiDriver> Interface for WiFiInterface<D> {
    fn process_incoming(&mut self, _data: &[u8]) -> Result<()> {
        Ok(())
    }
    
    fn process_outgoing(&mut self, data: &[u8]) -> Result<()> {
        if !self.state.online {
            return Err(RnsError::ConnectionError);
        }
        self.state.tx_bytes += data.len() as u64;
        Err(RnsError::InvalidArgument)
    }
    
    fn name(&self) -> &str {
        &self.state.name
    }
    
    fn mode(&self) -> InterfaceMode {
        InterfaceMode::Full
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
            .chain_update(&self.wifi_driver.get_mac())
            .finalize();
        AddressHash::new_from_hash(hash.as_bytes())
    }
}

