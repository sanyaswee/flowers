//! Wrapped tasks from core crate

use core_logic::SharedI2C;
use core_logic::{bh1750, bmp280, network};

use embassy_rp::gpio::Output;

use crate::PicoI2c;
use crate::wifi::WifiTransport;

/// Task wrapper for BH1750
#[embassy_executor::task]
pub async fn read_light_intensity(bus: &'static SharedI2C<PicoI2c>) {
    bh1750::read_light_intensity(bus).await;
}

/// Task wrapper for BMP280
#[embassy_executor::task]
pub async fn read_temp_pressure(bus: &'static SharedI2C<PicoI2c>) {
    bmp280::read_temp_pressure(bus).await;
}

/// Task wrapper for network tracker
#[embassy_executor::task]
pub async fn track_network(led: Output<'static>) {
    network::track_status(led).await;
}

/// Task wrapper for the consolidated MQTT network manager
#[embassy_executor::task]
pub async fn mqtt_network(transport: WifiTransport, client_id: &'static str) {
    network::mqtt_network_task(transport, client_id).await;
}