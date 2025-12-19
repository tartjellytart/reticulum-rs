use crate::error::{Result, RnsError};
use crate::interfaces::wifi::WiFiDriver;
use tokio::net::UdpSocket;
use std::sync::Arc;

pub struct HostWiFiDriver {
    socket: Option<Arc<UdpSocket>>,
    local_addr: Option<([u8; 4], u16)>,
    broadcast_addr: ([u8; 4], u16),
    connected: bool,
}

impl HostWiFiDriver {
    pub fn new(_bind_port: u16, broadcast_port: u16) -> Self {
        Self {
            socket: None,
            local_addr: None,
            broadcast_addr: ([255, 255, 255, 255], broadcast_port),
            connected: false,
        }
    }
}

impl WiFiDriver for HostWiFiDriver {
    async fn connect(&mut self, _ssid: &str, _password: &str) -> Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await
            .map_err(|e| RnsError::IoError(e.to_string()))?;
        
        socket.set_broadcast(true)
            .map_err(|e| RnsError::IoError(e.to_string()))?;
        
        let local = socket.local_addr()
            .map_err(|e| RnsError::IoError(e.to_string()))?;
        
        if let std::net::SocketAddr::V4(addr) = local {
            let ip = addr.ip().octets();
            let port = addr.port();
            self.local_addr = Some((ip, port));
        }
        
        self.socket = Some(Arc::new(socket));
        self.connected = true;
        
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.socket = None;
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn get_ip(&self) -> Option<[u8; 4]> {
        self.local_addr.map(|(ip, _)| ip)
    }

    async fn send_udp(&self, data: &[u8], addr: ([u8; 4], u16)) -> Result<()> {
        if let Some(socket) = &self.socket {
            let dest = format!("{}.{}.{}.{}:{}", addr.0[0], addr.0[1], addr.0[2], addr.0[3], addr.1);
            socket.send_to(data, dest).await
                .map_err(|e| RnsError::IoError(e.to_string()))?;
            Ok(())
        } else {
            Err(RnsError::ConnectionError)
        }
    }

    async fn recv_udp(&self, buffer: &mut [u8]) -> Result<Option<(usize, ([u8; 4], u16))>> {
        if let Some(socket) = &self.socket {
            tokio::select! {
                result = socket.recv_from(buffer) => {
                    match result {
                        Ok((len, addr)) => {
                            if let std::net::SocketAddr::V4(v4_addr) = addr {
                                let ip = v4_addr.ip().octets();
                                let port = v4_addr.port();
                                Ok(Some((len, (ip, port))))
                            } else {
                                Ok(None)
                            }
                        }
                        Err(_) => Ok(None),
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                    Ok(None)
                }
            }
        } else {
            Err(RnsError::ConnectionError)
        }
    }

    fn get_mac(&self) -> [u8; 6] {
        [0x02, 0x00, 0x00, 0x00, 0x00, 0x01]
    }
}

