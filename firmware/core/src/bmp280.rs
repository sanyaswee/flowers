//! A driver for the BMP280 pressure / temperature sensor
//! https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bmp280-ds001.pdf

use embedded_hal_async::i2c::I2c;
use embassy_time::Timer;
use defmt::{error, info};
use crate::i2c_mutex::SharedI2C;
use crate::telemetry_broker::TELEMETRY;

/// I2C address for BMP280
const ADDR: u8 = 0x76;

/// BMP280 registers
const ID: u8 = 0xD0;
const CALIB: u8 = 0x88;
const CTRL_MEAS: u8 = 0xF4;
const DATA_START: u8 = 0xF7;

/// BMP280 config
/// 001 => temperature oversampling x1
/// 001 => pressure oversampling x1
/// 11 => normal mode
const CONFIG: u8 = 0b_001_001_11;

/// BMP280 calibration data
struct Calibration {
    /// Temperature calibration
    dig_t1: u16, dig_t2: i16, dig_t3: i16,
    /// Pressure calibration
    dig_p1: u16, dig_p2: i16, dig_p3: i16, dig_p4: i16,
    dig_p5: i16, dig_p6: i16, dig_p7: i16, dig_p8: i16, dig_p9: i16,
}

/// Parse raw calibration data
fn parse_calibration(buf: &[u8; 24]) -> Calibration {
    Calibration {
        dig_t1: u16::from_le_bytes([buf[0], buf[1]]),
        dig_t2: i16::from_le_bytes([buf[2], buf[3]]),
        dig_t3: i16::from_le_bytes([buf[4], buf[5]]),
        dig_p1: u16::from_le_bytes([buf[6], buf[7]]),
        dig_p2: i16::from_le_bytes([buf[8], buf[9]]),
        dig_p3: i16::from_le_bytes([buf[10], buf[11]]),
        dig_p4: i16::from_le_bytes([buf[12], buf[13]]),
        dig_p5: i16::from_le_bytes([buf[14], buf[15]]),
        dig_p6: i16::from_le_bytes([buf[16], buf[17]]),
        dig_p7: i16::from_le_bytes([buf[18], buf[19]]),
        dig_p8: i16::from_le_bytes([buf[20], buf[21]]),
        dig_p9: i16::from_le_bytes([buf[22], buf[23]]),
    }
}

/// Compensation math
fn compensate(raw_t: i32, raw_p: i32, calibration: &Calibration) -> (f32, f32) {
    // Temperature compensation
    let var1 = (((raw_t >> 3) - ((calibration.dig_t1 as i32) << 1)) * (calibration.dig_t2 as i32)) >> 11;
    let var2 = (((((raw_t >> 4) - (calibration.dig_t1 as i32)) * ((raw_t >> 4) - (calibration.dig_t1 as i32))) >> 12) * (calibration.dig_t3 as i32)) >> 14;
    let t_fine = var1 + var2;
    let temp_c = ((t_fine * 5 + 128) >> 8) as f32 / 100.0;

    // Pressure compensation (64-bit precision)
    let mut p_var1 = (t_fine as i64) - 128000;
    let mut p_var2 = p_var1 * p_var1 * (calibration.dig_p6 as i64);
    p_var2 = p_var2 + ((p_var1 * (calibration.dig_p5 as i64)) << 17);
    p_var2 = p_var2 + ((calibration.dig_p4 as i64) << 35);
    p_var1 = ((p_var1 * p_var1 * (calibration.dig_p3 as i64)) >> 8) + ((p_var1 * (calibration.dig_p2 as i64)) << 12);
    p_var1 = (((1i64 << 47) + p_var1) * (calibration.dig_p1 as i64)) >> 33;

    let mut pressure_hpa = 0.0;
    if p_var1 != 0 {
        let mut p = 1048576i64 - (raw_p as i64);
        p = (((p << 31) - p_var2) * 3125) / p_var1;
        p_var1 = ((calibration.dig_p9 as i64) * (p >> 13) * (p >> 13)) >> 25;
        p_var2 = ((calibration.dig_p8 as i64) * p) >> 19;
        p = ((p + p_var1 + p_var2) >> 8) + ((calibration.dig_p7 as i64) << 4);

        pressure_hpa = (p as f32) / 256.0 / 100.0;
    }

    (temp_c, pressure_hpa)
}

/// Initialize, verify ID, configure, get calibration
async fn init<I2C, E>(bus: &'static SharedI2C<I2C>) -> Option<Calibration>
where
    I2C: I2c<Error = E>,
{
    let mut i2c = bus.lock().await;

    // Verify the connected peripheral is BMP280
    let mut id_buf = [0u8; 1];
    if i2c.write_read(ADDR, &[ID], &mut id_buf).await.is_err() {
        error!("Error reading from BMP280");
        return None;
    }

    if id_buf[0] != 0x58 {
        error!("ID does not match to BMP280");
        return None;
    }

    // Get calibration data
    let mut calibration_buf = [0u8; 24];
    if i2c.write_read(ADDR, &[CALIB], &mut calibration_buf).await.is_err() {
        error!("Error reading from BMP280");
        return None;
    }

    let calibration = parse_calibration(&calibration_buf);

    // Configure the sensor
    if i2c.write(ADDR, &[CTRL_MEAS, CONFIG]).await.is_err() {
        error!("Failed to configure BMP280");
        return None;
    }

    Some(calibration)

}

/// Main processing task
pub async fn read_temp_pressure<I2C, E>(bus: &'static SharedI2C<I2C>)
where
    I2C: I2c<Error = E>,
{
    // Read calibration
    let calibration = match init(bus).await {
        Some(c) => {
            info!("BMP280 initialized");
            c
        }
        None => {
            error!("Failed to initilize BMP280");
            return;
        }
    };

    Timer::after_millis(200).await;

    let mut data_buf = [0u8; 6];
    loop {
        let result = {
            let mut i2c = bus.lock().await;
            i2c.write_read(ADDR, &[DATA_START], &mut data_buf).await
        };

        match result {
            Ok(_) => {
                // Construct raw 20-bit values
                let raw_p = ((data_buf[0] as i32) << 12) | ((data_buf[1] as i32) << 4) | ((data_buf[2] as i32) >> 4);
                let raw_t = ((data_buf[3] as i32) << 12) | ((data_buf[4] as i32) << 4) | ((data_buf[5] as i32) >> 4);

                // Apply math
                let (temp_c, pressure_hpa) = compensate(raw_t, raw_p, &calibration);
                {
                    let mut t = TELEMETRY.lock().await;
                    t.temperature = Some(temp_c);
                    t.pressure = Some(pressure_hpa);
                }
            }
            Err(_) => {
                error!("Error reading from BMP280");
            }
        }

        Timer::after_secs(1).await;
    }
}
