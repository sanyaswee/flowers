//! The main firmware file

#![no_std]
#![no_main]

mod wifi;
mod wrappers;

use core::fmt::Write as _;

use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Output, Level};
use embassy_rp::i2c::{Async, Config, I2c, InterruptHandler};
use embassy_rp::peripherals;
use embassy_rp::otp;
use embassy_sync::mutex::Mutex;

use heapless::String;

use static_cell::StaticCell;

use core_logic::NODE_CONFIG;
use core_logic::SharedI2C;
use core_logic::telemetry;

use shared::node_config::{NodeConfig, TelemetryCapabilities, WaterTankDetection};

use wrappers::*;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<peripherals::I2C0>;
});

type PicoI2c = I2c<'static, peripherals::I2C0, Async>;
static I2C_BUS: StaticCell<SharedI2C<PicoI2c>> = StaticCell::new();
static CLIENT_ID: StaticCell<String<32>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let chip_id = otp::get_chipid().unwrap();

    let config = NodeConfig::new(
        chip_id,
        2,
        WaterTankDetection::None,
        TelemetryCapabilities::new(true, false, true, true)
    );

    NODE_CONFIG.init(config).unwrap();

    // Setup network status LED before any network operations
    let network_status_led = Output::new(p.PIN_18, Level::High);
    spawner.spawn(track_network(network_status_led).unwrap());

    let wifi_tr = wifi::init(
        spawner, p.PIO0, p.PIN_23, p.PIN_24, p.PIN_25, p.PIN_29, p.DMA_CH0,
    ).await;

    if let Some(config) = wifi_tr.stack.config_v4() {
        let ip = config.address.address().octets();
        info!(
            "Network configured! IP address: {}.{}.{}.{}",
            ip[0], ip[1], ip[2], ip[3]
        );
    }

    let sda = p.PIN_16;
    let scl = p.PIN_17;
    let i2c = I2c::new_async(p.I2C0, scl, sda, Irqs, Config::default());

    let shared_i2c = I2C_BUS.init(Mutex::new(i2c));

    // Telemetry tasks
    spawner.spawn(read_light_intensity(shared_i2c).unwrap());
    spawner.spawn(read_temp_pressure(shared_i2c).unwrap());
    spawner.spawn(telemetry::gather().unwrap());

    // Network tasks
    // Pass ownership of the transport directly to the MQTT manager
    let client_id_buf = CLIENT_ID.init(String::new());
    write!(client_id_buf, "pico-{:016x}", chip_id).unwrap();
    spawner.spawn(mqtt_network(wifi_tr, client_id_buf.as_str()).unwrap());

    info!("Node initialized! Configuration: {}", NODE_CONFIG.get().await);
}