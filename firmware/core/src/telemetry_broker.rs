//! Gathering all telemetry and sending it to the server

use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use defmt::info;
use embassy_time::Timer;
use shared::telemetry::NodeTelemetry;

/// Shared telemetry mutex
pub static TELEMETRY: Mutex<ThreadModeRawMutex, NodeTelemetry> = Mutex::new(NodeTelemetry::new());

/// Main broker task
#[embassy_executor::task]
pub async fn publish() {
    loop {
        // Needed to drop the lock
        let t = {
            let t = TELEMETRY.lock().await;
            t.clone()
        };
        info!("{}", t);
        Timer::after_secs(5).await;
    }
}