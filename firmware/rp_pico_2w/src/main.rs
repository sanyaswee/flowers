#![no_std]
#![no_main]

use embedded_hal_async::i2c::I2c as _;

use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_rp::i2c::{Config, I2c, InterruptHandler};
use embassy_rp::peripherals;
use embassy_rp::bind_interrupts;
use embassy_time::Timer;

use defmt::{error, info};

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
    info!("Starting dual sensor reading (BH1750 + BMP280)");

    let sda = p.PIN_16;
    let scl = p.PIN_17;
    let mut i2c = I2c::new_async(p.I2C0, scl, sda, Irqs, Config::default());

    // --- 1. Initialize BH1750FVI ---
    if let Err(e) = i2c.write(BH1750FVI_ADDR, &[CMD_POWER_ON]).await {
        error!("Failed to power on BH1750: {:?}", e);
        return;
    }

    if let Err(e) = i2c.write(BH1750FVI_ADDR, &[CMD_CONT_HIGH_RES]).await {
        error!("Failed to set mode for BH1750: {:?}", e);
        return;
    }
    info!("BH1750FVI initialized");

    // --- 2. Initialize BMP280 ---
    let mut id_buf = [0u8; 1];
    if i2c.write_read(BMP280_ADDR, &[0xD0], &mut id_buf).await.is_err() || id_buf[0] != 0x58 {
        error!("BMP280 not found! Check wiring. (Read ID: {:#04x})", id_buf[0]);
        return;
    }

    let mut calib_buf = [0u8; 24];
    if i2c.write_read(BMP280_ADDR, &[0x88], &mut calib_buf).await.is_err() {
        error!("Failed to read BMP280 calibration data");
        return;
    }
    let calib = parse_calibration(&calib_buf);

    // Register 0xF4: Normal mode (0b11), Temp oversampling x1 (0b001), Press oversampling x1 (0b001)
    if i2c.write(BMP280_ADDR, &[0xF4, 0x27]).await.is_err() {
        error!("Failed to configure BMP280");
        return;
    }
    info!("BMP280 initialized");

    // Give both sensors time to complete their first measurement
    Timer::after_millis(200).await;

    // Buffers for the main loop
    let mut bh1750_buf = [0u8; 2];
    let mut bmp280_buf = [0u8; 6];

    loop {
        // --- Read BH1750FVI ---
        match i2c.read(BH1750FVI_ADDR, &mut bh1750_buf).await {
            Ok(_) => {
                let raw_val = ((bh1750_buf[0] as u16) << 8) | (bh1750_buf[1] as u16);
                let lux = (raw_val as f32) / 1.2;
                info!("Light: {} lx", lux);
            }
            Err(e) => {
                error!("BH1750 read error: {:?}", e);
            }
        }

        // --- Read BMP280 ---
        // Register 0xF7 starts the 6 bytes of pressure and temperature data
        match i2c.write_read(BMP280_ADDR, &[0xF7], &mut bmp280_buf).await {
            Ok(_) => {
                let raw_p = ((bmp280_buf[0] as i32) << 12) | ((bmp280_buf[1] as i32) << 4) | ((bmp280_buf[2] as i32) >> 4);
                let raw_t = ((bmp280_buf[3] as i32) << 12) | ((bmp280_buf[4] as i32) << 4) | ((bmp280_buf[5] as i32) >> 4);

                let (temp_c, pressure_hpa) = compensate(raw_t, raw_p, &calib);
                info!("Temp: {} C, Pressure: {} hPa", temp_c, pressure_hpa);
            }
            Err(e) => {
                error!("BMP280 read error: {:?}", e);
            }
        }

        Timer::after_millis(1000).await;
    }
}

// --- Bosch Calibration Struct and Math Helpers ---

struct Calibration {
    dig_t1: u16, dig_t2: i16, dig_t3: i16,
    dig_p1: u16, dig_p2: i16, dig_p3: i16, dig_p4: i16,
    dig_p5: i16, dig_p6: i16, dig_p7: i16, dig_p8: i16, dig_p9: i16,
}

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

fn compensate(raw_t: i32, raw_p: i32, calib: &Calibration) -> (f32, f32) {
    // Temperature Compensation
    let var1 = (((raw_t >> 3) - ((calib.dig_t1 as i32) << 1)) * (calib.dig_t2 as i32)) >> 11;
    let var2 = (((((raw_t >> 4) - (calib.dig_t1 as i32)) * ((raw_t >> 4) - (calib.dig_t1 as i32))) >> 12) * (calib.dig_t3 as i32)) >> 14;
    let t_fine = var1 + var2;
    let temp_c = ((t_fine * 5 + 128) >> 8) as f32 / 100.0;

    // Pressure Compensation (64-bit precision)
    let mut p_var1 = (t_fine as i64) - 128000;
    let mut p_var2 = p_var1 * p_var1 * (calib.dig_p6 as i64);
    p_var2 = p_var2 + ((p_var1 * (calib.dig_p5 as i64)) << 17);
    p_var2 = p_var2 + ((calib.dig_p4 as i64) << 35);
    p_var1 = ((p_var1 * p_var1 * (calib.dig_p3 as i64)) >> 8) + ((p_var1 * (calib.dig_p2 as i64)) << 12);
    p_var1 = (((1i64 << 47) + p_var1) * (calib.dig_p1 as i64)) >> 33;

    let mut pressure_hpa = 0.0;
    if p_var1 != 0 {
        let mut p = 1048576i64 - (raw_p as i64);
        p = (((p << 31) - p_var2) * 3125) / p_var1;
        p_var1 = ((calib.dig_p9 as i64) * (p >> 13) * (p >> 13)) >> 25;
        p_var2 = ((calib.dig_p8 as i64) * p) >> 19;
        p = ((p + p_var1 + p_var2) >> 8) + ((calib.dig_p7 as i64) << 4);

        pressure_hpa = (p as f32) / 256.0 / 100.0;
    }

    (temp_c, pressure_hpa)
}