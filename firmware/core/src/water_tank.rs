//! Operations related to water tank

use defmt::error;

use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;

use embedded_hal::digital::OutputPin;

use crate::adc::AdcProvider;
use crate::telemetry::TELEMETRY;

/// Signal that pump sends after watering operation
pub(crate) static MEASURE_TANK_SIG: Signal<CriticalSectionRawMutex, ()> = Signal::new();

/// Read the water tank level
pub async fn read_level<ADC, PIN, Word, P>(
    adc_bus: &'static Mutex<NoopRawMutex, ADC>,
    mut pin: PIN,
    mut power_pin: P,
) where
    ADC: AdcProvider<PIN, Word>,
    Word: Into<f32>,
    P: OutputPin,
{
    loop {
        // Power up the sensor
        power_pin.set_high().unwrap();
        Timer::after_secs(2).await;

        let (reading, max) = {
            let mut adc = adc_bus.lock().await;
            let r = adc.read(&mut pin).await;
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
                t.water_tank_level = Some(percentage);
                if percentage > 10.0 {
                    t.water_tank_has_water = Some(true);
                } else {
                    t.water_tank_has_water = Some(false);
                }
            }
            Err(e) => {
                error!("Error reading water tank ADC: {}", e);
            }
        };
        power_pin.set_low().unwrap();
        
        // Wait until a pump operation
        MEASURE_TANK_SIG.wait().await;
        Timer::after_secs(5).await; // wait for water to calm down
    }
}
