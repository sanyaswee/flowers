//! A driver for the BH1750 light intensity sensor
//! https://www.alldatasheet.com/datasheet-pdf/view/338083/ROHM/BH1750FVI.html

use embedded_hal_async::i2c::I2c;
use embassy_time::Timer;
use defmt::{error};

use crate::i2c_mutex::SharedI2C;
use crate::telemetry_broker::TELEMETRY;

/// I2C address for BH1750FVI
const ADDR: u8 = 0x23;

/// Command to enable the module
const CMD_POWER_ON: u8 = 0x01;
/// Set high resolution mode
const CMD_CONT_HIGH_RES: u8 = 0x10;


/// Convert raw buffer bytes to lux value
fn raw_to_lux(buf: [u8; 2]) -> f32 {
    let raw = ((buf[0] as u16) << 8) | buf[1] as u16;
    raw as f32 / 1.2
}

/// Power on and set resolution
async fn init<I2C, E>(bus: &'static SharedI2C<I2C>) -> Result<(), E>
where
    I2C: I2c<Error = E>,
{
    let mut i2c = bus.lock().await;

    // Power on
    i2c.write(ADDR, &[CMD_POWER_ON]).await?;
    // Set resolution
    i2c.write(ADDR, &[CMD_CONT_HIGH_RES]).await?;

    Ok(())
}

/// Main processing task
pub async fn read_light_intensity<I2C, E>(bus: &'static SharedI2C<I2C>)
where
    I2C: I2c<Error = E>,
{
    if init(bus).await.is_err() {
        error!("Failed to initialize BH1750");
        return;
    }

    Timer::after_millis(200).await;
    let mut buf = [0u8; 2];

    loop {
        let reading = {
            let mut i2c = bus.lock().await;
            i2c.read(ADDR, &mut buf).await
        };

        match reading {
            Ok(_) => {
                let lux = raw_to_lux(buf);
                {
                    let mut t = TELEMETRY.lock().await;
                    t.light_intensity = Some(lux);
                }
            }
            Err(_) => {
                error!("Error reading light intensity")
            }
        };

        Timer::after_secs(1).await;
    }
}