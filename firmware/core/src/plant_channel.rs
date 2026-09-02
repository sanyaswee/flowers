//! Abstract and scalable plant channels

use defmt::error;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;

use embassy_time::Timer;

use embedded_hal::digital::OutputPin;

use crate::adc::AdcProvider;
use crate::settings;
use crate::telemetry::TELEMETRY;

/// Plant channel task
/// Has control over its moisture sensor and pump
pub async fn plant_channel<ADC, PIN, Word, P>(
    idx: usize,
    adc_bus: &'static Mutex<NoopRawMutex, ADC>,
    mut moisture_pin: PIN,
    mut moisture_power_pin: P,
    _pump_pin: P, // TODO
) where
    ADC: AdcProvider<PIN, Word>,
    Word: Into<f32>,
    P: OutputPin,
{
    loop {
        moisture_power_pin.set_high().unwrap();
        Timer::after_secs(2).await;

        let (reading, max) = {
            let mut adc = adc_bus.lock().await;
            let r = adc.read(&mut moisture_pin).await;
            let m = adc.max_value();
            (r, m)
        };

        match reading {
            Ok(raw) => {
                // convert to f32
                let val_f32: f32 = raw.into();
                let max_f32: f32 = max.into();

                // calculate percentage
                let percentage = (val_f32 / max_f32) * 100.0;

                let mut t = TELEMETRY.lock().await;
                t.plant_telemetry[idx].soil_moisture = Some(percentage);
            },
            Err(e) => {
                error!("Error reading soil moisture: {}", e)
            }
        }

        moisture_power_pin.set_low().unwrap();
        Timer::after_secs(settings::MOISTURE_M_FREQ_S).await;
    }
}