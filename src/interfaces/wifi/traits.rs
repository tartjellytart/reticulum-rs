use crate::error::Result;

pub trait WiFiDriver: Send + Sync {
    async fn connect(&mut self, ssid: &str, password: &str) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    fn is_connected(&self) -> bool;
    fn get_ip(&self) -> Option<[u8; 4]>;
    async fn send_udp(&self, data: &[u8], addr: ([u8; 4], u16)) -> Result<()>;
    async fn recv_udp(&self, buffer: &mut [u8]) -> Result<Option<(usize, ([u8; 4], u16))>>;
    fn get_mac(&self) -> [u8; 6];
}

#[derive(Debug, Clone)]
pub struct WiFiConfig {
    pub ssid: String,
    pub password: String,
    pub channel: Option<u8>,
    pub mode: WiFiMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WiFiMode {
    Station,
    AccessPoint,
    Mixed,
}

