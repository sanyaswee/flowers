#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_rp::i2c::{Config, I2c, InterruptHandler, Async};
use embassy_rp::peripherals;
use embassy_rp::bind_interrupts;
use embassy_sync::mutex::Mutex;
use static_cell::StaticCell;

use defmt::{info};

use core_logic::i2c_mutex::SharedI2C;
use core_logic::bh1750;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<peripherals::I2C0>;
});

type PicoI2c = I2c<'static, peripherals::I2C0, Async>;
static I2C_BUS: StaticCell<SharedI2C<PicoI2c>> = StaticCell::new();

// const BMP280_ADDR: u8 = 0x76;

/// Task wrapper for BH1750
#[embassy_executor::task]
async fn read_light_intensity(bus: &'static SharedI2C<PicoI2c>) {
    bh1750::read_light_intensity(bus).await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Starting dual sensor reading (BH1750 + BMP280)");

    let sda = p.PIN_16;
    let scl = p.PIN_17;
    let i2c = I2c::new_async(p.I2C0, scl, sda, Irqs, Config::default());

    let shared_i2c = I2C_BUS.init(Mutex::new(i2c));
    spawner.spawn(read_light_intensity(shared_i2c).unwrap());
}