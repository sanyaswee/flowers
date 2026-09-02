//! Wrapped tasks from core crate

use core_logic::adc::AdcProvider;
use core_logic::SharedI2C;
use core_logic::{bh1750, bmp280, network, water_tank};

use embassy_rp::adc::{Adc, Async, Channel as AdcChannel};
use embassy_rp::gpio::Output;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;

use crate::PicoI2c;
use crate::wifi::WifiTransport;

pub struct PicoAdc(pub Adc<'static, Async>);

impl AdcProvider<AdcChannel<'static>, u16> for PicoAdc {
    type Error = ();

    async fn read(&mut self, pin: &mut AdcChannel<'static>) -> Result<u16, Self::Error> {
        Ok(self.0.read(pin).await.unwrap())
    }

    fn max_value(&self) -> u16 {
        4095 // 12 bit maximum
    }
}

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

// Task wrapper for the water level reader
#[embassy_executor::task]
pub async fn read_water_level(
    bus: &'static Mutex<NoopRawMutex, PicoAdc>,
    pin: AdcChannel<'static>,
    power_pin: Output<'static>,
) {
    water_tank::read_level(bus, pin, power_pin).await;
}