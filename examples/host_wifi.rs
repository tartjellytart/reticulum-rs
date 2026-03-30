use reticulum_rs::interfaces::{Interface, wifi::{WiFiInterface, WiFiConfig, WiFiMode, WiFiDriver}};
use reticulum_rs::interfaces::wifi::host::HostWiFiDriver;
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("=== Reticulum-rs Host WiFi Test ===\n");

    let driver = HostWiFiDriver::new(4242, 4242);

    // Configure WiFi credentials via environment or config file
    // SECURITY: Do not commit real credentials to source control
    let ssid = std::env::var("WIFI_SSID").unwrap_or_else(|_| "ConfigureMe".to_string());
    let password = std::env::var("WIFI_PASSWORD").unwrap_or_else(|_| "ConfigureMe".to_string());
    
    let config = WiFiConfig {
        ssid: ssid.clone(),
        password,
        channel: None,
        mode: WiFiMode::Station,
    };

    let mut wifi = WiFiInterface::new(driver, config.clone(), "host_wifi")?;

    println!("📡 Connecting to WiFi network: {}", config.ssid);
    println!("   (Note: On host OS, WiFi is managed by the system)");
    println!("   (This test will use UDP broadcast on port 4242)\n");

    wifi.start().await?;

    println!("✅ WiFi interface started");
    if let Some(ip) = wifi.driver().get_ip() {
        println!("   Local IP: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
    }
    println!("   Interface: {}", wifi.name());
    println!("   MTU: {} bytes", wifi.mtu());
    println!("   Bitrate: {} Mbps\n", wifi.bitrate() / 1_000_000);

    println!("🔄 Starting main loop...");
    println!("   Sending broadcast packets every 2 seconds");
    println!("   Listening for incoming packets");
    println!("   Press Ctrl+C to exit\n");

    let mut tick = interval(Duration::from_secs(2));
    let mut packet_count = 0u32;

    loop {
        tokio::select! {
            _ = tick.tick() => {
                packet_count += 1;
                let message = format!("Hello from Reticulum-rs #{}", packet_count);
                
                match wifi.send_broadcast(message.as_bytes(), 4242).await {
                    Ok(_) => {
                        println!("📤 Sent: \"{}\" ({} bytes)", message, message.len());
                    }
                    Err(e) => {
                        eprintln!("❌ Send error: {:?}", e);
                    }
                }
            }
            
            result = wifi.read_and_process() => {
                match result {
                    Ok(_) => {
                        if wifi.rxb() > 0 {
                            println!("📥 Received packet ({} bytes total)", wifi.rxb());
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Receive error: {:?}", e);
                    }
                }
            }
        }
        
        if packet_count % 5 == 0 {
            println!("\n📊 Stats:");
            println!("   TX: {} bytes", wifi.txb());
            println!("   RX: {} bytes", wifi.rxb());
            println!("   Online: {}\n", wifi.is_online());
        }
    }
}

