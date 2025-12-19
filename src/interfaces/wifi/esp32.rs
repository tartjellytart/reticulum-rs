use crate::error::{RnsError, Result};
use crate::interfaces::wifi::traits::WiFiDriver;
use esp_wifi::wifi::{WifiController, WifiDevice, ClientConfiguration, Configuration};
use embassy_net::{Stack, IpAddress, Ipv4Address};

pub struct Esp32WiFiDriver {
    controller: WifiController<'static>,
    stack: &'static Stack<WifiDevice<'static>>,
}

impl Esp32WiFiDriver {
    pub fn new(
        controller: WifiController<'static>,
        stack: &'static Stack<WifiDevice<'static>>,
    ) -> Self {
        Self { controller, stack }
    }
}

impl WiFiDriver for Esp32WiFiDriver {
    async fn connect(&mut self, ssid: &str, password: &str) -> Result<()> {
        let config = Configuration::Client(ClientConfiguration {
            ssid: ssid.into(),
            password: password.into(),
            ..Default::default()
        });
        
        self.controller.set_configuration(&config)
            .map_err(|_| RnsError::ConfigError)?;
        
        self.controller.start().await
            .map_err(|_| RnsError::ConnectionError)?;
        
        self.controller.connect().await
            .map_err(|_| RnsError::ConnectionError)?;
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        self.controller.disconnect().await
            .map_err(|_| RnsError::ConnectionError)
    }
    
    fn is_connected(&self) -> bool {
        self.stack.is_link_up()
    }
    
    fn get_ip(&self) -> Option<[u8; 4]> {
        self.stack.config_v4()?.address.address().0.into()
    }
    
    async fn send_udp(&self, data: &[u8], addr: ([u8; 4], u16)) -> Result<()> {
        use embassy_net::udp::UdpSocket;
        
        let mut socket = UdpSocket::new(self.stack, &mut [0; 1024], &mut [0; 1024]);
        socket.bind(0).unwrap(); // Bind to any port
        
        let remote = (Ipv4Address::from_bytes(&addr.0), addr.1);
        socket.send_to(data, remote).await
            .map_err(|_| RnsError::NetworkError)?;
        
        Ok(())
    }
    
    async fn recv_udp(&self, buffer: &mut [u8]) -> Result<Option<(usize, ([u8; 4], u16))>> {
        // This needs a persistent socket - in real impl, create socket once
        // For now, return None (would need refactoring)
        Ok(None)
    }
    
    fn get_mac(&self) -> [u8; 6] {
        // Get MAC from ESP32 hardware
        [0; 6] // Placeholder
    }
}