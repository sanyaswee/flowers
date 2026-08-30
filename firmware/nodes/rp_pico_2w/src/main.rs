//! The main firmware file

#![no_std]
#![no_main]

mod wifi;
mod wrappers;

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

static WIFI_TRANSPORT: StaticCell<SharedWifiTransport> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let config = NodeConfig::new(
        otp::get_chipid().unwrap(),
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

    let wifi_tr = WIFI_TRANSPORT.init(Mutex::new(wifi_tr));

    let sda = p.PIN_16;
    let scl = p.PIN_17;
    let i2c = I2c::new_async(p.I2C0, scl, sda, Irqs, Config::default());

    let shared_i2c = I2C_BUS.init(Mutex::new(i2c));

    // Telemetry tasks
    spawner.spawn(read_light_intensity(shared_i2c).unwrap());
    spawner.spawn(read_temp_pressure(shared_i2c).unwrap());
    spawner.spawn(telemetry::gather().unwrap());

    // Network tasks
    spawner.spawn(auto_reconnect(wifi_tr).unwrap());
    spawner.spawn(telemetry_sender(wifi_tr).unwrap());

    info!("Node initialized! Configuration: {}", NODE_CONFIG.get().await);
}