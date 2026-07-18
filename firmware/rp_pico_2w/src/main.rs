#![no_std]
#![no_main]

use embedded_hal_async::i2c::{I2c as _};

use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_rp::i2c::{I2c, InterruptHandler, Config};
use embassy_rp::peripherals;
use embassy_rp::bind_interrupts;
use embassy_time::Timer;

use defmt::{info, error};

bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<peripherals::I2C0>;
});

const BMP280_ADDR: u8 = 0x76;

const BH1750FVI_ADDR: u8 = 0x23;

const CMD_POWER_ON: u8 = 0x01;
const CMD_CONT_HIGH_RES: u8 = 0x10;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello world!");

    let sda = p.PIN_16;
    let scl = p.PIN_17;
    let mut i2c = I2c::new_async(p.I2C0, scl, sda, Irqs, Config::default());

    if let Err(e) = i2c.write(BH1750FVI_ADDR, &[CMD_POWER_ON]).await {
        error!("Failed to power on BH1750: {:?}", e);
        return;
    }

    if let Err(e) = i2c.write(BH1750FVI_ADDR, &[CMD_CONT_HIGH_RES]).await {
        error!("Failed to set mode: {:?}", e);
        return;
    }

    Timer::after_millis(200).await;
    info!("BH1750FVI initialized");

    let mut buf = [0u8; 2];
    loop {
        match i2c.read(BH1750FVI_ADDR, &mut buf).await {
            Ok(_) => {
                // Combine the two bytes (High Byte << 8 | Low Byte)
                let raw_val = ((buf[0] as u16) << 8) | (buf[1] as u16);

                // The formula provided by the BH1750 datasheet is (Raw Value / 1.2)
                let lux = (raw_val as f32) / 1.2;
                info!("Light level: {} lx", lux);
            }
            Err(e) => {
                error!("I2C read error: {:?}", e);
            }
        }

        Timer::after_millis(1000).await;
    }
}
