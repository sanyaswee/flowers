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
use embassy_rp::i2c::{Async, Config, I2c, InterruptHandler};
use embassy_rp::peripherals;
use embassy_rp::otp;
use embassy_sync::mutex::Mutex;

use static_cell::StaticCell;

use core_logic::NODE_CONFIG;
use core_logic::SharedI2C;
use core_logic::telemetry_broker;

use shared::node_config::{NodeConfig, TelemetryCapabilities, WaterTankDetection};

use wrappers::*;
use wifi::WifiTransport;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<peripherals::I2C0>;
});

type PicoI2c = I2c<'static, peripherals::I2C0, Async>;
static I2C_BUS: StaticCell<SharedI2C<PicoI2c>> = StaticCell::new();

static WIFI_TRANSPORT: StaticCell<WifiTransport> = StaticCell::new();

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

    let wifi_tr = wifi::init(
        spawner, p.PIO0, p.PIN_23, p.PIN_24, p.PIN_25, p.PIN_29, p.DMA_CH0,
    ).await;
    let wifi_tr = WIFI_TRANSPORT.init(wifi_tr);

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

    spawner.spawn(read_light_intensity(shared_i2c).unwrap());
    spawner.spawn(read_temp_pressure(shared_i2c).unwrap());
    
    spawner.spawn(telemetry_broker::gather().unwrap());
    spawner.spawn(telemetry_sender(wifi_tr).unwrap());

    info!("Node initialized! Configuration: {}", NODE_CONFIG.get().await);
}