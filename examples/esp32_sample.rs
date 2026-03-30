#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_wifi::wifi::{WifiController, WifiDevice};
use embassy_net::{Stack, Config};
use reticulum_rs::interfaces::wifi::{WiFiInterface, WiFiConfig, WiFiMode};
use reticulum_rs::interfaces::wifi::esp32::Esp32WiFiDriver;

#[main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(Default::default());
    
    let (wifi_controller, wifi_device) = esp_wifi::initialize(...).unwrap();
    
    let stack = &*mk_static!(
        Stack<WifiDevice>,
        Stack::new(
            wifi_device,
            Config::dhcpv4(Default::default()),
            mk_static!(StackResources<3>, StackResources::new()),
            seed
        )
    );
    
    let driver = Esp32WiFiDriver::new(wifi_controller, stack);
    
// Configure WiFi credentials via environment or config file
// SECURITY: Do not commit real credentials to source control
    let config = WiFiConfig {
        ssid: option_env!("WIFI_SSID").unwrap_or("ConfigureMe").into(),
        password: option_env!("WIFI_PASSWORD").unwrap_or("ConfigureMe").into(),
        channel: None,
        mode: WiFiMode::Station,
    };
    
    let mut wifi = WiFiInterface::new(driver, config, "esp32_wifi").unwrap();
    wifi.start().await.unwrap();
    
    // Main loop - send broadcast packets
    loop {
        let packet = b"Hello from ESP32!";
        wifi.read_and_process().await.ok();
        embassy_time::Timer::after_millis(1000).await;
    }
}