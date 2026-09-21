//! Abstract and scalable plant channels

use defmt::{error, info};

use embassy_futures::join::join;

use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;

use embassy_time::{Instant, Timer};

use embedded_hal::digital::OutputPin;

use shared::MAX_PLANT_CHANNELS;
use shared::telemetry::PlantTelemetry;

use crate::adc::AdcProvider;
use crate::settings::DYNAMIC_SETTINGS;
use crate::telemetry::TELEMETRY;

/// Watering signals array
pub static WATERING_SIGNALS:
    [Signal<CriticalSectionRawMutex, ()>; MAX_PLANT_CHANNELS] = [const { Signal::new() }; MAX_PLANT_CHANNELS];

/// Monitor and report soil moisture
async fn monitor_moisture<ADC, PIN, Word, P>(
    idx: usize,
    adc_bus: &'static Mutex<NoopRawMutex, ADC>,
    mut moisture_pin: PIN,
    mut moisture_power_pin: P,
) where
    ADC: AdcProvider<PIN, Word>,
    Word: Into<f32>,
    P: OutputPin,
{
    let mut settings = DYNAMIC_SETTINGS.receiver().unwrap();

    loop {
        let current_settings = settings.get().await;
        if !current_settings.plant_settings[idx].enabled {
            // Channel is disabled, sleep until settings are changed
            settings.changed().await;
            continue;
        }

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
                // for analog soil moisture sensors -> high = dry, low = wet
                let percentage = 100.0 - ((val_f32 / max_f32) * 100.0);
                let stamp = Instant::now().as_millis();

                let mut t = TELEMETRY.lock().await;
                t.plant_telemetry[idx] = Some(PlantTelemetry::new(stamp, percentage));
            },
            Err(e) => {
                error!("Error reading soil moisture: {}", e)
            }
        }

        moisture_power_pin.set_low().unwrap();

        let wait = current_settings.plant_settings[idx].moisture_m_freq_s;
        Timer::after_secs(wait as u64).await;
    }
}

/// Water on command
async fn water_on_signal<P: OutputPin>(idx: usize, mut pump_pin: P) {
    let mut settings = DYNAMIC_SETTINGS.receiver().unwrap();

    loop {
        let current_settings = settings.get().await;
        if !current_settings.plant_settings[idx].enabled {
            // Channel is disabled, sleep until settings are changed
            settings.changed().await;
            continue;
        }

        // Wait for watering command
        WATERING_SIGNALS[idx].wait().await;

        // Water
        info!("Watering channel {}", idx);
        pump_pin.set_high().unwrap();
        let duration = current_settings.plant_settings[idx].watering_time_s as u64;
        Timer::after_secs(duration).await;
        pump_pin.set_low().unwrap();
    }
}

/// Plant channel task
/// Has control over its moisture sensor and pump
pub async fn plant_channel<ADC, PIN, Word, P>(
    idx: usize,
    adc_bus: &'static Mutex<NoopRawMutex, ADC>,
    moisture_pin: PIN,
    moisture_power_pin: P,
    pump_pin: P,
) where
    ADC: AdcProvider<PIN, Word>,
    Word: Into<f32>,
    P: OutputPin,
{
    join(
        monitor_moisture(idx, adc_bus, moisture_pin, moisture_power_pin),
        water_on_signal(idx, pump_pin)
    ).await;
}